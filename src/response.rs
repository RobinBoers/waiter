use crate::config;
use crate::files;
pub mod mime;

use std::path::Path;

use http_body_util::Full;
use hyper::body::Bytes;
use hyper::{Request, Response};

type Resp = Response<Full<Bytes>>;
type Req = Request<hyper::body::Incoming>;

pub fn serve(status_code: u16, message: &str) -> Resp {
    let body = message.to_string().into();

    Response::builder()
        .status(status_code)
        .body(Full::new(body))
        .unwrap()
}

pub fn serve_404() -> Resp {
    serve(404, "Resource was not found on this server")
}

pub async fn try_files(request: &Req) -> Resp {
    let path = &format!(".{}", request.uri().path());

    if let Some(response) = try_file(path).await {
        response
    } else if let Some(response) = try_index(path).await {
        response
    } else {
        serve_404()
    }
}

async fn try_file(path: &str) -> Option<Resp> {
    let path = find_file(path)?;
    handle_path(&path).await
}

async fn try_index(path: &str) -> Option<Resp> {
    try_file(&format!("{path}/index")).await
}

fn find_file(base_path: &str) -> Option<String> {
    let possible_indexes = vec![
        format!("{base_path}"),
        format!("{base_path}.htmd"),
        format!("{base_path}.txt"),
        format!("{base_path}.html"),
        format!("{base_path}.xml"),
    ];

    possible_indexes
        .iter()
        .find(|file_path| file_exists(file_path))
        .cloned()
}

fn file_exists(path: &str) -> bool {
    Path::new(path).exists()
}

async fn handle_path(path: &str) -> Option<Resp> {
    let path_buffer = files::get_path_buffer_for_allowed_path(path)?;

    let extension = path_buffer.extension().and_then(|s| s.to_str());
    let mime_type = mime::get_mime_type_by_extension(extension);

    let path = path_buffer.to_str().unwrap();

    match serve_file(path, mime_type).await {
        Ok(response) => Some(response),
        Err(_error) => None,
    }
}

async fn serve_file(filename: &str, mime_type: &str) -> Result<Resp, String> {
    let body = files::read_file(filename).await?;
    let charset = config::CONTENT_CHARSET;

    let response = Response::builder()
        .status(200)
        .header("Content-Type", format!("{mime_type}; charset={charset}"))
        .body(Full::new(body))
        .unwrap();

    Ok(response)
}
