mod website_checker;

use crate::{config::Config, db::Database};
use color_eyre::Result;

use self::website_checker::WebsiteChecker;

pub async fn main(config: Config) -> Result<()> {
    let db = Database::connect(&config.database.url).await?;

    let websites = db.get_websites().await?;
    let checker = WebsiteChecker::new();

    for website in websites {
        let checker = checker.clone();
        tokio::spawn(async move {
            let _ = checker.request_check(website.url).await;
            todo!("archive website");
        });
    }

    Ok(())
}
