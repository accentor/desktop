use accentor_api::codec_conversions::CodecConversion;
use anyhow::Result;
use sqlx::QueryBuilder;

use super::Database;

impl Database {
    pub async fn upsert_codec_conversions(
        &self,
        items: &[CodecConversion],
        loaded_at_ms: i64,
    ) -> Result<()> {
        if items.is_empty() {
            return Ok(());
        }
        let mut tx = self.pool.begin().await?;
        for chunk in items.chunks(50) {
            let mut qb = QueryBuilder::new(
                "INSERT INTO codec_conversions \
                 (id, name, ffmpeg_params, resulting_codec_id, loaded_at) ",
            );
            qb.push_values(chunk.iter(), |mut b, c| {
                b.push_bind(c.id)
                    .push_bind(&c.name)
                    .push_bind(&c.ffmpeg_params)
                    .push_bind(c.resulting_codec_id)
                    .push_bind(loaded_at_ms);
            });
            qb.push(
                " ON CONFLICT(id) DO UPDATE SET \
                 name = excluded.name, \
                 ffmpeg_params = excluded.ffmpeg_params, \
                 resulting_codec_id = excluded.resulting_codec_id, \
                 loaded_at = excluded.loaded_at",
            );
            qb.build().execute(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn codec_conversion(&self, id: u64) -> Result<Option<CodecConversion>> {
        #[derive(sqlx::FromRow)]
        struct CodecConversionRow {
            id: i64,
            name: String,
            ffmpeg_params: String,
            resulting_codec_id: i64,
        }

        Ok(sqlx::query_as::<_, CodecConversionRow>(
            "SELECT id, name, ffmpeg_params, resulting_codec_id \
             FROM codec_conversions WHERE id = ?",
        )
        .bind(id as i64)
        .fetch_optional(&self.pool)
        .await?
        .map(|r| CodecConversion {
            id: r.id,
            name: r.name,
            ffmpeg_params: r.ffmpeg_params,
            resulting_codec_id: r.resulting_codec_id,
        }))
    }

    pub async fn prune_codec_conversions(&self, started_at_ms: i64) -> Result<()> {
        self.prune_simple("codec_conversions", started_at_ms).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn roundtrip() {
        let (db, _dir) = crate::db::fresh().await;
        let original = CodecConversion {
            id: 3,
            name: "MP3 (320)".into(),
            ffmpeg_params: "-acodec mp3 -b:a 320k".into(),
            resulting_codec_id: 1,
        };
        db.upsert_codec_conversions(std::slice::from_ref(&original), 1_700_000_000_000)
            .await
            .unwrap();
        let read_back = db
            .codec_conversion(original.id as u64)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(original, read_back);
    }
}
