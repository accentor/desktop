use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Status {
    pub server_url: String,
    pub user_name: Option<String>,
    pub playing_track_id: Option<u64>,
    pub playing_position_ms: Option<u64>,
    pub sync_in_progress: bool,
    pub last_sync_at: Option<u64>,
    pub last_sync_error: Option<String>,
}

#[tarpc::service]
pub trait Communication {
    async fn login(server_url: String, name: String, password: String) -> Result<(), String>;
    async fn play(track_id: u64) -> Result<(), String>;
    async fn status() -> Result<Status, String>;
    async fn sync() -> Result<(), String>;
}
