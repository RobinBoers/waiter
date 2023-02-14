use crate::files;
use crate::response;

use std::convert::Infallible;

use http_body_util::{Full, BodyExt};
use hyper::body::Bytes;
use hyper::{Request, Response};

type Resp = Response<Full<Bytes>>;
type Req = Request<hyper::body::Incoming>;

pub async fn process_put_request(request: Req) -> Result<Resp, Infallible> {
    match read_body(request).await {
        Some(bytes) => handle_upload(request, bytes).await,
        None => Ok(response::serve(400, "Bad request; empty body, nothing to upload."))
    }
}

async fn read_body(request: Req) -> Option<Bytes> {
    let collected_body = request.into_body().collect().await.ok()?;
    Some(collected_body.to_bytes())
}

async fn handle_upload(request: Req, bytes: Bytes) -> Result<Resp, Infallible> {
    let path = &files::uri_to_local_path(request.uri());

    match files::get_path_buffer_for_allowed_path(path) {
        Some(path_buffer) => {
            let path = path_buffer.to_str().unwrap();

            match upload_file(path, bytes).await {
                Ok(response) => Ok(response),
                Err(error) => Ok(response::serve(500, &format!("Internal server error; {error}")))
            }
        },
        None => Ok(response::serve(403, "Forbidden; you cannot upload outside of scope."))
    } 
}

async fn upload_file(path: &str, bytes: Bytes) -> Result<Resp, String> {
    todo!()
}