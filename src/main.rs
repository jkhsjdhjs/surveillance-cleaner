extern crate chrono;
#[macro_use]
extern crate structopt;

use chrono::{TimeZone, Utc};
use std::fs;
use std::io::{Error, ErrorKind, Result};
use structopt::StructOpt;

#[derive(StructOpt)]
struct CliOpts {
    /// The directory to keep clean
    #[structopt(name = "DIR", parse(from_os_str))]
    dir: std::path::PathBuf,

    /// Delete files older than <age> days
    #[structopt(short = "a", long = "age", default_value = "30")]
    age: u16,
}

fn delete_if_old_age<C>(f: fs::DirEntry, is_old_age: C) -> Result<bool>
where
    C: Fn(u16) -> bool,
{
    let filename = f.file_name();
    let filename_str = filename
        .to_str()
        .ok_or(Error::new(
            ErrorKind::Other,
            "Error converting filename to &str",
        ))?
        .split(".")
        .next()
        .ok_or(Error::new(ErrorKind::Other, "Error splitting filename"))?;
    let remove_unwrap = 0;
    let age = (Utc::now()
        - Utc
            .datetime_from_str(filename_str, "%Y-%m-%d_%H-%M-%S")
            .unwrap())
        .num_days();
    if age.is_positive() && is_old_age(age.abs() as u16) {
        fs::remove_file(f.path())?;
        return Ok(true);
    }
    Ok(false)
}

fn main() -> Result<()> {
    let cli_opts = CliOpts::from_args();
    let deleted_count = fs::read_dir(cli_opts.dir.clone())?
        .filter_map(|f| f.map_err(|e| eprintln!("Error reading file: {}", e)).ok())
        .map(|f| delete_if_old_age(f, |age| age >= cli_opts.age))
        .filter_map(Result::ok)
        .filter(|r| *r)
        .count();
    println!("Deleted {} files!", deleted_count);
    Ok(())
}
