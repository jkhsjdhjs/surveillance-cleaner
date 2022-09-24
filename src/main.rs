extern crate chrono;
extern crate clap;

use chrono::{TimeZone, Utc};
use std::fs;
use clap::Parser;

#[derive(Parser)]
struct CliOpts {
    /// The directory to keep clean
    #[structopt(name = "DIR", parse(from_os_str))]
    dir: std::path::PathBuf,

    /// Delete files older than <age> days
    #[structopt(short = 'a', long = "age", default_value = "30")]
    age: u16,
}

fn delete_if_old_age<C>(f: fs::DirEntry, is_old_age: C) -> Result<bool, String>
where
    C: Fn(u16) -> bool,
{
    let filename_borrowed = f.file_name();
    let filename = filename_borrowed
        .to_str()
        .ok_or("Error converting a filename to &str")?;
    let filename_without_ext = filename
        .split(".")
        .next()
        .ok_or(format!("Error splitting filename: {}", filename))?;
    let age = (Utc::now()
        - Utc
            .datetime_from_str(filename_without_ext, "%Y-%m-%d_%H-%M-%S")
            .or(Err(format!(
                "Failed to parse date from filename: {}",
                filename
            )))?)
    .num_days();
    if age.is_positive() && is_old_age(age as u16) {
        fs::remove_file(f.path()).or(Err("error removing file"))?;
        return Ok(true);
    }
    Ok(false)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli_opts = CliOpts::from_args();
    let deleted_count = fs::read_dir(cli_opts.dir.clone())
        .expect("Failed to read specified directory!")
        .filter_map(|f| f.map_err(|e| eprintln!("Error reading file: {}", e)).ok())
        .map(|f| delete_if_old_age(f, |age| age >= cli_opts.age))
        .filter_map(|f| f.map_err(|e| eprintln!("{}", e)).ok())
        .filter(|r| *r)
        .count();
    println!("Deleted {} files!", deleted_count);
    Ok(())
}
