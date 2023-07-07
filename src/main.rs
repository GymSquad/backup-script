mod config;

use color_eyre::Result;
use config::Config;

fn main() -> Result<()> {
    color_eyre::install()?;

    let config_file = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "archive.toml".to_string());

    let config = match Config::try_read(config_file) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to read config file: {}", e);
            std::process::exit(1);
        }
    };

    println!("config: {config:#?}");

    Ok(())
}
