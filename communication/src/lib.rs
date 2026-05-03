use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrackSummary {
    pub id: u64,
    pub number: i64,
    pub title: String,
    pub album: String,
    pub artists: String,
    pub genres: String,
    pub length: Option<i64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrackPage {
    pub items: Vec<TrackSummary>,
    pub total_pages: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayingTrack {
    pub id: u64,
    pub position_ms: u64,
    pub title: String,
    pub album: String,
    pub artists: String,
    pub length: Option<i64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Status {
    pub server_url: String,
    pub user_name: Option<String>,
    pub playing_track: Option<PlayingTrack>,
    pub sync_in_progress: bool,
    pub last_sync_at: Option<u64>,
    pub last_sync_error: Option<String>,
}

#[tarpc::service]
pub trait Communication {
    async fn login(server_url: String, name: String, password: String) -> Result<(), String>;
    async fn play(track_id: u64) -> Result<PlayingTrack, String>;
    async fn status() -> Result<Status, String>;
    async fn sync() -> Result<(), String>;
    async fn track(track_id: u64) -> Result<Option<TrackSummary>, String>;
    async fn tracks(page: u32, per_page: u32) -> Result<TrackPage, String>;
}
