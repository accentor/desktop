use std::path::Path;
use std::str::FromStr;

use anyhow::Result;
use sqlx::sqlite::{
    SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqlitePoolOptions, SqliteSynchronous,
};
use sqlx::{AssertSqlSafe, QueryBuilder};

mod albums;
mod artists;
mod auth_tokens;
mod codec_conversions;
mod genres;
mod labels;
mod playlists;
mod plays;
mod tracks;
mod users;

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
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Normal);
        let pool = SqlitePoolOptions::new()
            .max_connections(4)
            .connect_with(opts)
            .await?;
        sqlx::migrate!("./migrations").run(&pool).await?;
        Ok(Self { pool })
    }

    async fn prune_simple(&self, table: &str, started_at_ms: i64) -> Result<()> {
        sqlx::query(AssertSqlSafe(format!(
            "DELETE FROM {table} WHERE loaded_at < ?"
        )))
        .bind(started_at_ms)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

async fn delete_children_by_parent_ids(
    tx: &mut sqlx::SqliteConnection,
    child_table: &str,
    fk_col: &str,
    parent_ids: &[i64],
) -> Result<()> {
    for chunk in parent_ids.chunks(500) {
        let mut qb = QueryBuilder::new(format!("DELETE FROM {child_table} WHERE {fk_col} IN ("));
        let mut sep = qb.separated(", ");
        for id in chunk {
            sep.push_bind(*id);
        }
        qb.push(")");
        qb.build().execute(&mut *tx).await?;
    }
    Ok(())
}

#[cfg(test)]
pub(super) async fn fresh() -> (Database, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.db");
    let db = Database::open(&path).await.unwrap();
    (db, dir)
}
