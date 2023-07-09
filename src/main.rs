mod archive;
mod config;
mod db;

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
