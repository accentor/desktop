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

    pub async fn prune_codec_conversions(&self, started_at_ms: i64) -> Result<()> {
        self.prune_simple("codec_conversions", started_at_ms).await
    }
}
