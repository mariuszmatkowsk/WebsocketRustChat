#[derive(Clone)]
pub struct HttpRequest {
    method: String,
    uri: String,
    version_major: u8,
    version_minor: u8,
    body: String,
}

impl HttpRequest {
    pub fn default() -> Self {
        Self {
            method: String::new(),
            uri: String::new(),
            version_major: 0,
            version_minor: 0,
            body: String::new(),
        }
    }
}
