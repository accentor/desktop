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

    #[allow(dead_code)]
    pub async fn artist(&self, id: u64) -> Result<Option<Artist>> {
        #[derive(sqlx::FromRow)]
        struct ArtistRow {
            id: i64,
            name: String,
            normalized_name: String,
            review_comment: Option<String>,
            image: Option<String>,
            image100: Option<String>,
            image250: Option<String>,
            image500: Option<String>,
            image_type: Option<String>,
            created_at: String,
            updated_at: String,
        }

        Ok(sqlx::query_as::<_, ArtistRow>(
            "SELECT id, name, normalized_name, review_comment, \
             image, image100, image250, image500, image_type, \
             created_at, updated_at FROM artists WHERE id = ?",
        )
        .bind(id as i64)
        .fetch_optional(&self.pool)
        .await?
        .map(|r| Artist {
            id: r.id,
            name: r.name,
            normalized_name: r.normalized_name,
            review_comment: r.review_comment,
            image: r.image,
            image100: r.image100,
            image250: r.image250,
            image500: r.image500,
            image_type: r.image_type,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }))
    }

    pub async fn prune_artists(&self, started_at_ms: i64) -> Result<()> {
        self.prune_simple("artists", started_at_ms).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn roundtrip() {
        let (db, _dir) = crate::db::fresh().await;
        let original = Artist {
            id: 9,
            name: "Miles Davis".into(),
            normalized_name: "miles davis".into(),
            review_comment: Some("New artist".into()),
            image: Some("img.jpg".into()),
            image100: Some("img100.jpg".into()),
            image250: Some("img250.jpg".into()),
            image500: Some("img500.jpg".into()),
            image_type: Some("image/jpeg".into()),
            created_at: "2024-01-01T00:00:00Z".into(),
            updated_at: "2024-06-01T00:00:00Z".into(),
        };
        db.upsert_artists(std::slice::from_ref(&original), 1_700_000_000_000)
            .await
            .unwrap();
        let read_back = db.artist(original.id as u64).await.unwrap().unwrap();
        assert_eq!(original, read_back);
    }

    #[tokio::test]
    async fn prune() {
        let (db, _dir) = crate::db::fresh().await;
        let artist = Artist {
            id: 1,
            name: "Miles Davis".into(),
            normalized_name: "miles davis".into(),
            review_comment: Some("legend".into()),
            image: Some("img.jpg".into()),
            image100: Some("img100.jpg".into()),
            image250: Some("img250.jpg".into()),
            image500: Some("img500.jpg".into()),
            image_type: Some("image/jpeg".into()),
            created_at: "2024-01-01T00:00:00Z".into(),
            updated_at: "2024-06-01T00:00:00Z".into(),
        };
        db.upsert_artists(std::slice::from_ref(&artist), 1_700_000_000_000)
            .await
            .unwrap();
        db.prune_artists(1_700_000_001_000).await.unwrap();
        assert_eq!(None, db.artist(artist.id as u64).await.unwrap())
    }
}
