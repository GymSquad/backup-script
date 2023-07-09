use color_eyre::{eyre::Context, Result};
use sqlx::{Pool, Postgres};

pub struct Database {
    pool: Pool<Postgres>,
}

impl Database {
    pub async fn connect(database_url: &str) -> Result<Self> {
        let pool = Pool::connect(database_url).await?;
        Ok(Self { pool })
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct Website {
    pub id: String,
    pub url: String,
    pub is_valid: bool,
}

impl Database {
    pub async fn get_websites(&self) -> Result<Vec<Website>> {
        sqlx::query_as::<_, Website>(
            r#"
            SELECT "id", "url", "isValid" as is_valid
            FROM "Website"
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .wrap_err("Failed to fetch websites from database")
    }

    pub async fn update_website(&self, website: &Website) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE "Website"
            SET "isValid" = $1
            WHERE "id" = $2
            "#,
        )
        .bind(website.is_valid)
        .bind(&website.id)
        .execute(&self.pool)
        .await
        .wrap_err("Failed to update website in database")?;

        Ok(())
    }
}
