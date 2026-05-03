use accentor_api::auth_tokens::AuthToken;
use anyhow::Result;
use sqlx::QueryBuilder;

use super::Database;

impl Database {
    pub async fn upsert_auth_tokens(&self, items: &[AuthToken], loaded_at_ms: i64) -> Result<()> {
        if items.is_empty() {
            return Ok(());
        }
        let mut tx = self.pool.begin().await?;
        for chunk in items.chunks(50) {
            let mut qb = QueryBuilder::new(
                "INSERT INTO auth_tokens (id, user_id, user_agent, application, loaded_at) ",
            );
            qb.push_values(chunk.iter(), |mut b, a| {
                b.push_bind(a.id)
                    .push_bind(a.user_id)
                    .push_bind(&a.user_agent)
                    .push_bind(&a.application)
                    .push_bind(loaded_at_ms);
            });
            qb.push(
                " ON CONFLICT(id) DO UPDATE SET \
                 user_id = excluded.user_id, \
                 user_agent = excluded.user_agent, \
                 application = excluded.application, \
                 loaded_at = excluded.loaded_at",
            );
            qb.build().execute(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn prune_auth_tokens(&self, started_at_ms: i64) -> Result<()> {
        self.prune_simple("auth_tokens", started_at_ms).await
    }
}
