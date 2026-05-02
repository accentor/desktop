use accentor_communication::Communication;
use accentor_utils::get_socket_path;
use anyhow::Error;
use futures::{future, prelude::*};
use tarpc::{
    context,
    server::{self, Channel},
    tokio_serde::formats::Json,
};
use tokio::signal::unix::{SignalKind, signal};

#[derive(Clone)]
struct AccentorServer();

impl Communication for AccentorServer {
    async fn hello(self, _: context::Context, name: String) -> String {
        format!("Hello, {name}! You are connected!")
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let socket_path = get_socket_path();

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
            let server = AccentorServer();
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
