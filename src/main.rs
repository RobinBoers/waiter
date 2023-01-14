use std::{fs::File, path::Path};
use rouille::Response;

fn main() {
    let address = "localhost:4000";
    println!("Now listening on {}", address);

    rouille::start_server(address, move |request| {
        let mut response = rouille::match_assets(request, ".");

        if !response.is_success() {
            if request.url() == "/" {
                response = serve_index()
            } else {
                response = serve_404()
            }
        }

        response = set_cache_time(response, request.url());
        response = set_correct_mime_type(response, request);
        response = set_server_header(response);

        response
    });
}

fn serve_index() -> rouille::Response {
    match find_index() {
        Some((filename, mime_type)) => serve_file(filename, mime_type),
        None => serve_404()
    }
}

fn find_index() -> Option<(String, String)> {
    let possible_indexes = vec![
        (String::from("index.htmd"), String::from("text/htmd")),
        (String::from("index.txt"), String::from("text/plain")),
        (String::from("index.html"), String::from("text/html")),
        (String::from("index.xml"), String::from("text/xml")),
    ];

    possible_indexes
        .iter()
        .find(|&(filename, _)| Path::new(&filename).exists())
        .cloned()
}

fn serve_file(filename: String, mime_type: String) -> rouille::Response {
    let file = File::open(filename).unwrap();
    Response::from_file(mime_type, file)
}

fn serve_404() -> rouille::Response {
    Response::text("Resource was not found on this server").with_status_code(404)
}

fn set_cache_time(response: rouille::Response, request_url: String) -> rouille::Response {
    let cache_time_in_seconds = get_cache_time_for_filetype(request_url);
    response.with_public_cache(cache_time_in_seconds)
}

fn get_cache_time_for_filetype(filename: String) -> u64 {
    if is_static_asset(filename) {
        31536000 // One year
    } else {
        43200 // One day
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

fn set_correct_mime_type(
    response: rouille::Response,
    request: &rouille::Request,
) -> rouille::Response {
    if is_htmd_file(&response, request) {
        if accepts_htmd_mime_type(request) {
            response.with_unique_header("Content-Type", "text/htmd")
        } else {
            response.with_unique_header("Content-Type", "text/plain")
        }
    } else {
        response
    }
}

fn is_htmd_file(response: &rouille::Response, request: &rouille::Request) -> bool {
    request.url().ends_with(".htmd") || current_response_has_htmd_content_type(response)
}

fn current_response_has_htmd_content_type(response: &rouille::Response) -> bool {
    let content_type = response.headers
            .iter()
            .find(|&&(ref k, _)| k.eq_ignore_ascii_case("Content-Type"))
            .map(|&(_, ref v)| &v[..])
            .unwrap();

    content_type.contains("text/htmd")
}

fn accepts_htmd_mime_type(request: &rouille::Request) -> bool {
    let accept_header = request.header("Accept").unwrap_or("*/*");
    accept_header.contains("text/htmd")
}

fn set_server_header(response: rouille::Response) -> rouille::Response {
    response.with_unique_header("Server", "waiter (Rust)")
}
