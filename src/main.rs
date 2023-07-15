mod archive;
mod config;
mod db;

use clap::Parser;
use color_eyre::Result;
use config::Config;
use time::{macros::format_description, UtcOffset};
use tracing_subscriber::fmt::time::OffsetTime;

#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Opts {
    /// Custom config file location
    #[clap(default_value = "archive.toml")]
    config: String,
}

fn main() -> Result<()> {
    let timer = OffsetTime::new(
        UtcOffset::from_hms(8, 0, 0).unwrap(),
        format_description!("[year]-[month]-[day] [hour]:[minute]:[second]"),
    );
    tracing_subscriber::fmt()
        .with_target(false)
        .with_timer(timer)
        .with_level(true)
        .init();
    color_eyre::install()?;

    let opts = Opts::parse();

    tracing::info!("Reading config from {}", opts.config);
    let config = Config::try_read(opts.config)?;

    let mut builder = tokio::runtime::Builder::new_multi_thread();

    if let Some(threads) = config.runner.as_ref().and_then(|r| r.num_threads) {
        builder.worker_threads(threads as usize);
    }

    let runtime = builder.enable_all().build()?;

    runtime.block_on(async move {
        tokio::select! {
            result = archive::main(config) => {
                match result {
                    Ok(_) => tracing::info!("Archive completed successfully!"),
                    Err(e) => tracing::error!("Archive failed with error: {}", e),
                }
            }
            _ = tokio::signal::ctrl_c() => {
                tracing::info!("Ctrl-C received, shutting down...");
            }
        }
    });

    Ok(())
}
