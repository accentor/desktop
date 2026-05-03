use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct Play {
    pub id: i64,
    pub played_at: String,
    pub track_id: i64,
    pub user_id: i64,
}
