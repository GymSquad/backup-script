mod website_checker;

use crate::{config::Config, db::Database};
use color_eyre::Result;

use self::website_checker::WebsiteChecker;

pub async fn main(config: Config) -> Result<()> {
    let db = Database::connect(&config.database.url).await?;

    let websites = db.get_websites().await?;
    let checker = WebsiteChecker::new();

    let num_urls = config
        .runner
        .as_ref()
        .and_then(|r| r.num_runs.map(|n| n as usize))
        .unwrap_or(websites.len());

    for website in websites.into_iter().take(num_urls) {
        let checker = checker.clone();
        let db = db.clone();
        tokio::spawn(async move {
            use website_checker::WebsiteStatus::*;

            let (_url, is_valid) = match checker.request_check(website.url.clone()).await {
                Ok(Valid) => (website.url, true),
                Ok(Redirected(url)) => (url, true),
                Ok(Dead) => (website.url, false),
                Err(_) => return,
            };

            if website.is_valid != is_valid {
                db.update_website_status(&website.id, is_valid)
                    .await
                    .unwrap();
            }

            if !is_valid {
                return;
            }

            todo!("archive website");
        });
    }

    Ok(())
}
