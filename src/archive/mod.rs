mod controller;
mod website_checker;

use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

use crate::{config::Config, db::Database};
use color_eyre::Result;

use self::controller::ArchiveController;
use self::website_checker::WebsiteChecker;

static COMMAND: OnceLock<Vec<String>> = OnceLock::new();
static OUTPUT_PATH: OnceLock<PathBuf> = OnceLock::new();

pub async fn main(config: Config) -> Result<()> {
    let mut command = COMMAND
        .get_or_init(|| config.archive.command.clone())
        .iter()
        .map(|s| s.as_str());
    let output_path = OUTPUT_PATH.get_or_init(|| config.archive.output_dir.clone());

    let db = Database::connect(&config.database.url).await?;
    let program: Arc<str> = command.next().unwrap_or("wget").into();
    let command_args = command.collect::<Arc<_>>();
    let output_path: Arc<Path> = Arc::from(output_path.as_path());

    let websites = db.get_websites().await?;
    let checker = WebsiteChecker::new();

    let mut controller = ArchiveController::new(db, checker, program, command_args, output_path);

    let num_urls = config
        .runner
        .as_ref()
        .and_then(|r| r.num_runs.map(|n| n as usize))
        .unwrap_or(websites.len());

    tracing::info!("Archiving {} websites", num_urls);

    for website in websites.into_iter().take(num_urls) {
        controller.archive(website);
    }

    controller.wait().await;

    Ok(())
}
