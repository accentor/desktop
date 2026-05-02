use std::sync::{Arc, Mutex};

use accentor_api::{AuthTokenWithToken, create_auth_token};
use accentor_communication::{Communication, Status};
use accentor_utils::{get_data_path, get_socket_path};
use anyhow::Error;
use futures::{future, prelude::*};
use serde::{Deserialize, Serialize};
use tarpc::{
    context,
    server::{self, Channel},
    tokio_serde::formats::Json,
};
use tokio::signal::unix::{SignalKind, signal};

const AUTH_TOKEN_FILENAME: &str = "auth_token.json";

#[derive(Clone, Debug, Serialize, Deserialize)]
struct StoredToken {
    server_url: String,
    token: AuthTokenWithToken,
}

#[derive(Clone)]
struct AccentorServer {
    state: Arc<Mutex<Option<StoredToken>>>,
}

impl Communication for AccentorServer {
    async fn login(
        self,
        _: context::Context,
        server_url: String,
        name: String,
        password: String,
    ) -> Result<(), String> {
        let auth_token = create_auth_token(&server_url, &name, &password)
            .await
            .map_err(|e| e.to_string())?;

        let stored = StoredToken {
            server_url,
            token: auth_token,
        };

        let path = get_data_path().join(AUTH_TOKEN_FILENAME);
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| e.to_string())?;
        }
        let json = serde_json::to_vec_pretty(&stored).map_err(|e| e.to_string())?;
        tokio::fs::write(&path, json)
            .await
            .map_err(|e| e.to_string())?;

        *self.state.lock().unwrap() = Some(stored);

        Ok(())
    }

    async fn status(self, _: context::Context) -> Result<Status, String> {
        match &*self.state.lock().unwrap() {
            Some(stored) => Ok(Status {
                server_url: stored.server_url.clone(),
                user_id: stored.token.user_id,
            }),
            None => Err("Not logged in".to_string()),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let socket_path = get_socket_path();
    let token_path = get_data_path().join(AUTH_TOKEN_FILENAME);

    let initial = match tokio::fs::read(&token_path).await {
        Ok(bytes) => Some(serde_json::from_slice::<StoredToken>(&bytes)?),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => return Err(e.into()),
    };
    let state = Arc::new(Mutex::new(initial));

    if let Some(parent) = socket_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    match std::fs::remove_file(&socket_path) {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(e.into()),
    }

    let mut listener = tarpc::serde_transport::unix::listen(&socket_path, Json::default).await?;
    listener.config_mut().max_frame_length(usize::MAX);

    let server = listener
        .filter_map(|r| future::ready(r.ok()))
        .map(server::BaseChannel::with_defaults)
        .map(|channel| {
            let server = AccentorServer {
                state: state.clone(),
            };
            channel.execute(server.serve()).for_each(|f| async move {
                tokio::spawn(f);
            })
        })
        .buffer_unordered(10)
        .for_each(|_| async {});

    let mut sigterm = signal(SignalKind::terminate())?;
    let mut sigint = signal(SignalKind::interrupt())?;

    tokio::select! {
        _ = server => {}
        _ = sigterm.recv() => {}
        _ = sigint.recv() => {}
    }

    let _ = std::fs::remove_file(&socket_path);
    Ok(())
}
