use crate::{config::Config, db::Database};
use color_eyre::Result;

pub async fn main(config: Config) -> Result<()> {
    let db = Database::connect(&config.database.url).await?;

    let websites = db.get_websites().await?;

    println!("Websites: {:?}", websites);

    Ok(())
}
