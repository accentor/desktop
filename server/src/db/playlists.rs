use accentor_api::playlists::{Playlist, PlaylistAccess, PlaylistType};
use anyhow::Result;
use sqlx::QueryBuilder;

use super::Database;

impl Database {
    pub async fn upsert_playlists(&self, items: &[Playlist], loaded_at_ms: i64) -> Result<()> {
        if items.is_empty() {
            return Ok(());
        }
        let mut tx = self.pool.begin().await?;

        for chunk in items.chunks(50) {
            let mut qb = QueryBuilder::new(
                "INSERT INTO playlists \
                 (id, name, description, user_id, access, playlist_type, \
                 created_at, updated_at, loaded_at) ",
            );
            qb.push_values(chunk.iter(), |mut b, p| {
                b.push_bind(p.id)
                    .push_bind(&p.name)
                    .push_bind(&p.description)
                    .push_bind(p.user_id)
                    .push_bind(p.access.as_ref())
                    .push_bind(p.playlist_type.as_ref())
                    .push_bind(&p.created_at)
                    .push_bind(&p.updated_at)
                    .push_bind(loaded_at_ms);
            });
            qb.push(
                " ON CONFLICT(id) DO UPDATE SET \
                 name = excluded.name, \
                 description = excluded.description, \
                 user_id = excluded.user_id, \
                 access = excluded.access, \
                 playlist_type = excluded.playlist_type, \
                 created_at = excluded.created_at, \
                 updated_at = excluded.updated_at, \
                 loaded_at = excluded.loaded_at",
            );
            qb.build().execute(&mut *tx).await?;
        }

        let playlist_ids: Vec<i64> = items.iter().map(|p| p.id).collect();
        let all_items: Vec<_> = items
            .iter()
            .flat_map(|p| {
                p.item_ids
                    .iter()
                    .enumerate()
                    .map(move |(pos, &item_id)| (p.id, pos as i64, item_id))
            })
            .collect();

        super::delete_children_by_parent_ids(
            &mut tx,
            "playlist_items",
            "playlist_id",
            &playlist_ids,
        )
        .await?;

        for chunk in all_items.chunks(50) {
            let mut qb =
                QueryBuilder::new("INSERT INTO playlist_items (playlist_id, position, item_id) ");
            qb.push_values(
                chunk.iter().copied(),
                |mut b, (playlist_id, position, item_id)| {
                    b.push_bind(playlist_id)
                        .push_bind(position)
                        .push_bind(item_id);
                },
            );
            qb.build().execute(&mut *tx).await?;
        }

        tx.commit().await?;
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn playlist(&self, id: u64) -> Result<Option<Playlist>> {
        #[derive(sqlx::FromRow)]
        struct PlaylistRow {
            id: i64,
            name: String,
            description: Option<String>,
            user_id: i64,
            access: String,
            playlist_type: String,
            created_at: String,
            updated_at: String,
        }

        let Some(row) = sqlx::query_as::<_, PlaylistRow>(
            "SELECT id, name, description, user_id, access, playlist_type, \
             created_at, updated_at FROM playlists WHERE id = ?",
        )
        .bind(id as i64)
        .fetch_optional(&self.pool)
        .await?
        else {
            return Ok(None);
        };

        let item_id_rows: Vec<(i64,)> = sqlx::query_as(
            "SELECT item_id FROM playlist_items WHERE playlist_id = ? ORDER BY position",
        )
        .bind(id as i64)
        .fetch_all(&self.pool)
        .await?;

        Ok(Some(Playlist {
            id: row.id,
            name: row.name,
            description: row.description,
            user_id: row.user_id,
            access: row
                .access
                .parse::<PlaylistAccess>()
                .map_err(|e: strum::ParseError| anyhow::anyhow!("{e}"))?,
            playlist_type: row
                .playlist_type
                .parse::<PlaylistType>()
                .map_err(|e: strum::ParseError| anyhow::anyhow!("{e}"))?,
            item_ids: item_id_rows.into_iter().map(|(i,)| i).collect(),
            created_at: row.created_at,
            updated_at: row.updated_at,
        }))
    }

    pub async fn prune_playlists(&self, started_at_ms: i64) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("DELETE FROM playlists WHERE loaded_at < ?")
            .bind(started_at_ms)
            .execute(&mut *tx)
            .await?;
        sqlx::query(
            "DELETE FROM playlist_items WHERE playlist_id NOT IN (SELECT id FROM playlists)",
        )
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn roundtrip() {
        let (db, _dir) = crate::db::fresh().await;
        let original = Playlist {
            id: 50,
            name: "Favorite tracks".into(),
            description: Some("Listen on repeat".into()),
            user_id: 1,
            access: PlaylistAccess::Personal,
            playlist_type: PlaylistType::Track,
            item_ids: vec![300, 100, 200],
            created_at: "2024-01-01T00:00:00Z".into(),
            updated_at: "2024-06-01T00:00:00Z".into(),
        };
        db.upsert_playlists(std::slice::from_ref(&original), 1_700_000_000_000)
            .await
            .unwrap();
        let read_back = db.playlist(original.id as u64).await.unwrap().unwrap();
        assert_eq!(original, read_back);
    }
}
