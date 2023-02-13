use std::path::Path;

use http_body_util::Full;
use hyper::body::Bytes;
use hyper::{Request, Response};

type Resp = Response<Full<Bytes>>;
type Req = Request<hyper::body::Incoming>;

pub mod mime;

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
    serve_path(&path).await
}

async fn try_index(path: &str) -> Option<Resp> {
    try_file(&format!("{path}/index")).await
}

fn find_file(base_path: &str) -> Option<String> {
    let possible_indexes = vec![
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

async fn serve_path(path: &str) -> Option<Resp> {
    let scope = Path::new(crate::SCOPE).canonicalize().unwrap();
    let path = Path::new(path).canonicalize();

    let path_buffer = match path {
        Ok(path_buffer) => path_buffer,
        Err(_error) => return None,
    };

    if !path_buffer.starts_with(scope) {
        return None;
    }

    let extension = path_buffer.extension().and_then(|s| s.to_str());
    let mime_type = mime::get_mime_type_by_extension(extension);

    let path = path_buffer.to_str().unwrap();

    match serve_file(path, mime_type).await {
        Ok(response) => Some(response),
        Err(_error) => None,
    }
}

async fn serve_file(filename: &str, mime_type: &str) -> Result<Resp, String> {
    if let Ok(contents) = tokio::fs::read(filename).await {
        let body: Bytes = contents.into();
        let charset = crate::CONTENT_CHARSET;

        let response = Response::builder()
            .status(200)
            .header("Content-Type", format!("{mime_type}; charset={charset}"))
            .body(Full::new(body))
            .unwrap();

        return Ok(response);
    }

    Err(String::from("File not found or couldn't read it"))
}
