use crate::ws::http_header::HttpHeader;

#[derive(Clone, Copy)]
pub enum StatusType {
    Ok = 200,
    // Created = 201,
    // Accepted = 202,
    // NoContent = 204,
    // MultipleChoices = 300,
    // MovedPermanently = 301,
    // MovedTemporarily = 302,
    // NotModified = 304,
    // BadRequest = 400,
    // Unauthorized = 401,
    // Forbidden = 403,
    // NotFound = 404,
    // MethodNotAllowed = 405,
    InternalServerError = 500,
    // NotImplemented = 501,
    // BadGetway = 502,
    // ServiceUnavailable = 503,
}

impl StatusType {
    fn to_string(&self) -> String {
        match self {
            Self::Ok => String::from("Ok"),
            // Self::NotFound => String::from("Not Found"),
            Self::InternalServerError => String::from("Internal Server Errror"),
        }
    }
}

#[derive(Clone)]
pub struct HttpResponse {
    pub status: StatusType,
    pub headers: Vec<HttpHeader>,
    pub body: Vec<u8>,
}

impl HttpResponse {
    pub fn default() -> Self {
        Self {
            status: StatusType::Ok,
            headers: Vec::new(),
            body: Vec::new(),
        }
    }

    pub fn bytes(&self) -> Vec<u8> {
        let mut response = Vec::new();

        response.extend_from_slice(
            format!(
                "HTTP/1.1 {} {}\r\n",
                self.status as u8,
                self.status.to_string()
            )
            .as_bytes(),
        );

        for header in &self.headers {
            response
                .extend_from_slice(format!("{}: {}\r\n", header.name, header.value)
                .as_bytes());
        }

        response.extend_from_slice("\r\n".as_bytes());

        response.extend_from_slice(&self.body);

        response
    }
}
