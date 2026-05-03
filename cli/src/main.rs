use accentor_communication::{CommunicationClient, PlayingTrack};
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
            Ok(pt) => print_playing_track(&pt),
            Err(msg) => anyhow::bail!(msg),
        },
        Command::Status => match client.status(context::current()).await? {
            Ok(status) => {
                println!(
                    "Logged in to {} as {}",
                    status.server_url,
                    status.user_name.unwrap_or_else(|| "unknown".to_string())
                );
                if status.sync_in_progress {
                    println!("Sync: in progress");
                } else if let Some(err) = &status.last_sync_error {
                    println!("Last sync failed: {err}");
                } else if let Some(ts) = status.last_sync_at {
                    println!("Last sync: {ts}");
                } else {
                    println!("Sync: never run");
                }
                match status.playing_track {
                    Some(pt) => print_playing_track(&pt),
                    None => println!("Not playing"),
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

fn print_playing_track(pt: &PlayingTrack) {
    let pos = format_duration(pt.position_ms / 1_000);
    let total = pt
        .length
        .map(|s| format_duration(s as u64))
        .unwrap_or_else(|| "?:??".to_string());
    println!("Playing: {}", pt.title);
    println!("  by {}", pt.artists);
    println!("  on {}", pt.album);
    println!("  ({} / {})", pos, total);
}

fn format_duration(secs: u64) -> String {
    let hours = secs / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;
    if hours > 0 {
        format!("{}:{:02}:{:02}", hours, minutes, seconds)
    } else {
        format!("{}:{:02}", minutes, seconds)
    }
}
