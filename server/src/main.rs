mod db;
mod playback;

use std::future::Future;
use std::sync::{Arc, Mutex};

use accentor_api::{Page, fetch_page};
use accentor_api::auth_tokens::{AuthTokenWithToken, create_auth_token};
use accentor_communication::{Communication, Status};
use accentor_utils::{get_data_path, get_socket_path, unix_ms};
use anyhow::Error;
use futures::{future, prelude::*};
use playback::Playback;
use serde::{Deserialize, Serialize};
use tarpc::{
    context,
    server::{self, Channel},
    tokio_serde::formats::Json,
};
use tokio::signal::unix::{SignalKind, signal};

const AUTH_TOKEN_FILENAME: &str = "auth_token.json";

async fn sync_resource<T, FF, UF, PF>(
    db: db::Database,
    started_at_ms: i64,
    fetch: impl Fn(u32) -> FF,
    upsert: impl Fn(db::Database, Vec<T>, i64) -> UF,
    prune: impl FnOnce(db::Database, i64) -> PF,
) -> anyhow::Result<()>
where
    FF: Future<Output = anyhow::Result<Page<T>>>,
    UF: Future<Output = anyhow::Result<()>>,
    PF: Future<Output = anyhow::Result<()>>,
{
    let first = fetch(1).await?;
    upsert(db.clone(), first.items, unix_ms()).await?;
    for page in 2..=first.total_pages {
        let p = fetch(page).await?;
        upsert(db.clone(), p.items, unix_ms()).await?;
    }
    prune(db, started_at_ms).await
}

#[derive(Default)]
struct SyncState {
    in_progress: bool,
    last_completed_at: Option<u64>,
    last_error: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct StoredToken {
    server_url: String,
    token: AuthTokenWithToken,
}

#[derive(Clone)]
struct AccentorServer {
    state: Arc<Mutex<Option<StoredToken>>>,
    playback: Playback,
    db: db::Database,
    sync_state: Arc<Mutex<SyncState>>,
}

impl Communication for AccentorServer {
    async fn login(
        self,
        context: context::Context,
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

        self.sync(context).await?;

        Ok(())
    }

    async fn play(self, _: context::Context, track_id: u64) -> Result<(), String> {
        let (server_url, token) = {
            let guard = self.state.lock().unwrap();
            let stored = guard.as_ref().ok_or("Not logged in")?;
            (stored.server_url.clone(), stored.token.token.clone())
        };
        self.playback
            .play(&server_url, token, track_id)
            .await
            .map_err(|e| e.to_string())
    }

    async fn status(self, _: context::Context) -> Result<Status, String> {
        let (server_url, user_id) = {
            let guard = self.state.lock().unwrap();
            let stored = guard.as_ref().ok_or("Not logged in")?;
            (stored.server_url.clone(), stored.token.user_id)
        };
        let user_name = self
            .db
            .user_name(user_id)
            .await
            .map_err(|e| e.to_string())?;
        let playing = self.playback.current_playing();
        let sync = self.sync_state.lock().unwrap();
        Ok(Status {
            server_url,
            user_name,
            playing_track_id: playing.map(|(id, _)| id),
            playing_position_ms: playing.map(|(_, pos)| pos.as_millis() as u64),
            sync_in_progress: sync.in_progress,
            last_sync_at: sync.last_completed_at,
            last_sync_error: sync.last_error.clone(),
        })
    }

    async fn sync(self, _: context::Context) -> Result<(), String> {
        let (server_url, token) = {
            let guard = self.state.lock().unwrap();
            let stored = guard.as_ref().ok_or("Not logged in")?;
            (stored.server_url.clone(), stored.token.token.clone())
        };

        {
            let mut s = self.sync_state.lock().unwrap();
            if s.in_progress {
                return Err("Sync already in progress".to_string());
            }
            s.in_progress = true;
        }

        let db = self.db.clone();
        let sync_state = self.sync_state.clone();
        tokio::spawn(async move {
            let started_at_ms = unix_ms();
            let result = tokio::try_join!(
                sync_resource(
                    db.clone(), started_at_ms,
                    |p| fetch_page(&server_url, &token, "users", p),
                    |db, items, ts| async move { db.upsert_users(&items, ts).await },
                    |db, ts| async move { db.prune_users(ts).await },
                ),
                sync_resource(
                    db.clone(), started_at_ms,
                    |p| fetch_page(&server_url, &token, "artists", p),
                    |db, items, ts| async move { db.upsert_artists(&items, ts).await },
                    |db, ts| async move { db.prune_artists(ts).await },
                ),
                sync_resource(
                    db.clone(), started_at_ms,
                    |p| fetch_page(&server_url, &token, "genres", p),
                    |db, items, ts| async move { db.upsert_genres(&items, ts).await },
                    |db, ts| async move { db.prune_genres(ts).await },
                ),
                sync_resource(
                    db.clone(), started_at_ms,
                    |p| fetch_page(&server_url, &token, "labels", p),
                    |db, items, ts| async move { db.upsert_labels(&items, ts).await },
                    |db, ts| async move { db.prune_labels(ts).await },
                ),
                sync_resource(
                    db.clone(), started_at_ms,
                    |p| fetch_page(&server_url, &token, "codec_conversions", p),
                    |db, items, ts| async move { db.upsert_codec_conversions(&items, ts).await },
                    |db, ts| async move { db.prune_codec_conversions(ts).await },
                ),
                sync_resource(
                    db.clone(), started_at_ms,
                    |p| fetch_page(&server_url, &token, "auth_tokens", p),
                    |db, items, ts| async move { db.upsert_auth_tokens(&items, ts).await },
                    |db, ts| async move { db.prune_auth_tokens(ts).await },
                ),
                sync_resource(
                    db.clone(), started_at_ms,
                    |p| fetch_page(&server_url, &token, "albums", p),
                    |db, items, ts| async move { db.upsert_albums(&items, ts).await },
                    |db, ts| async move { db.prune_albums(ts).await },
                ),
                sync_resource(
                    db.clone(), started_at_ms,
                    |p| fetch_page(&server_url, &token, "tracks", p),
                    |db, items, ts| async move { db.upsert_tracks(&items, ts).await },
                    |db, ts| async move { db.prune_tracks(ts).await },
                ),
                sync_resource(
                    db.clone(), started_at_ms,
                    |p| fetch_page(&server_url, &token, "plays", p),
                    |db, items, ts| async move { db.upsert_plays(&items, ts).await },
                    |db, ts| async move { db.prune_plays(ts).await },
                ),
                sync_resource(
                    db.clone(), started_at_ms,
                    |p| fetch_page(&server_url, &token, "playlists", p),
                    |db, items, ts| async move { db.upsert_playlists(&items, ts).await },
                    |db, ts| async move { db.prune_playlists(ts).await },
                ),
            )
            .map(|_| ());

            let mut s = sync_state.lock().unwrap();
            s.in_progress = false;
            match result {
                Ok(()) => {
                    s.last_completed_at = Some(unix_ms() as u64 / 1000);
                    s.last_error = None;
                }
                Err(e) => {
                    eprintln!("sync failed: {e:#}");
                    s.last_error = Some(format!("{e:#}"));
                }
            }
        });

        Ok(())
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
    let playback = Playback::spawn()?;
    let db = db::Database::open(&get_data_path().join("accentor.db")).await?;
    let sync_state = Arc::new(Mutex::new(SyncState::default()));

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
                playback: playback.clone(),
                db: db.clone(),
                sync_state: sync_state.clone(),
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
