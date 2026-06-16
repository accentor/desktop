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

    #[allow(dead_code)]
    pub async fn play(&self, id: u64) -> Result<Option<Play>> {
        #[derive(sqlx::FromRow)]
        struct PlayRow {
            id: i64,
            played_at: String,
            track_id: i64,
            user_id: i64,
        }

        Ok(sqlx::query_as::<_, PlayRow>(
            "SELECT id, played_at, track_id, user_id FROM plays WHERE id = ?",
        )
        .bind(id as i64)
        .fetch_optional(&self.pool)
        .await?
        .map(|r| Play {
            id: r.id,
            played_at: r.played_at,
            track_id: r.track_id,
            user_id: r.user_id,
        }))
    }

    pub async fn prune_plays(&self, started_at_ms: i64) -> Result<()> {
        self.prune_simple("plays", started_at_ms).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn roundtrip() {
        let (db, _dir) = crate::db::fresh().await;
        let original = Play {
            id: 101,
            played_at: "2024-08-15T12:34:56Z".into(),
            track_id: 1234,
            user_id: 1,
        };
        db.upsert_plays(std::slice::from_ref(&original), 1_700_000_000_000)
            .await
            .unwrap();
        let read_back = db.play(original.id as u64).await.unwrap().unwrap();
        assert_eq!(original, read_back);
    }
}
