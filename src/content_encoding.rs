// Implementation stolen from rouille
// MIT licensed.

use http_body_util::Full;
use hyper::body::Bytes;
use hyper::{Request, Response};
use hyper::header::HeaderValue;

type Resp = Response<Full<Bytes>>;
type Req = Request<hyper::body::Incoming>;

pub fn apply(response: &mut Resp, request: &Req) {
    if !content_is_text(response) || has_content_encoding_header(response) {
        return
    }

    let _encoding_preference = ["br", "gzip", "x-gzip", "identity"];

    let default_header = HeaderValue::from_static("");
    let _accept_encoding_header = request.headers().get("Accept-Encoding").unwrap_or(&default_header);

    // MAGIC!
    // if let Some(preferred_index) = input::priority_header_preferred(
    //     accept_encoding_header,
    //     encoding_preference.iter().cloned(),
    // ) {
    //     match encoding_preference[preferred_index] {
    //         "br" => brotli(&mut response),
    //         "gzip" | "x-gzip" => gzip(&mut response),
    //         _ => (),
    //     }
    // }
}

fn content_is_text(response: &mut Resp) -> bool {
    let default_header = HeaderValue::from_static("*/*");
    let content_type_header = response.headers().get("Content-Type").unwrap_or(&default_header).to_str().expect("Content-Type header in response should be a string").to_lowercase();

    content_type_header.starts_with("text/")
        || content_type_header.contains("javascript")
        || content_type_header.contains("json")
        || content_type_header.contains("xml")
        || content_type_header.contains("font")
}

fn has_content_encoding_header(response: &mut Resp) -> bool {
    response.headers().get("Content-Encoding").is_some()
}