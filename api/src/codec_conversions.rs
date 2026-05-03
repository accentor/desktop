use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct CodecConversion {
    pub id: i64,
    pub name: String,
    pub ffmpeg_params: String,
    pub resulting_codec_id: i64,
}
