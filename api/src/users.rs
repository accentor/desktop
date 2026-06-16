use serde::{Deserialize, Serialize};

#[derive(
    Clone, Copy, Debug, Serialize, Deserialize, PartialEq, strum::AsRefStr, strum::EnumString,
)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum Permission {
    User,
    Moderator,
    Admin,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct User {
    pub id: u64,
    pub name: String,
    pub permission: Permission,
}
