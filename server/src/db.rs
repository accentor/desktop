use std::path::Path;
use std::str::FromStr;

use accentor_api::users::User;
use anyhow::Result;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};

#[derive(Clone)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        let opts = SqliteConnectOptions::from_str(&format!("sqlite://{}", path.display()))?
            .create_if_missing(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(opts)
            .await?;
        sqlx::migrate!("./migrations").run(&pool).await?;
        Ok(Self { pool })
    }

    pub async fn upsert_users(&self, users: &[User], loaded_at_ms: i64) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        for u in users {
            sqlx::query(
                "INSERT INTO users (id, name, permission, loaded_at) VALUES (?, ?, ?, ?)
                 ON CONFLICT(id) DO UPDATE SET
                     name       = excluded.name,
                     permission = excluded.permission,
                     loaded_at  = excluded.loaded_at",
            )
            .bind(u.id as i64)
            .bind(&u.name)
            .bind(u.permission.as_str())
            .bind(loaded_at_ms)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn prune_users(&self, started_at_ms: i64) -> Result<()> {
        sqlx::query("DELETE FROM users WHERE loaded_at < ?")
            .bind(started_at_ms)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn user_name(&self, user_id: u64) -> Result<Option<String>> {
        let row: Option<(String,)> = sqlx::query_as("SELECT name FROM users WHERE id = ?")
            .bind(user_id as i64)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(|(n,)| n))
    }
}
