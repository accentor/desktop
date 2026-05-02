use std::str::FromStr;

use anyhow::{Error, bail};
use serde::{Deserialize, Serialize};

use crate::{CLIENT, endpoint};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Permission {
    User,
    Moderator,
    Admin,
}

impl Permission {
    pub fn as_str(self) -> &'static str {
        match self {
            Permission::User => "user",
            Permission::Moderator => "moderator",
            Permission::Admin => "admin",
        }
    }
}

impl FromStr for Permission {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "user" => Ok(Permission::User),
            "moderator" => Ok(Permission::Moderator),
            "admin" => Ok(Permission::Admin),
            _ => bail!("unknown permission: {s}"),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    pub id: u64,
    pub name: String,
    pub permission: Permission,
}

pub struct UsersPage {
    pub users: Vec<User>,
    pub total_pages: u32,
}

pub async fn fetch_users_page(
    server_url: &str,
    token: &str,
    page: u32,
) -> Result<UsersPage, Error> {
    let response = CLIENT
        .get(format!("{}?page={}", endpoint(server_url, "users"), page))
        .bearer_auth(token)
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        bail!("server returned {status}: {body}");
    }

    let total_pages: u32 = response
        .headers()
        .get("x-total-pages")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);
    let users: Vec<User> = response.json().await?;
    Ok(UsersPage { users, total_pages })
}
