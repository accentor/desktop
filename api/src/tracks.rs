use anyhow::Error;
use serde::Deserialize;

use crate::endpoint;

#[derive(Clone, Debug, Deserialize, strum::AsRefStr)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum TrackArtistRole {
    Main,
    Performer,
    Composer,
    Conductor,
    Remixer,
    Producer,
    Arranger,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TrackArtist {
    pub artist_id: i64,
    pub role: TrackArtistRole,
    pub order: i64,
    pub name: String,
    pub normalized_name: String,
    pub hidden: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Track {
    pub id: i64,
    pub title: String,
    pub normalized_title: String,
    pub number: i64,
    pub album_id: i64,
    pub review_comment: Option<String>,
    pub genre_ids: Vec<i64>,
    pub codec_id: Option<i64>,
    pub length: Option<i64>,
    pub bitrate: Option<i64>,
    pub location_id: Option<i64>,
    pub audio_file_id: Option<i64>,
    pub track_artists: Vec<TrackArtist>,
    pub filename: Option<String>,
    pub sample_rate: Option<i64>,
    pub bit_depth: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
}

pub fn track_audio_url(server_url: &str, track_id: u64) -> Result<reqwest::Url, Error> {
    Ok(endpoint(server_url, &format!("tracks/{}/audio", track_id)).parse()?)
}
