use filetime::{self, FileTime};
use std::{error::Error, fs, sync::Arc};
extern crate chrono;
use chrono::{DateTime, Duration, Utc};
use clap::Parser;
use csv::Writer;
use indicatif::{ProgressBar, ProgressStyle};
use jwalk::WalkDir;
use rayon::prelude::*;
use std::path::Path;
use std::sync::Mutex;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, help="Relative path to export audit results.",default_value_t = String::from("DFS_audit.csv") ) ]
    out_file: String,
    #[arg(
        short,
        help = "The path to the directory/DFS space you wish to audit.",
        long
    )]
    path: String,
    #[arg(
        short = 'D',
        long,
        help = "Whether to include only directories. Does not take a value",
        action
    )]
    directories: bool,
    #[arg(
        short,
        long,
        help = "The amount of threads to use for performance.Higher is faster, but can be less accurate. I've found 50 to be the best tradeoff. Never inaccurate.",
        default_value_t = 50
    )]
    threads: usize,
    #[arg(
        short,
        long,
        help = "Cut-off, in days, to include. e.g. 365 to show files not accessed in the last year.",
        default_value_t = 1095
    )]
    days: i64,
}

#[derive(Debug, Clone)]
struct FileResult {
    path: String,
    accessed: String,
}

impl std::fmt::Display for FileResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.path, self.accessed)
    }
}

fn return_access_stamp(
    file: &Path,
    access_cutoff: i64,
) -> Result<FileResult, Box<dyn std::error::Error>> {
    let metadata = fs::metadata(file)?;
    let access_time = FileTime::from_last_access_time(&metadata).unix_seconds();
    let datetime = DateTime::<Utc>::from_timestamp(access_time, 0).expect("A Valid date time.");
    let readable_time = &datetime.format("%Y-%m-%d %H:%M:%S");
    if !check_within_spec_time(datetime, access_cutoff) {
        let this_file = FileResult {
            path: file.to_string_lossy().to_string(),
            accessed: readable_time.to_string(),
        };
        Ok(this_file)
    } else {
        Err(format!("File accessed at {}.", readable_time).into())
    }
}

/**
  Checks to see if a given date is within the last days_since days.
  ```
  let access_cutoff: i64 = 1095;
  let datetime: DateTime<Utc> = DateTime::from_naive_utc_and_offset(file_time_readable, Utc);
  check_within_spec_time(datetime, access_cutoff);
  ```
**/
fn check_within_spec_time(date: DateTime<Utc>, days_since: i64) -> bool {
    let current_date = Utc::now();
    let time_in_past = current_date - Duration::days(days_since);
    date >= time_in_past && date <= current_date
}

fn visit_dirs(dir: &Path, threads: usize, access_cutoff: i64, dirs_only: bool) -> Vec<FileResult> {
    // let mut counter = 0;
    // Use Mutex for interior mutability
    let start = std::time::Instant::now();
    let all_files = Mutex::new(Vec::new());
    let pb1 = Arc::new(Mutex::new(ProgressBar::new_spinner()));
    // NOTE: Filters out to be only dirs
    let entries: Vec<_> = WalkDir::new(dir)
        .parallelism(jwalk::Parallelism::RayonNewPool(threads))
        .process_read_dir(move |_, _, _, _| {
            pb1.lock().unwrap().inc(1);
        })
        .into_iter()
        .collect();
    println!("Total files:: \x1b[0;31m{:?}\x1b[0m", &entries.len());
    println!(
        "Building Vec took: \x1b[0;32m{:.2?}\x1b[0m",
        start.elapsed()
    );
    if dir.is_dir() {
        // Thread count for the par_iter()
        let pb = ProgressBar::new(entries.len() as u64);
        let pb = Mutex::new(pb);
        pb.lock().expect("Expecting to successfully lock pb").set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] ({pos}/{len}, ETA {eta})",
        )
        .expect("Expected to successfully create a progress bar style.")
    );
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .expect("A Valid Rayon pool.");
        entries.par_iter().for_each(|entry| match entry {
            Ok(entry) => {
                pb.lock()
                    .expect("Expected to lock PB again during OK arm")
                    .inc(1);
                let path = entry.path();
                // NOTE: Match statement determines behavior based on the dirs_only bool which
                // is passed by the user at run time
                match (path.is_dir(), dirs_only) {
                    // Always add directories.
                    (true, true) | (true, false) => {
                        // Check if the entry is a directory
                        match return_access_stamp(&path, access_cutoff) {
                            Ok(result) => {
                                let mut guard = all_files.lock().expect("A Valid Mutex Guard"); // Lock the Mutex
                                guard.push(result); // Mutate the Vec inside the Mutex
                            }
                            Err(err) => {
                                if let Some(io_err) = err.downcast_ref::<std::io::Error>() {
                                    if io_err.kind() == std::io::ErrorKind::PermissionDenied {
                                        // eprintln!("System Error: \x1b[0;31m{}\x1b[0m", err);
                                    } else {
                                        // eprintln!("Not found error: \x1b[0;31m{}\x1b[0m", err);
                                    }
                                }
                            }
                        }
                    }
                    (false, true) => {
                        // We never want to add a file if the dirs_only bool is true
                    }
                    (false, false) => {
                        // Default state: Do both files and dirs.
                        match return_access_stamp(&path, access_cutoff) {
                            Ok(result) => {
                                let mut guard = all_files.lock().expect("A Valid Mutex Guard"); // Lock the Mutex
                                guard.push(result); // Mutate the Vec inside the Mutex
                            }
                            Err(err) => {
                                if let Some(io_err) = err.downcast_ref::<std::io::Error>() {
                                    if io_err.kind() == std::io::ErrorKind::PermissionDenied {
                                        // eprintln!("System Error: \x1b[0;31m{}\x1b[0m", err);
                                    } else {
                                        // eprintln!("Not found error: \x1b[0;31m{}\x1b[0m", err);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Err(_err) => {
                pb.lock()
                    .expect("Expected to lock PB again during err arm")
                    .inc(1);
                // println!("Permissions Error: \x1b[0;31m{}\x1b[0m", err);
            }
        });
    } else {
        eprintln!("Not a directory: \x1b[0;31m{:?}\x1b[0m", dir);
    }
    println!("Total files walked: \x1b[0;32m{}\x1b[0m", entries.len());
    let guard = all_files.lock().expect("A valid mutex lock."); // Lock the Mutex
    guard.clone() // Clone the Vec inside the Mutex
}
fn write_data(data: Vec<FileResult>, filename: &str) -> Result<(), Box<dyn Error>> {
    let header = vec!["path", "accessed"];
    let mut writer = Writer::from_path(filename)?;

    writer.write_record(&header)?;

    for row in data {
        writer.write_record(&[row.path, row.accessed])?;
    }

    Ok(())
}

fn main() {
    // Specify the path to the directory you want to start the recursive iteration
    let args = Args::parse();
    let directory_path = args.path;
    // let directory_path = "/Users/hkirkwoo/Projects";
    println!("Now inspecting \x1b[0;35m{}\x1b[0m", &directory_path);
    // Use the Path type to create a path from the directory path string
    let path = Path::new(&directory_path);
    let threads: usize = args.threads;
    let access_cufoff = args.days;
    let out_file = args.out_file;
    // "Benchmarking"
    let start = std::time::Instant::now();
    let processed_files = visit_dirs(path, threads, access_cufoff, args.directories);
    let untouched_files = processed_files.len();
    match write_data(processed_files, &out_file) {
        Ok(_) => {
            println!("Data written to \x1b[0;32m{}\x1b[0m", out_file);
        }
        Err(err) => {
            eprintln!("Error {}", err);
        }
    }
    println!("Total time taken: \x1b[0;32m{:.2?}\x1b[0m", start.elapsed());
    match args.directories {
        true => {
            println!(
                "Untouched directories: \x1b[0;31m{:?}\x1b[0m",
                untouched_files
            );
        }
        false => {
            println!("Untouched files: \x1b[0;31m{:?}\x1b[0m", untouched_files);
        }
    }
}
