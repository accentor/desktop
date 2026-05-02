use accentor_communication::CommunicationClient;
use accentor_utils::get_socket_path;
use anyhow::Error;
use clap::{Parser, Subcommand};
use tarpc::{client, context, tokio_serde::formats::Json};

#[derive(Parser)]
#[command(name = "accentor-cli", version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Login {
        server_url: String,
        username: String,
    },
    Play {
        track_id: u64,
    },
    Status,
    Sync,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let cli = Cli::parse();

    let socket_path = get_socket_path();
    let mut transport = tarpc::serde_transport::unix::connect(&socket_path, Json::default);
    transport.config_mut().max_frame_length(usize::MAX);
    let client = CommunicationClient::new(
        client::Config::default(),
        transport.await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                anyhow::anyhow!("Could not connect to accentord: socket {} does not exist. Is the daemon running?", socket_path.display())
            } else {
                e.into()
            }
        })?,
    )
    .spawn();

    match cli.command {
        Command::Login {
            server_url,
            username,
        } => {
            let password = rpassword::prompt_password("Password: ")?;
            match client
                .login(context::current(), server_url.clone(), username, password)
                .await?
            {
                Ok(()) => println!("Logged in to {}", server_url),
                Err(msg) => anyhow::bail!(msg),
            }
        }
        Command::Play { track_id } => match client.play(context::current(), track_id).await? {
            Ok(()) => println!("Playing track {}", track_id),
            Err(msg) => anyhow::bail!(msg),
        },
        Command::Status => match client.status(context::current()).await? {
            Ok(status) => {
                println!("Logged in to {}", status.server_url);
                match &status.user_name {
                    Some(name) => println!("User: {}", name),
                    None => println!("User: unknown"),
                };
                match status.playing_track_id {
                    Some(id) => {
                        let ms = status.playing_position_ms.unwrap_or(0);
                        println!(
                            "Playing track {} ({}:{:02})",
                            id,
                            ms / 60_000,
                            ms / 1_000 % 60
                        );
                    }
                    None => println!("Not playing"),
                }
                if status.sync_in_progress {
                    println!("Sync: in progress");
                } else if let Some(err) = &status.last_sync_error {
                    println!("Last sync failed: {err}");
                } else if let Some(ts) = status.last_sync_at {
                    println!("Last sync: {ts}");
                } else {
                    println!("Sync: never run");
                }
            }
            Err(err) => {
                println!("{}", err)
            }
        },
        Command::Sync => match client.sync(context::current()).await? {
            Ok(()) => println!("Started sync"),
            Err(msg) => anyhow::bail!(msg),
        },
    }

    Ok(())
}
