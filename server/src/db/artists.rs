use accentor_api::artists::Artist;
use anyhow::Result;
use sqlx::QueryBuilder;

use super::Database;

impl Database {
    pub async fn upsert_artists(&self, artists: &[Artist], loaded_at_ms: i64) -> Result<()> {
        if artists.is_empty() {
            return Ok(());
        }
        let mut tx = self.pool.begin().await?;
        for chunk in artists.chunks(50) {
            let mut qb = QueryBuilder::new(
                "INSERT INTO artists \
                 (id, name, normalized_name, review_comment, \
                 image, image100, image250, image500, image_type, \
                 created_at, updated_at, loaded_at) ",
            );
            qb.push_values(chunk.iter(), |mut b, a| {
                b.push_bind(a.id)
                    .push_bind(&a.name)
                    .push_bind(&a.normalized_name)
                    .push_bind(&a.review_comment)
                    .push_bind(&a.image)
                    .push_bind(&a.image100)
                    .push_bind(&a.image250)
                    .push_bind(&a.image500)
                    .push_bind(&a.image_type)
                    .push_bind(&a.created_at)
                    .push_bind(&a.updated_at)
                    .push_bind(loaded_at_ms);
            });
            qb.push(
                " ON CONFLICT(id) DO UPDATE SET \
                 name = excluded.name, \
                 normalized_name = excluded.normalized_name, \
                 review_comment = excluded.review_comment, \
                 image = excluded.image, \
                 image100 = excluded.image100, \
                 image250 = excluded.image250, \
                 image500 = excluded.image500, \
                 image_type = excluded.image_type, \
                 created_at = excluded.created_at, \
                 updated_at = excluded.updated_at, \
                 loaded_at = excluded.loaded_at",
            );
            qb.build().execute(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn prune_artists(&self, started_at_ms: i64) -> Result<()> {
        self.prune_simple("artists", started_at_ms).await
    }
}
