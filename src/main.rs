use rouille::{Request, RequestBody, Response};
use std::{fs, fs::File, io::Read, path::Path};

const SERVER_NAME: &str = "0bx11/waiter (Rust)";
const CONTENT_LANGUAGE: &str = "en-US";
const CONTENT_CHARSET: &str = "UTF-8";

const CACHE_TIME_ASSETS: u64 = 31536000;
const CACHE_TIME_CONTENT: u64 = 43200;

const AUTH_USERNAME: &str = "root";
const AUTH_PASSWORD: &str = "toor";

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Address for the server to run on
    #[arg(short, long, default_value_t = String::from("localhost:4000"))]
    address: String,
}

fn main() {
    let args = Args::parse();
    println!("Now listening on {}", args.address);

    rouille::start_server(args.address, move |request| match request.method() {
        "PUT" => handle_put_request(request),
        "GET" => handle_get_request(request),
        _ => serve(
            400,
            "Bad request; only `GET` and `PUT` requests are supported.",
        ),
    });
}

fn handle_put_request(request: &Request) -> Response {
    match require_authentication(request) {
        Ok(request) => process_put_request(request),
        Err(response) => response,
    }
}

fn require_authentication(request: &Request) -> Result<&Request, Response> {
    let auth = match rouille::input::basic_http_auth(request) {
        Some(credentials) => credentials,
        None => return Err(Response::basic_http_auth_login_required("PUT")),
    };

    if auth.login == AUTH_USERNAME && auth.password == AUTH_PASSWORD {
        Ok(request)
    } else {
        Err(serve(403, "Invalid credentials; authentication is required for `PUT` requests."))
    }
}

fn process_put_request(request: &Request) -> Response {
    if request.header("Expect") != Some("100-continue") {
        match request.data() {
            Some(request_body) => upload_file(request_body, request.url()),
            _ => serve(400, "Bad request; empty body, nothing to upload."),
        }
    } else {
        serve(
            400,
            "Bad request; waiter doesn't support the Expect header.",
        )
    }
}

fn upload_file(mut request_body: RequestBody, filepath: String) -> Response {
    let mut buffer = String::new();

    match request_body.read_to_string(&mut buffer) {
        Ok(0) => serve(400, "Bad request; empty body, nothing to upload."),
        Ok(_len) => {
            fs::write(format!(".{filepath}"), buffer)
                .expect(&format!("Unable to write file {filepath}"));

            serve(201, "")
        }
        Err(_) => serve(400, "Bad request; couldn't parse request body."),
    }
}

fn handle_get_request(request: &Request) -> Response {
    let mut response = rouille::match_assets(request, ".");

    if !response.is_success() {
        let url = request.url();

        if url.ends_with("/") {
            response = serve_index(url)
        } else {
            response = serve_404()
        }
    }

    response = set_cache_time(response, request.url());
    response = set_correct_mime_type(response, request);
    response = set_additional_headers(response);

    rouille::content_encoding::apply(request, response)
}

fn serve_index(path: String) -> Response {
    match find_index(path) {
        Some((filename, mime_type)) => serve_file(filename, mime_type),
        None => serve_404(),
    }
}

fn find_index(path: String) -> Option<(String, String)> {
    let possible_indexes = vec![
        (format!(".{path}index.htmd"), String::from("text/htmd")),
        (format!(".{path}index.txt"), String::from("text/plain")),
        (format!(".{path}index.html"), String::from("text/html")),
        (format!(".{path}index.xml"), String::from("text/xml")),
    ];

    possible_indexes
        .iter()
        .find(|&(filename, _)| Path::new(&filename).exists())
        .cloned()
}

fn serve_file(filename: String, mime_type: String) -> Response {
    let file = File::open(filename).unwrap();
    Response::from_file(mime_type, file)
}

fn serve_404() -> Response {
    serve(404, "Resource was not found on this server")
}

fn serve(status_code: u16, message: &str) -> Response {
    Response::text(message).with_status_code(status_code)
}

fn set_cache_time(response: Response, request_url: String) -> Response {
    let cache_time_in_seconds = get_cache_time_for_filetype(request_url);
    response.with_public_cache(cache_time_in_seconds)
}

fn get_cache_time_for_filetype(filename: String) -> u64 {
    if is_static_asset(filename) {
        CACHE_TIME_ASSETS
    } else {
        CACHE_TIME_CONTENT
    }
}

fn is_static_asset(filename: String) -> bool {
    let asset_file_types = vec![
        ".ico", ".jpg", ".jpeg", ".png", ".webp", ".gif", ".svg", ".woff", ".woff2",
    ];

    asset_file_types
        .iter()
        .any(|&suffix| filename.ends_with(suffix))
}

fn set_correct_mime_type(response: Response, request: &Request) -> rouille::Response {
    if is_htmd_file(&response, request) {
        if accepts_htmd_mime_type(request) {
            response.with_unique_header(
                "Content-Type",
                format!("text/htmd; charset={CONTENT_CHARSET}"),
            )
        } else {
            response.with_unique_header(
                "Content-Type",
                format!("text/plain; charset={CONTENT_CHARSET}"),
            )
        }
    } else {
        append_charset_to_content_type(response)
    }
}

fn is_htmd_file(response: &Response, request: &Request) -> bool {
    request.url().ends_with(".htmd") || current_response_has_htmd_content_type(response)
}

fn current_response_has_htmd_content_type(response: &Response) -> bool {
    let content_type = get_content_type_header(response);
    content_type.contains("text/htmd")
}

fn get_content_type_header(response: &Response) -> &str {
    response
        .headers
        .iter()
        .find(|&&(ref k, _)| k.eq_ignore_ascii_case("Content-Type"))
        .map(|&(_, ref v)| &v[..])
        .unwrap()
}

fn accepts_htmd_mime_type(request: &Request) -> bool {
    let accept_header = request.header("Accept").unwrap_or("*/*");
    accept_header.contains("text/htmd")
}

fn append_charset_to_content_type(response: Response) -> Response {
    let content_type_header = get_content_type_header(&response);

    if !content_type_header.contains("charset") {
        let content_type = format!("{content_type_header}; charset={CONTENT_CHARSET}");
        response.with_unique_header("Content-Type", content_type)
    } else {
        response
    }
}

fn set_additional_headers(response: Response) -> Response {
    response
        .with_unique_header("Server", SERVER_NAME)
        .with_unique_header("Content-Language", CONTENT_LANGUAGE)
}
