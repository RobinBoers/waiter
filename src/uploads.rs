use http_body_util::Full;
use hyper::body::Bytes;
use hyper::{Request, Response};

type Resp = Response<Full<Bytes>>;
type Req = Request<hyper::body::Incoming>;

pub fn process_put_request(_request: Req) -> Resp {
    todo!()
}
