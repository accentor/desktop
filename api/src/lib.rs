use std::sync::LazyLock;

use anyhow::{Error, bail};
use serde::de::DeserializeOwned;

pub mod albums;
pub mod artists;
pub mod auth_tokens;
pub mod codec_conversions;
pub mod genres;
pub mod labels;
pub mod playlists;
pub mod plays;
pub mod tracks;
pub mod users;

pub(crate) const USER_AGENT: &str = concat!("accentord/", env!("CARGO_PKG_VERSION"));

pub(crate) static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .expect("failed to build reqwest client")
});

pub fn http_client() -> &'static reqwest::Client {
    &CLIENT
}

pub(crate) fn endpoint(server_url: &str, path: &str) -> String {
    format!("{}/api/{}", server_url.trim_end_matches('/'), path)
}

pub struct Page<T> {
    pub items: Vec<T>,
    pub total_pages: u32,
}

pub async fn fetch_page<T: DeserializeOwned>(
    server_url: &str,
    token: &str,
    path: &str,
    page: u32,
) -> Result<Page<T>, Error> {
    let response = CLIENT
        .get(format!("{}?page={}", endpoint(server_url, path), page))
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
        .ok_or_else(|| anyhow::anyhow!("missing x-total-pages header"))?;
    let items: Vec<T> = response.json().await?;
    Ok(Page { items, total_pages })
}
