use anyhow::Result;
use clap::Parser;
use std::time::SystemTime;
use walkdir::WalkDir;
use indicatif::ProgressIterator;

#[macro_use]
extern crate log;

// HTTP cache optimised for Shimmie galleries
#[derive(Parser, Clone)]
#[clap(author, about, long_about = None)]
pub struct Args {
    /// Where the cached files should be stored
    #[clap(short = 'c', default_value = "/data/shm_cache/")]
    pub cache: String,

    /// Delete files if we have less than this much free space
    #[clap(short = 'f', long = "free", default_value = "10.0")]
    pub free: f64,

    /// Delete files for real (otherwise just print what would be deleted)
    #[clap(short = 'd', long = "delete")]
    pub delete: bool,

    /// Show version
    #[structopt(long = "version")]
    pub version: bool,
}

fn main() -> Result<()> {
    pretty_env_logger::init();

    let args = Args::parse();
    info!(
        "shm-decimate {} built on {}",
        env!("VERGEN_GIT_SHA").chars().take(7).collect::<String>(),
        env!("VERGEN_BUILD_DATE"),
    );
    if args.version {
        return Ok(());
    }

    let free = disk_free_percent(&args.cache)?;
    info!("Disk has {:.2}% free", free);
    if free > args.free {
        return Ok(());
    }

    decimate(&args.cache, args.delete)?;

    Ok(())
}

fn disk_free_percent(path: &String) -> Result<f64> {
    let free = fs2::available_space(&path)? as f64;
    let total = fs2::total_space(&path)? as f64;
    Ok(free / total * 100.0)
}

fn decimate(path: &String, delete: bool) -> Result<()> {
    let mut files = Vec::new();
    for entry in WalkDir::new(&path) {
        let entry = entry?;
        if entry.file_type().is_file() {
            let atime = entry
                .metadata()?
                .accessed()?
                .duration_since(SystemTime::UNIX_EPOCH)?
                .as_secs();
            files.push((atime, entry.into_path()));
        }
    }
    files.sort_unstable();

    let ten_percent = files.len() / 10;
    info!("{} files found - removing {} of them", files.len(), ten_percent);
    for (atime, path) in files.into_iter().take(ten_percent).progress() {
        debug!("Removing {:?} (last accessed {:?})", path, atime);
        // std::thread::sleep(std::time::Duration::from_millis(100));
        if delete {
            std::fs::remove_file(path)?;
        }
    }

    Ok(())
}
