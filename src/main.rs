mod archive;
mod config;

use color_eyre::Result;
use config::Config;

fn main() -> Result<()> {
    color_eyre::install()?;

    let config_file = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "archive.toml".to_string());

    let config = Config::try_read(config_file)?;

    let mut builder = tokio::runtime::Builder::new_multi_thread();

    if let Some(threads) = config.runner.as_ref().and_then(|r| r.num_threads) {
        builder.worker_threads(threads as usize);
    }

    let runtime = builder.enable_all().build()?;

    Ok(runtime.block_on(async {
        tokio::select! {
            _ = archive::main() => {
                println!("Archive completed successfully!");
            }
            _ = tokio::signal::ctrl_c() => {
                println!("Ctrl-C received, shutting down...");
            }
        }
    }))
}
