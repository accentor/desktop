use anyhow::{Error, bail};
use serde::{Deserialize, Serialize};

use crate::{CLIENT, USER_AGENT, endpoint};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthTokenWithToken {
    pub id: u64,
    pub user_id: u64,
    pub user_agent: String,
    pub token: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct AuthToken {
    pub id: i64,
    pub user_id: i64,
    pub user_agent: String,
    pub application: Option<String>,
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
