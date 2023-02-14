use crate::config;

use std::path::Path;
use std::path::PathBuf;
use hyper::Uri;

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