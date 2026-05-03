use accentor_api::plays::Play;
use anyhow::Result;
use sqlx::QueryBuilder;

use super::Database;

impl Database {
    pub async fn upsert_plays(&self, items: &[Play], loaded_at_ms: i64) -> Result<()> {
        if items.is_empty() {
            return Ok(());
        }
        let mut tx = self.pool.begin().await?;
        for chunk in items.chunks(50) {
            let mut qb = QueryBuilder::new(
                "INSERT INTO plays (id, played_at, track_id, user_id, loaded_at) ",
            );
            qb.push_values(chunk.iter(), |mut b, p| {
                b.push_bind(p.id)
                    .push_bind(&p.played_at)
                    .push_bind(p.track_id)
                    .push_bind(p.user_id)
                    .push_bind(loaded_at_ms);
            });
            qb.push(
                " ON CONFLICT(id) DO UPDATE SET \
                 played_at = excluded.played_at, \
                 track_id = excluded.track_id, \
                 user_id = excluded.user_id, \
                 loaded_at = excluded.loaded_at",
            );
            qb.build().execute(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn prune_plays(&self, started_at_ms: i64) -> Result<()> {
        self.prune_simple("plays", started_at_ms).await
    }
}
