use accentor_api::labels::Label;
use anyhow::Result;
use sqlx::QueryBuilder;

use super::Database;

impl Database {
    pub async fn upsert_labels(&self, labels: &[Label], loaded_at_ms: i64) -> Result<()> {
        if labels.is_empty() {
            return Ok(());
        }
        let mut tx = self.pool.begin().await?;
        for chunk in labels.chunks(50) {
            let mut qb =
                QueryBuilder::new("INSERT INTO labels (id, name, normalized_name, loaded_at) ");
            qb.push_values(chunk.iter(), |mut b, l| {
                b.push_bind(l.id)
                    .push_bind(&l.name)
                    .push_bind(&l.normalized_name)
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

    pub async fn prune_labels(&self, started_at_ms: i64) -> Result<()> {
        self.prune_simple("labels", started_at_ms).await
    }
}
