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

#[cfg(debug_assertions)]
fn data_dir() -> PathBuf {
    PathBuf::from(std::env::var("PRJ_DATA_DIR").unwrap_or(".".to_string()))
}

#[cfg(not(debug_assertions))]
fn data_dir() -> PathBuf {
    dirs::data_local_dir()
        .or_else(|| dirs::home_dir().map(|h| h.join(".local/share")))
        .expect("could not determine a data directory")
}

pub fn get_data_path() -> PathBuf {
    data_dir().join("accentor")
}

#[cfg(debug_assertions)]
fn cache_dir() -> PathBuf {
    PathBuf::from(std::env::var("PRJ_DATA_DIR").unwrap_or(".".to_string()))
}

#[cfg(not(debug_assertions))]
fn cache_dir() -> PathBuf {
    dirs::cache_dir()
        .or_else(|| dirs::home_dir().map(|h| h.join(".cache")))
        .expect("could not determine a cache directory")
}

pub fn get_cache_path() -> PathBuf {
    cache_dir().join("accentor")
}

fn url_to_filename(url: &str) -> String {
    let mut out = String::with_capacity(url.len());
    for b in url.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-' | b'_') {
            out.push(b as char);
        } else {
            use std::fmt::Write;
            let _ = write!(out, "%{b:02X}");
        }
    }
    out
}

pub async fn fetch_cover(url: &str) -> anyhow::Result<Vec<u8>> {
    let cache_file = get_cache_path().join("covers").join(url_to_filename(url));
    match tokio::fs::read(&cache_file).await {
        Ok(bytes) => return Ok(bytes),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(e.into()),
    }
    let bytes = accentor_api::http_client()
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?
        .to_vec();
    if let Some(parent) = cache_file.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    tokio::fs::write(&cache_file, &bytes).await?;
    Ok(bytes)
}

pub fn unix_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}
