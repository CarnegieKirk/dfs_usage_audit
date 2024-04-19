// use std::os::macos::fs::MetadataExt;
use filetime::{self, FileTime};
use std::{error::Error, fs};
extern crate chrono;
use chrono::{DateTime, Duration, Utc};
use rayon::prelude::*;
use std::path::Path;
// use walkdir::WalkDir;
use csv::Writer;
use jwalk::WalkDir;
use std::sync::Mutex;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    path: String,

    #[arg(short, long, default_value_t = 50)]
    threads: usize,
    #[arg(short, long, default_value_t = 1095)]
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
    // Shows the timestamp of the last access date.
    let access_time = FileTime::from_last_access_time(&metadata).unix_seconds();
    let datetime = DateTime::<Utc>::from_timestamp(access_time, 0).expect("A Valid date time.");
    // Format the datetime how you want
    let readable_time = &datetime.format("%Y-%m-%d %H:%M:%S");
    // Print the newly formatted date and time
    // Shows file not accessed within the last X days.
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
  // Three years
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

fn visit_dirs(dir: &Path, threads: usize, access_cutoff: i64) -> Vec<FileResult> {
    // let mut counter = 0;
    let start = std::time::Instant::now();
    let all_files = Mutex::new(Vec::new()); // Use Mutex for interior mutability
    let entries: Vec<_> = WalkDir::new(dir)
        .parallelism(jwalk::Parallelism::RayonNewPool(threads))
        .into_iter()
        .collect();
    println!("Total files:: \x1b[0;31m{:?}\x1b[0m", &entries.len());
    println!("Building Vec took: \x1b[0;32m{:?}\x1b[0m", start.elapsed());
    if dir.is_dir() {
        // Thread count for the par_iter()
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .expect("A Valid Rayon pool.");
        entries.par_iter().for_each(|entry| match entry {
            Ok(entry) => {
                let path = entry.path();
                match return_access_stamp(&path, access_cutoff) {
                    Ok(result) => {
                        let mut guard = all_files.lock().expect("A Valid Mutex Guard"); // Lock the Mutex
                        guard.push(result); // Mutate the Vec inside the Mutex
                    }
                    Err(err) => {
                        if let Some(io_err) = err.downcast_ref::<std::io::Error>() {
                            if io_err.kind() == std::io::ErrorKind::PermissionDenied {
                                println!(
                                    "System Error: \x1b[0;31m{}\x1b[0m - {}",
                                    path.to_string_lossy(),
                                    err
                                );
                            } else {
                                println!("Not found error: \x1b[0;31m{}\x1b[0m", err);
                            }
                        }
                    }
                }
            }
            Err(err) => {
                println!("Permissions Error: \x1b[0;31m{}\x1b[0m", err);
            }
        });
    } else {
        println!("Not a directory: \x1b[0;31m{:?}\x1b[0m", dir);
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
    let out_file = "DFS_audit.csv";
    // "Benchmarking"
    let start = std::time::Instant::now();
    let processed_files = visit_dirs(path, threads, access_cufoff);
    let time_processing = start.elapsed();
    let untouched_files = processed_files.len();
    let middle = std::time::Instant::now();
    match write_data(processed_files, out_file) {
        Ok(_) => {
            println!("Data written to {}", out_file);
        }
        Err(err) => {
            eprintln!("Error {}", err);
        }
    }
    println!(
        "Time taken processing: \x1b[0;32m{:?}\x1b[0m",
        time_processing
    );
    println!(
        "Time taken printing output: \x1b[0;32m{:?}\x1b[0m",
        middle.elapsed()
    );
    println!("Total time taken: \x1b[0;32m{:?}\x1b[0m", start.elapsed());
    println!("Untouched files:: \x1b[0;31m{:?}\x1b[0m", untouched_files);
}
