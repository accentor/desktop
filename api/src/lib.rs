use std::sync::LazyLock;

use anyhow::{Error, bail};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthTokenWithToken {
    pub id: u64,
    pub user_id: u64,
    pub user_agent: String,
    pub token: String,
}

#[derive(Serialize)]
struct CreateAuthTokenBody<'a> {
    name: &'a str,
    password: &'a str,
    auth_token: CreateAuthTokenNested<'a>,
}

#[derive(Serialize)]
struct CreateAuthTokenNested<'a> {
    user_agent: &'a str,
}

const USER_AGENT: &str = concat!("accentord/", env!("CARGO_PKG_VERSION"));

static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .expect("failed to build reqwest client")
});

pub fn http_client() -> &'static reqwest::Client {
    &CLIENT
}

fn endpoint(server_url: &str, path: &str) -> String {
    format!("{}/api/{}", server_url.trim_end_matches('/'), path)
}

pub fn track_audio_url(server_url: &str, track_id: u64) -> Result<reqwest::Url, Error> {
    Ok(endpoint(server_url, &format!("tracks/{}/audio", track_id)).parse()?)
}

pub async fn create_auth_token(
    server_url: &str,
    name: &str,
    password: &str,
) -> Result<AuthTokenWithToken, Error> {
    let body = CreateAuthTokenBody {
        name,
        password,
        auth_token: CreateAuthTokenNested {
            user_agent: USER_AGENT,
        },
    };

    let response = CLIENT
        .post(endpoint(server_url, "auth_tokens"))
        .json(&body)
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        bail!("server returned {status}: {body}");
    }

    Ok(response.json().await?)
}
