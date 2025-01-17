use std::sync::Arc;

use crate::ws::file_storage::FileStorage;
use crate::ws::handler::Handler;
use crate::ws::http_header::HttpHeader;
use crate::ws::http_request::HttpRequest;
use crate::ws::http_response::{HttpResponse, StatusType};

pub struct StaticFileHandler {
    file_storage: Arc<FileStorage>,
    file_name: String,
}

impl Handler for StaticFileHandler {
    fn handle(&self, _request: &HttpRequest, response: &mut HttpResponse) {
        let file_content = if let Some(file_content) = self.file_storage.get(&self.file_name) {
            file_content
        } else {
            todo!("Response Not found");
        };

        let mut headers = Vec::new();
        headers.push(HttpHeader::new(
            "Content-Type",
            extension_to_http_mimo_type(
                &self.file_name[self
                    .file_name
                    .find('.')
                    .expect("File should have extension")..],
            ),
        ));

        *response = HttpResponse::new(StatusType::Ok, headers, file_content.to_vec());
    }
}

impl StaticFileHandler {
    pub fn new(file_storage: Arc<FileStorage>, file_name: String) -> Self {
        Self {
            file_storage,
            file_name,
        }
    }
}

fn extension_to_http_mimo_type(extension: &str) -> &str {
    match extension {
        ".html" => "text/html",
        ".css" => "text/css",
        ".png" => "image/png",
        ".js" => "text/javascript",
        _ => {
            todo!("Not supported mimo type");
        }
    }
}
