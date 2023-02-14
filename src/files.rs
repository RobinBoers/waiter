use crate::config;

use std::path::Path;
use std::path::PathBuf;

use hyper::Uri;
use hyper::body::Bytes;

pub async fn read_file(filename: &str) -> Result<Bytes, String> {
    if let Ok(contents) = tokio::fs::read(filename).await {
        let body: Bytes = contents.into();
        return Ok(body)
    }

    Err(String::from("File not found or couldn't read it."))
}

pub async fn write_file(filename: &str, bytes: Bytes) -> Result<(), String> {
    // let file = open_or_create_file(filename).await?;

    let error = format!("An unknown error occured while trying to write {filename}.");
    tokio::fs::write(filename, bytes).await.map_err(|_| error)?;

    Ok(())
}

// async fn open_or_create_file(filename: &str) -> Result<File, String> {
//     if let Ok(file) = tokio::fs::File::open(filename).await {
//         Ok(file)
//     } else {
//         match tokio::fs::File::create(filename).await {
//             Ok(file) => Ok(file),
//             Err(_error) => Err(format!("Couldn't create file {filename}"))
//         }
//     }
// }

pub fn get_path_buffer_for_allowed_path(path: &str) -> Option<PathBuf> {
    let scope = Path::new(config::SCOPE).canonicalize().unwrap();
    let path = Path::new(path).canonicalize();

    let path_buffer = path.ok()?;

    if !path_buffer.starts_with(scope) {
        return None
    }

    Some(path_buffer)
}

pub fn uri_to_local_path(uri: &Uri) -> String {
    format!(".{}", uri.path())
}