use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct Genre {
    pub id: i64,
    pub name: String,
    pub normalized_name: String,
}
