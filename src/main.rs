mod archive;
mod config;
mod db;

use clap::Parser;
use color_eyre::Result;
use config::Config;

#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Opts {
    /// Custom config file location
    #[clap(default_value = "archive.toml")]
    config: String,
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let Opts { config } = Opts::parse();

    let config = Config::try_read(config)?;

    let mut builder = tokio::runtime::Builder::new_multi_thread();

    if let Some(threads) = config.runner.as_ref().and_then(|r| r.num_threads) {
        builder.worker_threads(threads as usize);
    }

    let runtime = builder.enable_all().build()?;

    runtime.block_on(async move {
        tokio::select! {
            result = archive::main(config) => {
                match result {
                    Ok(_) => println!("Archive completed successfully!"),
                    Err(e) => println!("Archive failed with error: {}", e),
                }
            }
            _ = tokio::signal::ctrl_c() => {
                println!("Ctrl-C received, shutting down...");
            }
        }
    });

    Ok(())
}
