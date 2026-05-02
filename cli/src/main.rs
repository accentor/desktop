use accentor_communication::CommunicationClient;
use accentor_utils::get_socket_path;
use anyhow::Error;
use tarpc::{client, context, tokio_serde::formats::Json};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let socket_path = get_socket_path();
    let mut transport = tarpc::serde_transport::unix::connect(&socket_path, Json::default);
    transport.config_mut().max_frame_length(usize::MAX);
    let client = CommunicationClient::new(client::Config::default(), transport.await?).spawn();

    let hello = client
        .hello(context::current(), "Accentor".to_string())
        .await?;
    println!("{}", hello);

    Ok(())
}
