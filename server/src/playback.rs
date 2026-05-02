use std::sync::mpsc::{self, TrySendError};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use anyhow::{Error, anyhow};
use rodio::{DeviceSinkBuilder, MixerDeviceSink, Player, play};
use stream_download::http::{HttpStream, RANGE_HEADER_KEY, format_range_header_bytes};
use stream_download::storage::temp::TempStorageProvider;
use stream_download::{Settings, StreamDownload};

type AudioReader = StreamDownload<TempStorageProvider>;

#[derive(Clone)]
struct AuthClient {
    token: String,
}

impl stream_download::http::Client for AuthClient {
    type Url = reqwest::Url;
    type Headers = reqwest::header::HeaderMap;
    type Response = reqwest::Response;
    type Error = reqwest::Error;

    fn create() -> Self {
        unreachable!()
    }

    async fn get(&self, url: &Self::Url) -> Result<Self::Response, Self::Error> {
        self.request(url).send().await
    }

    async fn get_range(
        &self,
        url: &Self::Url,
        start: u64,
        end: Option<u64>,
    ) -> Result<Self::Response, Self::Error> {
        self.request(url)
            .header(RANGE_HEADER_KEY, format_range_header_bytes(start, end))
            .send()
            .await
    }
}

impl AuthClient {
    fn request(&self, url: &reqwest::Url) -> reqwest::RequestBuilder {
        accentor_api::http_client()
            .get(url.clone())
            .bearer_auth(&self.token)
    }
}

struct NowPlaying {
    track_id: u64,
    player: Player,
}

enum PlaybackCmd {
    Play(AudioReader, u64),
}

#[derive(Clone)]
pub struct Playback {
    tx: mpsc::SyncSender<PlaybackCmd>,
    now_playing: Arc<Mutex<Option<NowPlaying>>>,
}

impl Playback {
    pub fn spawn() -> Result<Self, Error> {
        let (tx, rx) = mpsc::sync_channel::<PlaybackCmd>(1);
        let now_playing = Arc::new(Mutex::new(None::<NowPlaying>));
        let now_playing_thread = now_playing.clone();

        let mut sink: MixerDeviceSink = DeviceSinkBuilder::open_default_sink()?;
        sink.log_on_drop(false);

        thread::Builder::new()
            .name("accentord-audio".to_string())
            .spawn(move || {
                let mut playing = false;
                loop {
                    let cmd = if playing {
                        match rx.recv_timeout(Duration::from_millis(500)) {
                            Ok(cmd) => Some(cmd),
                            Err(mpsc::RecvTimeoutError::Timeout) => None,
                            Err(mpsc::RecvTimeoutError::Disconnected) => break,
                        }
                    } else {
                        match rx.recv() {
                            Ok(cmd) => Some(cmd),
                            Err(_) => break,
                        }
                    };

                    match cmd {
                        Some(PlaybackCmd::Play(reader, track_id)) => {
                            {
                                let mut guard = now_playing_thread.lock().unwrap();
                                if let Some(prev) = guard.take() {
                                    prev.player.stop();
                                }
                            }
                            match play(sink.mixer(), reader) {
                                Ok(player) => {
                                    *now_playing_thread.lock().unwrap() =
                                        Some(NowPlaying { track_id, player });
                                    playing = true;
                                }
                                Err(e) => {
                                    eprintln!("decode error: {e}");
                                    playing = false;
                                }
                            }
                        }
                        None => {
                            let mut guard = now_playing_thread.lock().unwrap();
                            if guard.as_ref().map(|s| s.player.empty()).unwrap_or(false) {
                                *guard = None;
                                playing = false;
                            }
                        }
                    }
                }
            })?;

        Ok(Self { tx, now_playing })
    }

    pub fn current_playing(&self) -> Option<(u64, Duration)> {
        self.now_playing
            .lock()
            .unwrap()
            .as_ref()
            .map(|s| (s.track_id, s.player.get_pos()))
    }

    pub async fn play(&self, server_url: &str, token: String, track_id: u64) -> Result<(), Error> {
        let url = accentor_api::track_audio_url(server_url, track_id)?;

        let stream = HttpStream::new(AuthClient { token }, url)
            .await
            .map_err(|e| anyhow!("failed to open audio stream: {e}"))?;

        let reader =
            StreamDownload::from_stream(stream, TempStorageProvider::new(), Settings::default())
                .await
                .map_err(|e| anyhow!("failed to initialize stream: {e}"))?;

        self.tx
            .try_send(PlaybackCmd::Play(reader, track_id))
            .map_err(|e| match e {
                TrySendError::Full(_) => anyhow!("playback busy"),
                TrySendError::Disconnected(_) => anyhow!("audio thread is gone"),
            })?;
        Ok(())
    }
}
