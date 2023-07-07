use std::path::{Path, PathBuf};

use color_eyre::{eyre::Context, Help, Result};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub database: DatabaseConfig,
    pub archive: ArchiveConfig,
    pub runner: Option<RunnerConfig>,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct ArchiveConfig {
    #[serde(alias = "output-dir")]
    pub output_dir: PathBuf,
    #[serde(alias = "accept-format")]
    pub accept_format: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct RunnerConfig {
    #[serde(alias = "num-threads")]
    pub num_threads: Option<u32>,
    #[serde(alias = "num-runs")]
    pub num_runs: Option<u32>,
}

impl Config {
    pub fn try_read<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let content = std::fs::read_to_string(path)
            .wrap_err("Failed to read config file")
            .with_suggestion(|| "Check if the specified file name is correct")?;
        let config = toml::from_str(&content)
            .wrap_err("Failed to parse config file")
            .with_suggestion(|| "Check if the fields or syntax is correct")?;
        Ok(config)
    }
}
