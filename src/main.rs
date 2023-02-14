use hyper::server::conn::http1;
use hyper::service::service_fn;
use std::convert::Infallible;
use tokio::net::TcpListener;

use http_body_util::Full;
use hyper::body::Bytes;
use hyper::header::HeaderValue;
use hyper::{Method, Request, Response};

type Resp = Response<Full<Bytes>>;
type Req = Request<hyper::body::Incoming>;

mod auth;
mod config;
mod files;
mod response;
mod uploads;
mod content_encoding;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Address for the server to run on
    #[arg(short, long, default_value_t = String::from("localhost:4000"))]
    address: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args = Args::parse();
    println!("Now listening on {}", args.address);

    let listener = TcpListener::bind(args.address).await?;

    loop {
        let (stream, _) = listener.accept().await?;

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(stream, service_fn(handle_request))
                .await
            {
                println!("Error serving connection: {:?}", err);
            }
        });
    }
}

async fn handle_request(request: Req) -> Result<Resp, Infallible> {
    match *request.method() {
        Method::PUT => handle_put_request(request).await,
        Method::GET => handle_get_request(request).await,
        _ => Ok(response::serve(
            400,
            "Bad request; only `GET` and `PUT` requests are supported.",
        )),
    }
}

async fn handle_put_request(request: Req) -> Result<Resp, Infallible> {
    match auth::require_authentication(request) {
        Ok(request) => uploads::process_put_request(request).await,
        Err(response) => Ok(response),
    }
}

async fn handle_get_request(request: Req) -> Result<Resp, Infallible> {
    let mut response = response::try_files(&request).await;
    let url = request.uri();

    set_cache_time(&mut response, url.path());
    set_additional_headers(&mut response);
    maybe_correct_content_type(&mut response, &request);
    
    content_encoding::apply(&mut response, &request);

    Ok(response)
}

fn set_cache_time(response: &mut Resp, request_url: &str) {
    let cache_time_in_seconds = get_cache_time_for_filetype(request_url);
    let header_value = format!("max-age={cache_time_in_seconds}");

    let headers = response.headers_mut();
    headers.insert(
        "Cache-Control",
        HeaderValue::from_str(&header_value).unwrap(),
    );
}

fn get_cache_time_for_filetype(filename: &str) -> u64 {
    if is_static_asset(filename) {
        config::CACHE_TIME_ASSETS
    } else {
        config::CACHE_TIME_CONTENT
    }
}

fn is_static_asset(filename: &str) -> bool {
    let asset_file_types = vec![
        ".ico", ".jpg", ".jpeg", ".png", ".webp", ".gif", ".svg", ".woff", ".woff2",
    ];

    asset_file_types
        .iter()
        .any(|&suffix| filename.ends_with(suffix))
}

fn set_additional_headers(response: &mut Resp) {
    let headers = response.headers_mut();

    headers.insert("Server", HeaderValue::from_static(config::SERVER_NAME));
    headers.insert(
        "Content-Language",
        HeaderValue::from_static(config::CONTENT_LANGUAGE),
    );
}

fn maybe_correct_content_type(response: &mut Resp, request: &Req) {
    if serving_htmd_file(response) && !accepts_htmd_mime_type(request) {
        let headers = response.headers_mut();

        headers.insert("Content-Type", HeaderValue::from_static("text/plain"));
    }
}

fn serving_htmd_file(response: &mut Resp) -> bool {
    let default_header = HeaderValue::from_static("*/*");

    response
        .headers()
        .get("Content-Type")
        .unwrap_or(&default_header)
        .to_str()
        .expect("Content-Type header in response should be a string!")
        .contains("text/htmd")
}

fn accepts_htmd_mime_type(request: &Req) -> bool {
    let default_header = HeaderValue::from_static("*/*");

    request
        .headers()
        .get("Accept")
        .unwrap_or(&default_header)
        .to_str()
        .expect("Accept header in request should be a string!")
        .contains("text/htmd")
}
