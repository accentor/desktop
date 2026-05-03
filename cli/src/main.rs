use accentor_communication::{CommunicationClient, PlayingTrack, TrackSummary};

const TRACKS_PER_PAGE: u32 = 30;
const WIDE_COVER_WIDTH: u32 = 50;
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
    Track {
        #[command(subcommand)]
        command: TrackCommand,
    },
    Status,
    Sync,
}

#[derive(Subcommand)]
enum TrackCommand {
    Play {
        track_id: u64,
    },
    Show {
        track_id: u64,
    },
    List {
        #[arg(long, default_value_t = 1)]
        page: u32,
    },
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
        Command::Track { command } => match command {
            TrackCommand::Play { track_id } => {
                match client.play(context::current(), track_id).await? {
                    Ok(pt) => display_playing_track(&pt).await,
                    Err(msg) => anyhow::bail!(msg),
                }
            }
            TrackCommand::Show { track_id } => {
                match client.track(context::current(), track_id).await? {
                    Ok(Some(t)) => print_track(&t),
                    Ok(None) => anyhow::bail!("Track {track_id} not found"),
                    Err(msg) => anyhow::bail!(msg),
                }
            }
            TrackCommand::List { page } => {
                match client
                    .tracks(context::current(), page, TRACKS_PER_PAGE)
                    .await?
                {
                    Ok(tp) => {
                        print_tracks_table(&tp.items);
                        println!("Page {page} of {}", tp.total_pages);
                    }
                    Err(msg) => anyhow::bail!(msg),
                }
            }
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
                    Some(pt) => display_playing_track(&pt).await,
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

fn print_track(t: &TrackSummary) {
    let length = t
        .length
        .map(|s| format_duration(s as u64))
        .unwrap_or_else(|| "?:??".to_string());
    println!("{}: {}", t.id, t.title);
    println!("  by {}", t.artists);
    println!("  on {}", t.album);
    println!("  genres: {}", t.genres);
    println!("  ({})", length);
}

fn print_tracks_table(tracks: &[TrackSummary]) {
    use comfy_table::presets::UTF8_FULL;
    let mut table = comfy_table::Table::new();
    table.load_preset(UTF8_FULL).set_header(vec![
        "ID", "#", "Title", "Album", "Artists", "Genres", "Length",
    ]);
    for t in tracks {
        let length = t
            .length
            .map(|s| format_duration(s as u64))
            .unwrap_or_else(|| "?:??".to_string());
        table.add_row(vec![
            t.id.to_string(),
            t.number.to_string(),
            t.title.clone(),
            t.album.clone(),
            t.artists.clone(),
            t.genres.clone(),
            length,
        ]);
    }
    println!("{table}");
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

async fn display_playing_track(pt: &PlayingTrack) {
    if let Some(url) = &pt.cover_url {
        let _ = render_cover(url, WIDE_COVER_WIDTH).await;
    }
    print_playing_track(pt);
}

async fn render_cover(url: &str, size: u32) -> anyhow::Result<()> {
    let bytes = accentor_utils::fetch_cover(url).await?;
    let img = image::load_from_memory(&bytes)?;
    let cfg = viuer::Config {
        width: Some(size),
        absolute_offset: false,
        ..Default::default()
    };
    viuer::print(&img, &cfg)?;
    println!();
    Ok(())
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
