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
    Status,
    Login {
        server_url: String,
        username: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let cli = Cli::parse();

    let socket_path = get_socket_path();
    let mut transport = tarpc::serde_transport::unix::connect(&socket_path, Json::default);
    transport.config_mut().max_frame_length(usize::MAX);
    let client = CommunicationClient::new(client::Config::default(), transport.await?).spawn();

    match cli.command {
        Command::Status => match client.status(context::current()).await? {
            Ok(status) => {
                println!("Logged in to {}", status.server_url);
                println!("User id: {}", status.user_id);
            }
            Err(err) => {
                println!("{}", err)
            }
        },
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
    }

    Ok(())
}
