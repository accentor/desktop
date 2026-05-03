use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, strum::AsRefStr)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum PlaylistType {
    Album,
    Artist,
    Track,
}

#[derive(Clone, Debug, Deserialize, strum::AsRefStr)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum PlaylistAccess {
    Shared,
    Personal,
    Secret,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Playlist {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub user_id: i64,
    pub access: PlaylistAccess,
    pub playlist_type: PlaylistType,
    pub item_ids: Vec<i64>,
    pub created_at: String,
    pub updated_at: String,
}
