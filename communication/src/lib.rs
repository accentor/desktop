use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Status {
    pub server_url: String,
    pub user_id: u64,
    pub playing_track_id: Option<u64>,
    pub playing_position_ms: Option<u64>,
}

#[tarpc::service]
pub trait Communication {
    async fn login(server_url: String, name: String, password: String) -> Result<(), String>;
    async fn status() -> Result<Status, String>;
    async fn play(track_id: u64) -> Result<(), String>;
}
