use color_eyre::eyre::WrapErr;
use color_eyre::Result;
use sqlx::{Pool, Postgres};

#[derive(Debug, Clone)]
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

impl Website {
    pub fn is_stale(&self, is_valid: bool) -> bool {
        self.is_valid != is_valid
    }
}

impl Database {
    pub async fn get_websites(&self) -> Result<Vec<Website>> {
        sqlx::query_as!(
            Website,
            r#"
            SELECT "id", "url", "isUrlValid" as is_valid
            FROM "Website"
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .wrap_err("Failed to fetch websites from database")
    }

    pub async fn update_website_status(&self, id: &str, is_valid: bool) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE "Website"
            SET "isUrlValid" = $1
            WHERE "id" = $2
            "#,
            is_valid,
            id,
        )
        .execute(&self.pool)
        .await
        .wrap_err("Failed to update website in database")?;

        Ok(())
    }
}
