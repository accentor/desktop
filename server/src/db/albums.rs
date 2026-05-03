use accentor_api::albums::Album;
use anyhow::Result;
use sqlx::QueryBuilder;

use super::Database;

impl Database {
    pub async fn upsert_albums(&self, items: &[Album], loaded_at_ms: i64) -> Result<()> {
        if items.is_empty() {
            return Ok(());
        }
        let mut tx = self.pool.begin().await?;

        for chunk in items.chunks(50) {
            let mut qb = QueryBuilder::new(
                "INSERT INTO albums \
                 (id, title, normalized_title, release, review_comment, \
                 edition, edition_description, \
                 image, image100, image250, image500, image_type, \
                 created_at, updated_at, loaded_at) ",
            );
            qb.push_values(chunk.iter(), |mut b, a| {
                b.push_bind(a.id)
                    .push_bind(&a.title)
                    .push_bind(&a.normalized_title)
                    .push_bind(&a.release)
                    .push_bind(&a.review_comment)
                    .push_bind(&a.edition)
                    .push_bind(&a.edition_description)
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
                 title = excluded.title, \
                 normalized_title = excluded.normalized_title, \
                 release = excluded.release, \
                 review_comment = excluded.review_comment, \
                 edition = excluded.edition, \
                 edition_description = excluded.edition_description, \
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

        let album_ids: Vec<i64> = items.iter().map(|a| a.id).collect();
        let all_artists: Vec<_> = items
            .iter()
            .flat_map(|a| a.album_artists.iter().map(move |aa| (a.id, aa)))
            .collect();
        let all_labels: Vec<_> = items
            .iter()
            .flat_map(|a| a.album_labels.iter().map(move |al| (a.id, al)))
            .collect();

        super::delete_children_by_parent_ids(&mut tx, "album_artists", "album_id", &album_ids)
            .await?;

        for chunk in all_artists.chunks(50) {
            let mut qb = QueryBuilder::new(
                "INSERT INTO album_artists \
                 (album_id, artist_id, name, normalized_name, \"order\", separator) ",
            );
            qb.push_values(chunk.iter().copied(), |mut b, (album_id, aa)| {
                b.push_bind(album_id)
                    .push_bind(aa.artist_id)
                    .push_bind(&aa.name)
                    .push_bind(&aa.normalized_name)
                    .push_bind(aa.order)
                    .push_bind(&aa.separator);
            });
            qb.build().execute(&mut *tx).await?;
        }

        super::delete_children_by_parent_ids(&mut tx, "album_labels", "album_id", &album_ids)
            .await?;

        for chunk in all_labels.chunks(50) {
            let mut qb = QueryBuilder::new(
                "INSERT INTO album_labels (album_id, label_id, catalogue_number) ",
            );
            qb.push_values(chunk.iter().copied(), |mut b, (album_id, al)| {
                b.push_bind(album_id)
                    .push_bind(al.label_id)
                    .push_bind(&al.catalogue_number);
            });
            qb.build().execute(&mut *tx).await?;
        }

        tx.commit().await?;
        Ok(())
    }

    pub async fn prune_albums(&self, started_at_ms: i64) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("DELETE FROM albums WHERE loaded_at < ?")
            .bind(started_at_ms)
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM album_artists WHERE album_id NOT IN (SELECT id FROM albums)")
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM album_labels WHERE album_id NOT IN (SELECT id FROM albums)")
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(())
    }
}
