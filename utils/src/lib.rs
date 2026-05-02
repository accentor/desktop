use std::path::PathBuf;

#[cfg(debug_assertions)]
fn socket_dir() -> PathBuf {
    PathBuf::from(std::env::var("PRJ_DATA_DIR").unwrap_or(".".to_string()))
}

#[cfg(not(debug_assertions))]
fn socket_dir() -> PathBuf {
    dirs::state_dir()
        .or_else(dirs::data_local_dir)
        .or_else(|| dirs::home_dir().map(|h| h.join(".local/share")))
        .expect("could not determine a directory for the socket")
}

pub fn get_socket_path() -> PathBuf {
    socket_dir().join("accentord.socket")
}
