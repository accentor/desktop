use accentor_api::genres::Genre;
use anyhow::Result;
use sqlx::QueryBuilder;

use super::Database;

impl Database {
    pub async fn upsert_genres(&self, genres: &[Genre], loaded_at_ms: i64) -> Result<()> {
        if genres.is_empty() {
            return Ok(());
        }
        let mut tx = self.pool.begin().await?;
        for chunk in genres.chunks(50) {
            let mut qb =
                QueryBuilder::new("INSERT INTO genres (id, name, normalized_name, loaded_at) ");
            qb.push_values(chunk.iter(), |mut b, g| {
                b.push_bind(g.id)
                    .push_bind(&g.name)
                    .push_bind(&g.normalized_name)
                    .push_bind(loaded_at_ms);
            });
            qb.push(
                " ON CONFLICT(id) DO UPDATE SET \
                 name = excluded.name, \
                 normalized_name = excluded.normalized_name, \
                 loaded_at = excluded.loaded_at",
            );
            qb.build().execute(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn genre(&self, id: u64) -> Result<Option<Genre>> {
        #[derive(sqlx::FromRow)]
        struct GenreRow {
            id: i64,
            name: String,
            normalized_name: String,
        }

        Ok(sqlx::query_as::<_, GenreRow>(
            "SELECT id, name, normalized_name FROM genres WHERE id = ?",
        )
        .bind(id as i64)
        .fetch_optional(&self.pool)
        .await?
        .map(|r| Genre {
            id: r.id,
            name: r.name,
            normalized_name: r.normalized_name,
        }))
    }

    pub async fn prune_genres(&self, started_at_ms: i64) -> Result<()> {
        self.prune_simple("genres", started_at_ms).await
    }
}
