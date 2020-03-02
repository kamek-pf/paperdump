use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::Result;
use argh::FromArgs;
use serde::Deserialize;

#[derive(FromArgs)]
/// Eat your paperwork
struct Args {
    /// path to the configuration file
    #[argh(option, short = 'c')]
    pub config: PathBuf,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub watch_folder: PathBuf,
    pub destination_folder: PathBuf,
    pub language: String,
    #[serde(with = "humantime_serde")]
    pub process_delay: Duration,
}

impl Config {
    pub fn init() -> Result<Config> {
        // Initialize CLI
        let args: Args = argh::from_env();

        // Read config file
        let mut file = File::open(&args.config)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        // Parse TOML
        let parsed = toml::from_str(&contents)?;
        Ok(parsed)
    }
}
