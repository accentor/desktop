use anyhow::Error;

use crate::endpoint;

pub fn track_audio_url(server_url: &str, track_id: u64) -> Result<reqwest::Url, Error> {
    Ok(endpoint(server_url, &format!("tracks/{}/audio", track_id)).parse()?)
}
