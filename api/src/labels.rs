use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Label {
    pub id: i64,
    pub name: String,
    pub normalized_name: String,
}
