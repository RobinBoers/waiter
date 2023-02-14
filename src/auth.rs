use crate::config;
use crate::response;

use http_body_util::Full;
use hyper::body::Bytes;
use hyper::header::HeaderValue;
use hyper::{Request, Response};

type Resp = Response<Full<Bytes>>;
type Req = Request<hyper::body::Incoming>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientCredentials {
    pub username: String,
    pub password: String,
}

pub fn require_authentication(request: Req) -> Result<Req, Resp> {
    let auth = match fetch_auth_credentials(&request) {
        Some(credentials) => credentials,
        None => return Err(serve_auth_required()),
    };

    if auth.username == config::AUTH_USERNAME && auth.password == config::AUTH_PASSWORD {
        Ok(request)
    } else {
        Err(response::serve(
            403,
            "Invalid credentials; authentication is required for `PUT` requests.",
        ))
    }
}

fn fetch_auth_credentials(request: &Req) -> Option<ClientCredentials> {
    let header = match request.headers().get("Authorization") {
        None => return None,
        Some(header) => header.to_str().unwrap(),
    };

    let mut split = header.splitn(2, |c| c == ' ');
    let authtype = match split.next() {
        None => return None,
        Some(t) => t,
    };

    if authtype != "Basic" {
        return None;
    }

    let authvalue = match split.next().and_then(|val| base64::decode(val).ok()) {
        Some(v) => v,
        None => return None,
    };

    let mut split = authvalue.splitn(2, |&c| c == b':');

    let username = match split
        .next()
        .map(Vec::from)
        .and_then(|l| String::from_utf8(l).ok())
    {
        Some(l) => l,
        None => return None,
    };
    let password = match split
        .next()
        .map(Vec::from)
        .and_then(|p| String::from_utf8(p).ok())
    {
        Some(p) => p,
        None => return None,
    };

    Some(ClientCredentials { username, password })
}

fn serve_auth_required() -> Resp {
    let mut response = response::serve(
        401,
        "Missing credentials; authentication is required for `PUT` requests.",
    );

    set_www_authenticate_header(&mut response);
    response
}

fn set_www_authenticate_header(response: &mut Resp) {
    let headers = response.headers_mut();
    let header_value = format!("Basic realm=\"{}\"", config::AUTH_REALM);

    headers.insert(
        "WWW-Authenticate",
        HeaderValue::from_str(&header_value).unwrap(),
    );
}
