use anyhow::anyhow;
use clap::Parser;
use regex::Regex;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;
use tracing::{debug, error, info};
use walkdir::WalkDir;

fn process_directory(entry: &walkdir::DirEntry, re: &Regex) -> anyhow::Result<()> {
    let dir_name = entry
        .path()
        .file_name()
        .ok_or(anyhow!("No file name"))?
        .to_str()
        .ok_or(anyhow!("No str"))?;

    if let Some(captures) = re.captures(dir_name) {
        if let (Some(ttl_value), Some(ttl_unit)) = (captures.get(1), captures.get(2)) {
            let ttl_value = ttl_value.as_str().parse::<u64>()?;
            let ttl_seconds = match ttl_unit.as_str() {
                "min" => ttl_value * 60,
                "d" => ttl_value * 24 * 60 * 60,
                "m" => ttl_value * 30 * 24 * 60 * 60, // Approximate
                "y" => ttl_value * 365 * 24 * 60 * 60, // Approximate
                _ => return Ok(()),                   // Skip if unit is not recognized
            };

            let metadata = fs::metadata(entry.path())?;
            let creation_time = metadata.created()?;
            let current_time = SystemTime::now();

            if let Ok(duration) = current_time.duration_since(creation_time) {
                if duration.as_secs() > ttl_seconds {
                    info!("Deleting directory: {}", entry.path().display());
                    fs::remove_dir_all(entry.path())?;
                } else {
                    debug!("Directory {} not yet expired", entry.path().display());
                }
            }
        }
    } else {
        debug!("Directory {dir_name} does not match TTL pattern");
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
struct Config {
    paths_to_watch: Vec<PathBuf>,
}

#[derive(Parser, Debug)]
#[command(
    version,
    about = "A tool to clean up directories based on ttl described by directories"
)]
struct Cli {
    /// Path to the yaml configuration file
    #[arg(
        short,
        long,
        value_name = "FILE",
        help = "Specifies the path to the yaml configuration file"
    )]
    config: PathBuf,
}

fn do_main(config: Config) -> anyhow::Result<()> {
    let re = Regex::new(r"^ttl=(\d+)(min|d|m|y)$")?;
    for path in config.paths_to_watch {
        debug!("Processing path: {}", path.display());
        let walker = WalkDir::new(path);
        for entry in walker.into_iter() {
            let entry = entry?;
            if entry.file_type().is_dir() {
                if let Err(e) = process_directory(&entry, &re) {
                    error!(
                        "Error processing directory {}: {}",
                        entry.path().display(),
                        e
                    );
                }
            }
        }
    }
    Ok(())
}

fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    // Read and parse the config file
    let config_content = fs::read_to_string(&cli.config).expect("Failed to read config file");
    let config: Config =
        serde_yaml::from_str(&config_content).expect("Failed to parse config file");

    info!("Starting directory cleanup");
    if let Err(e) = do_main(config) {
        error!("Error: {}", e);
        std::process::exit(1);
    }
    info!("Directory cleanup completed");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    fn create_test_directory(base_path: &Path, name: &str) -> anyhow::Result<()> {
        let dir_path = base_path.join(name);
        fs::create_dir(&dir_path)?;
        Ok(())
    }

    #[test]
    fn test_do_main_with_expired_directory() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        create_test_directory(temp_dir.path(), "ttl=1min")?;

        // Simulate passage of time
        std::thread::sleep(std::time::Duration::from_secs(61));

        let config = Config {
            paths_to_watch: vec![temp_dir.path().to_path_buf()],
        };

        do_main(config)?;

        assert!(!temp_dir.path().join("ttl=1min").exists());
        Ok(())
    }

    #[test]
    fn test_do_main_with_non_expired_directory() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        create_test_directory(temp_dir.path(), "ttl=1d")?;

        let config = Config {
            paths_to_watch: vec![temp_dir.path().to_path_buf()],
        };

        do_main(config)?;

        assert!(temp_dir.path().join("ttl=1d").exists());
        Ok(())
    }

    #[test]
    fn test_do_main_with_non_ttl_directory() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        create_test_directory(temp_dir.path(), "regular_dir")?;

        let config = Config {
            paths_to_watch: vec![temp_dir.path().to_path_buf()],
        };

        do_main(config)?;

        assert!(temp_dir.path().join("regular_dir").exists());
        Ok(())
    }
}
