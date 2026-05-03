use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct AlbumArtist {
    pub artist_id: i64,
    pub name: String,
    pub normalized_name: String,
    pub order: i64,
    pub separator: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AlbumLabel {
    pub label_id: i64,
    pub catalogue_number: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Album {
    pub id: i64,
    pub title: String,
    pub normalized_title: String,
    pub release: String,
    pub review_comment: Option<String>,
    pub edition: Option<String>,
    pub edition_description: Option<String>,
    pub album_artists: Vec<AlbumArtist>,
    pub album_labels: Vec<AlbumLabel>,
    pub image: Option<String>,
    pub image100: Option<String>,
    pub image250: Option<String>,
    pub image500: Option<String>,
    pub image_type: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}
