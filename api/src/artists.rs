use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Artist {
    pub id: i64,
    pub name: String,
    pub normalized_name: String,
    pub review_comment: Option<String>,
    pub image: Option<String>,
    pub image100: Option<String>,
    pub image250: Option<String>,
    pub image500: Option<String>,
    pub image_type: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}
