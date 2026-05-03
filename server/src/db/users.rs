use accentor_api::users::User;
use anyhow::Result;
use sqlx::QueryBuilder;

use super::Database;

impl Database {
    pub async fn upsert_users(&self, users: &[User], loaded_at_ms: i64) -> Result<()> {
        if users.is_empty() {
            return Ok(());
        }
        let mut tx = self.pool.begin().await?;
        for chunk in users.chunks(50) {
            let mut qb = QueryBuilder::new("INSERT INTO users (id, name, permission, loaded_at) ");
            qb.push_values(chunk.iter(), |mut b, u| {
                b.push_bind(u.id as i64)
                    .push_bind(&u.name)
                    .push_bind(u.permission.as_ref())
                    .push_bind(loaded_at_ms);
            });
            qb.push(
                " ON CONFLICT(id) DO UPDATE SET \
                 name = excluded.name, \
                 permission = excluded.permission, \
                 loaded_at = excluded.loaded_at",
            );
            qb.build().execute(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn prune_users(&self, started_at_ms: i64) -> Result<()> {
        self.prune_simple("users", started_at_ms).await
    }

    pub async fn user(&self, user_id: u64) -> Result<Option<User>> {
        #[derive(sqlx::FromRow)]
        struct UserRow {
            id: i64,
            name: String,
            permission: String,
        }
        let Some(row) =
            sqlx::query_as::<_, UserRow>("SELECT id, name, permission FROM users WHERE id = ?")
                .bind(user_id as i64)
                .fetch_optional(&self.pool)
                .await?
        else {
            return Ok(None);
        };
        Ok(Some(User {
            id: row.id as u64,
            name: row.name,
            permission: row
                .permission
                .parse()
                .map_err(|e: strum::ParseError| anyhow::anyhow!("{e}"))?,
        }))
    }
}
