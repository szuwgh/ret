use crate::error::{RetError, RetResult};
use http_body_util::Empty;
use http_body_util::combinators::BoxBody;
use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper::http;
use url::{Host, Url};

pub(crate) type HttpBoxBody = http_body_util::combinators::BoxBody<Bytes, hyper::Error>;

pub(crate) fn host_addr(uri: &http::Uri) -> Option<String> {
    uri.authority().map(|auth| auth.to_string())
}

pub(crate) fn empty() -> HttpBoxBody {
    Empty::<Bytes>::new()
        .map_err(|never| match never {})
        .boxed()
}

pub(crate) fn empty_stream() -> BoxBody<Bytes, std::io::Error> {
    Empty::<Bytes>::new()
        .map_err(|never| match never {})
        .boxed()
}

pub(crate) fn full<T: Into<Bytes>>(chunk: T) -> HttpBoxBody {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}

#[derive(Debug)]
pub(crate) struct UrlProto {
    scheme: String,
    host: String,
    port: u16,
}

impl UrlProto {
    fn get_target_host(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

//解析 http://:8080
pub(crate) fn parse_url(input: &str) -> RetResult<UrlProto> {
    //
    if let Some((scheme, rest)) = input.split_once("://") {
        let (host_part, port_part) = if let Some((host, port)) = rest.split_once(':') {
            (host, port)
        } else {
            (rest, "")
        };

        return Ok(UrlProto {
            scheme: scheme.to_string(),
            host: host_part.to_string(),
            port: port_part.parse::<u16>()?,
        });
    }
    return Err(RetError::AddrParseError(input.to_string()));
}

#[cfg(test)]
mod tests {

    use super::*;
    #[test]
    fn test_parse_url() {
        let u = parse_url("http://127.0.0.1:8080").unwrap();
        println!("u:{:?}", u);
    }
}
