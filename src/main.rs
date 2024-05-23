#![allow(dead_code)]
#![allow(unused_must_use)]

use clap::{Parser, Subcommand};
use std::{
    env,
    path::PathBuf,
    process::Command,
    time::{Duration, SystemTime},
};
use uuid::*;
use filesort::*;

/*

*/

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Sorts the files in the current directory
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Sorts files based on file extension matching our database
    Sort {
        /// The input directory
        #[arg(short, long)]
        inputdir: String,

        /// The output directory
        #[arg(short, long)]
        outputdir: String,

        /// Number of directory levels (1-3)
        #[arg(short, long, default_value_t = 2)]
        nesting_level: u8,

        /// Use alternative sorting directory name
        #[arg(short, long, default_value_t = false)]
        use_alt: bool,

        /// Verbose mode
        #[arg(short, long, default_value_t = false)]
        verbose: bool,

        /// Generates a log file
        #[arg(short, long, default_value_t = false)]
        log: bool,
    },
    /// Creates a specified amount of files
    Create {
        /// The amount of files to create
        #[arg(short, long)]
        amount: u32,
    },
    /// Sorts files based on custom file extensions
    Customsort {
        /// The input directory
        #[arg(short, long)]
        inputdir: String,

        /// The output directory
        #[arg(short, long)]
        outputdir: String,

        /// The file extension to sort
        #[arg(short, long)]
        extension: String,

        /// Verbose mode
        #[arg(short, long, default_value_t = false)]
        verbose: bool,

        /// Generates a log file
        #[arg(short, long, default_value_t = false)]
        log: bool,
    },
    /// Note: Only run in a new empty directory. Runs a benchmark test
    Benchmark {},
}

fn main() {
    let cli = Cli::parse();

    let start = SystemTime::now();
    match &cli.command {
        Some(Commands::Sort {
            inputdir,
            outputdir,
            nesting_level,
            use_alt,
            verbose,
            log,
        }) => {
            let in_dir = PathBuf::from(inputdir);
            let out_dir = PathBuf::from(outputdir);

            if !in_dir.is_dir() {
                panic!("Provided path is not a directory: '{:?}'", in_dir)
            }

            sort_files(in_dir, out_dir, *nesting_level, *use_alt, *verbose, *log);
            let end = SystemTime::now();
            let duration = end.duration_since(start).unwrap();
            println!("Time taken: {:?}", duration);
        }
        Some(Commands::Customsort {
            inputdir,
            outputdir,
            extension,
            verbose,
            log,
        }) => {
            let end = SystemTime::now();
            let duration = end.duration_since(start).unwrap();
            custom_sort(inputdir, outputdir, extension, *verbose, *log);
        }
        Some(Commands::Create { amount }) => {
            create_files(amount + 1);
            let end = SystemTime::now();
            let duration = end.duration_since(start).unwrap();
            println!("Time taken: {:?}", duration);

            // if !cli.disable_telemetry {
            //     collect_telemetry(
            //         "N/A".to_string(),
            //         "N/A".to_string(),
            //         "N/A",
            //         "N/A",
            //         "N/A",
            //         "N/A",
            //         "N/A".to_string(),
            //         amount.to_string().as_str(),
            //         "Create Files",
            //         duration,
            //     );
            // }
        }
        Some(Commands::Benchmark { .. }) => {
            let time = benchmark();
            println!("Time Taken: {:?}", time);
            if !cli.disable_telemetry {
                collect_telemetry(
                    "N/A".to_string(),
                    "N/A".to_string(),
                    "N/A",
                    "N/A",
                    "N/A",
                    "N/A",
                    "N/A".to_string(),
                    "N/A",
                    "Benchmark",
                    time,
                );
            }
        }
        None => println!("No command provided. Use 'filesort --help' for more information."),
    }
}
