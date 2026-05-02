use std::sync::LazyLock;

pub mod auth_tokens;
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
