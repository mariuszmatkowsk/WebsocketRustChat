use crate::ws::http_header::HttpHeader;

#[derive(Clone, Debug)]
pub struct HttpRequest {
    pub method: String,
    pub uri: String,
    pub headers: Vec<HttpHeader>,
    pub version_major: u8,
    pub version_minor: u8,
    pub body: String,
}

impl HttpRequest {
    pub fn default() -> Self {
        Self {
            method: String::new(),
            uri: String::new(),
            headers: Vec::new(),
            version_major: 0,
            version_minor: 0,
            body: String::new(),
        }
    }
}
