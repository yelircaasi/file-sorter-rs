mod config;
#[cfg(test)]
mod tests;

use config::EXTENSIONS;
use rand::Rng;
use std::{
    collections::HashMap,
    ffi::OsStr,
    fs,
    io::Write,
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};

pub fn create_files(amount: u32) {
    for file in 1..amount {
        let mut file_name = String::new();
        file_name.push_str(&file.to_string());
        file_name.push('.');
        let mut rng = rand::thread_rng();
        let random_extension = EXTENSIONS[rng.gen_range(0..EXTENSIONS.len())].0;
        file_name.push_str(random_extension);
        let _file = fs::File::create(file_name).expect("Failed to create file");
    }
}

pub fn custom_sort(
    input_directory: &str,
    output_directory: &str,
    extension: &str,
    verbose: bool,
    log: bool,
) {
    // Set up the directories
    let input_directory = Path::new(input_directory);
    let output_directory = Path::new(output_directory);

    // Get all the files in the input directory
    let files = fs::read_dir(input_directory).unwrap();

    // Loop through each file and move it to the appropriate output directory
    for file in files {
        let file = file.unwrap().path();
        let _file_name = match file.file_name() {
            Some(file_name) => file_name,
            None => continue,
        };

        match file.extension() {
            Some(ext) if ext == extension => {
                fs::create_dir_all(output_directory).unwrap();
                let output_file = output_directory.join(file.file_name().unwrap());
                fs::rename(file.clone(), output_file).unwrap();
            }
            _ => continue,
        }

        if verbose {
            println!("Moved file: {:?} to {:?}", file, output_directory);
        }

        if log {
            write_logfile(
                file.as_os_str(),
                output_directory,
                input_directory.to_str().unwrap(),
            );
        }
    }
}

/// # Usage
/// ```markdown
/// (ext, (type, alt, sorted_dir)),
///
/// ("gif", ("image", Some("animated"), None)),
/// ("qt", ("video", None, Some("quicktime"))),
/// ("mp4", ("video", None, None)),
///
/// nesting_level, use_alt => gif, qt, mp4
///
/// 1, false => "image", "video", "video"
/// 2, false => "image/gif", "video/quicktime", "video/mp4"
/// 3, false => "image/gif", "video/quicktime", "video/mp4"
///
/// 1, true => "image", "video", "video"
/// 2, true => "image/animated", "video/quicktime", "video/mp4"
/// 3, true => "image/animated/gif", "video/quicktime", "video/mp4"
/// ```
pub fn get_subdir_by_extension(ext: &str, nesting_level: u8, use_alt: bool) -> PathBuf {
    if !(1..=3).contains(&nesting_level) {
        panic!("Nesting level is out of range.");
    }

    let extensions: HashMap<&str, (&str, Option<&str>, Option<&str>)> =
        HashMap::from(config::EXTENSIONS);

    let ext_data = match extensions.get(ext) {
        None => return PathBuf::from("other"),
        Some(e) => e,
    };

    let mut path = PathBuf::from(ext_data.0);

    match (nesting_level, use_alt) {
        (1, _) => {} // Do nothing
        (2, true) => {
            path.push(ext_data.1.unwrap_or(ext_data.2.unwrap_or(ext))); // use alt, then use sorted_dir, then use provided ext.
        }
        (3, true) => {
            if ext_data.1.is_some() {
                path.push(ext_data.1.unwrap())
            }
            path.push(ext_data.2.unwrap_or(ext));
        }
        (_, false) => {
            // 2 or 3
            // If sorted_dir is present in config, use it, otherwise fallback to provided one.
            path.push(ext_data.2.unwrap_or(ext));
        }
        _ => {
            panic!(
                "{} | get_subdir_by_extension() | nesting_level: {nesting_level}, use_alt: {use_alt}",
                file!()
            )
        }
    }

    path
}

pub fn write_logfile(file_name: &OsStr, moveto_directory: &Path, input_directory: &str) -> bool {
    let logdir = Path::new(input_directory).join("sorter-logs/");
    fs::create_dir_all(logdir.clone()).unwrap();
    let mut logfile = fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(logdir.to_str().unwrap().to_owned() + "sorter.log")
        .expect("create failed");

    logfile
        .write_all(format!("{:?}", file_name).as_bytes())
        .expect("write failed");
    logfile
        .write_all(" Moved to ".as_bytes())
        .expect("write failed");
    logfile
        .write_all(format!("{:?}\n", moveto_directory.display()).as_bytes())
        .expect("write failed");

    true
}

pub fn sort_files(
    in_dir: PathBuf,
    out_dir: PathBuf,
    nesting_level: u8,
    use_alt: bool,
    verbose: bool,
    log: bool,
) -> std::io::Result<()> {
    for entry in fs::read_dir(in_dir.clone())? {
        let path = entry?.path();
        let file_name = match path.file_name() {
            None => continue,
            Some(f) => f,
        };
        let ext = match path.extension() {
            None => continue,
            Some(e) => e,
        };

        let moveto_directory = out_dir.join(get_subdir_by_extension(
            ext.to_str().unwrap(),
            nesting_level,
            use_alt,
        ));
        fs::create_dir_all(&moveto_directory).unwrap();
        fs::rename(&path, moveto_directory.join(path.file_name().unwrap()))?;

        if verbose {
            println!("{:?} moved to {:?}", file_name, moveto_directory.display());
        }

        if log {
            let log_dir = "sorter-logs";
            fs::create_dir_all(log_dir).unwrap();
            write_logfile(file_name, &moveto_directory, in_dir.to_str().unwrap());
        }
    }

    Ok(())
}

pub fn benchmark() -> Duration {
    let files = fs::read_dir(".");
    if files.is_ok() && files.unwrap().count() > 0 {
        println!("Please run benchmark in an empty directory.");
        return Duration::from_secs(0);
    }

    let startbench = SystemTime::now();
    create_files(10001);
    sort_files(".".into(), "./benchmark".into(), 3, false, false, false)
        .expect("Failed to sort files");
    let endbench = SystemTime::now();
    std::fs::remove_dir_all("./benchmark").expect("Failed to remove benchmark directory");
    endbench.duration_since(startbench).unwrap()
}
