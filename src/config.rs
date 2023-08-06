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
#[serde(rename_all = "kebab-case")]
pub struct ArchiveConfig {
    pub output_dir: PathBuf,
    pub command: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct RunnerConfig {
    pub num_threads: Option<usize>,
    pub num_runs: Option<usize>,
    pub num_workers: Option<usize>,
}

impl Config {
    pub fn try_read(path: impl AsRef<Path>) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .wrap_err("Failed to read config file")
            .with_suggestion(|| "Check if the specified file name is correct")?;
        let config = toml::from_str(&content)
            .wrap_err("Failed to parse config file")
            .with_suggestion(|| "Check if the fields or syntax is correct")?;
        Ok(config)
    }
}
