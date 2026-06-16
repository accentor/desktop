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

    #[allow(dead_code)]
    pub async fn auth_token(&self, id: u64) -> Result<Option<AuthToken>> {
        #[derive(sqlx::FromRow)]
        struct AuthTokenRow {
            id: i64,
            user_id: i64,
            user_agent: String,
            application: Option<String>,
        }

        Ok(sqlx::query_as::<_, AuthTokenRow>(
            "SELECT id, user_id, user_agent, application FROM auth_tokens WHERE id = ?",
        )
        .bind(id as i64)
        .fetch_optional(&self.pool)
        .await?
        .map(|r| AuthToken {
            id: r.id,
            user_id: r.user_id,
            user_agent: r.user_agent,
            application: r.application,
        }))
    }

    pub async fn prune_auth_tokens(&self, started_at_ms: i64) -> Result<()> {
        self.prune_simple("auth_tokens", started_at_ms).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn roundtrip() {
        let (db, _dir) = crate::db::fresh().await;
        let original = AuthToken {
            id: 11,
            user_id: 5,
            user_agent: "accentord/0.1".into(),
            application: Some("desktop".into()),
        };
        db.upsert_auth_tokens(std::slice::from_ref(&original), 1_700_000_000_000)
            .await
            .unwrap();
        let read_back = db.auth_token(original.id as u64).await.unwrap().unwrap();
        assert_eq!(original, read_back);
    }
}
