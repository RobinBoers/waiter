use crate::config;

use std::path::Path;
use std::path::PathBuf;

use hyper::body::Bytes;
use hyper::Uri;

pub async fn read_file(filename: &str) -> Result<Bytes, String> {
    if let Ok(contents) = tokio::fs::read(filename).await {
        let body: Bytes = contents.into();
        return Ok(body);
    }

    Err(format!("File {filename} not found or couldn't read it."))
}

pub async fn write_file(filename: &str, bytes: Bytes) -> Result<(), String> {
    let error = format!("An unknown error occured while trying to write {filename}.");
    tokio::fs::write(filename, bytes).await.map_err(|_| error)?;

    Ok(())
}

pub fn get_path_buffer_for_allowed_path(path: &str) -> Option<PathBuf> {
    let scope = Path::new(config::SCOPE).canonicalize().ok()?;
    let path = Path::new(path).canonicalize();

    let path_buffer = path.ok()?;

    if !path_buffer.starts_with(scope) {
        return None;
    }

    Some(path_buffer)
}

pub fn uri_to_local_path(uri: &Uri) -> String {
    format!(".{}", uri.path())
}
