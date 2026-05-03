use accentor_api::tracks::Track;
use anyhow::Result;
use sqlx::QueryBuilder;

use super::Database;

impl Database {
    pub async fn upsert_tracks(&self, items: &[Track], loaded_at_ms: i64) -> Result<()> {
        if items.is_empty() {
            return Ok(());
        }
        let mut tx = self.pool.begin().await?;

        for chunk in items.chunks(50) {
            let mut qb = QueryBuilder::new(
                "INSERT INTO tracks \
                 (id, title, normalized_title, number, album_id, \
                 review_comment, codec_id, length, bitrate, location_id, audio_file_id, \
                 filename, sample_rate, bit_depth, created_at, updated_at, loaded_at) ",
            );
            qb.push_values(chunk.iter(), |mut b, t| {
                b.push_bind(t.id)
                    .push_bind(&t.title)
                    .push_bind(&t.normalized_title)
                    .push_bind(t.number)
                    .push_bind(t.album_id)
                    .push_bind(&t.review_comment)
                    .push_bind(t.codec_id)
                    .push_bind(t.length)
                    .push_bind(t.bitrate)
                    .push_bind(t.location_id)
                    .push_bind(t.audio_file_id)
                    .push_bind(&t.filename)
                    .push_bind(t.sample_rate)
                    .push_bind(t.bit_depth)
                    .push_bind(&t.created_at)
                    .push_bind(&t.updated_at)
                    .push_bind(loaded_at_ms);
            });
            qb.push(
                " ON CONFLICT(id) DO UPDATE SET \
                 title = excluded.title, \
                 normalized_title = excluded.normalized_title, \
                 number = excluded.number, \
                 album_id = excluded.album_id, \
                 review_comment = excluded.review_comment, \
                 codec_id = excluded.codec_id, \
                 length = excluded.length, \
                 bitrate = excluded.bitrate, \
                 location_id = excluded.location_id, \
                 audio_file_id = excluded.audio_file_id, \
                 filename = excluded.filename, \
                 sample_rate = excluded.sample_rate, \
                 bit_depth = excluded.bit_depth, \
                 created_at = excluded.created_at, \
                 updated_at = excluded.updated_at, \
                 loaded_at = excluded.loaded_at",
            );
            qb.build().execute(&mut *tx).await?;
        }

        let track_ids: Vec<i64> = items.iter().map(|t| t.id).collect();
        let all_track_artists: Vec<_> = items
            .iter()
            .flat_map(|t| t.track_artists.iter().map(move |ta| (t.id, ta)))
            .collect();
        let all_genre_ids: Vec<_> = items
            .iter()
            .flat_map(|t| t.genre_ids.iter().map(move |&g| (t.id, g)))
            .collect();

        super::delete_children_by_parent_ids(&mut tx, "track_artists", "track_id", &track_ids)
            .await?;

        for chunk in all_track_artists.chunks(50) {
            let mut qb = QueryBuilder::new(
                "INSERT INTO track_artists \
                 (track_id, artist_id, role, \"order\", name, normalized_name, hidden) ",
            );
            qb.push_values(chunk.iter().copied(), |mut b, (track_id, ta)| {
                b.push_bind(track_id)
                    .push_bind(ta.artist_id)
                    .push_bind(ta.role.as_ref())
                    .push_bind(ta.order)
                    .push_bind(&ta.name)
                    .push_bind(&ta.normalized_name)
                    .push_bind(ta.hidden);
            });
            qb.build().execute(&mut *tx).await?;
        }

        super::delete_children_by_parent_ids(&mut tx, "track_genres", "track_id", &track_ids)
            .await?;

        for chunk in all_genre_ids.chunks(50) {
            let mut qb = QueryBuilder::new("INSERT INTO track_genres (track_id, genre_id) ");
            qb.push_values(chunk.iter().copied(), |mut b, (track_id, genre_id)| {
                b.push_bind(track_id).push_bind(genre_id);
            });
            qb.build().execute(&mut *tx).await?;
        }

        tx.commit().await?;
        Ok(())
    }

    pub async fn prune_tracks(&self, started_at_ms: i64) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("DELETE FROM tracks WHERE loaded_at < ?")
            .bind(started_at_ms)
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM track_artists WHERE track_id NOT IN (SELECT id FROM tracks)")
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM track_genres WHERE track_id NOT IN (SELECT id FROM tracks)")
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(())
    }
}
