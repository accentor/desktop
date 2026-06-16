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

    #[allow(dead_code)]
    pub async fn label(&self, id: u64) -> Result<Option<Label>> {
        #[derive(sqlx::FromRow)]
        struct LabelRow {
            id: i64,
            name: String,
            normalized_name: String,
        }

        Ok(sqlx::query_as::<_, LabelRow>(
            "SELECT id, name, normalized_name FROM labels WHERE id = ?",
        )
        .bind(id as i64)
        .fetch_optional(&self.pool)
        .await?
        .map(|r| Label {
            id: r.id,
            name: r.name,
            normalized_name: r.normalized_name,
        }))
    }

    pub async fn prune_labels(&self, started_at_ms: i64) -> Result<()> {
        self.prune_simple("labels", started_at_ms).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn roundtrip() {
        let (db, _dir) = crate::db::fresh().await;
        let original = Label {
            id: 42,
            name: "Blue Note".into(),
            normalized_name: "blue note".into(),
        };
        db.upsert_labels(std::slice::from_ref(&original), 1_700_000_000_000)
            .await
            .unwrap();
        let read_back = db.label(original.id as u64).await.unwrap().unwrap();
        assert_eq!(original, read_back);
    }
}
