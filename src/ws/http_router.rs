use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use crate::ws::file_storage::FileStorage;
use crate::ws::http_header::HttpHeader;
use crate::ws::http_request::HttpRequest;
use crate::ws::http_response::{HttpResponse, StatusType};
use crate::ws::method::Method;

type Handler = Box<dyn Fn(&HttpRequest, &mut HttpResponse) + Send + Sync>;

// pub trait Handler {
//     pub fn handle(&HttpRequest, &mut HttpResponse);
// }

pub struct HttpRouter {
    routes: HashMap<Method, HashMap<String, Handler>>,
    file_storage: Arc<FileStorage>,
}

impl HttpRouter {
    pub fn new(file_storage: Arc<FileStorage>) -> Self {
        Self {
            routes: HashMap::new(),
            file_storage,
        }
    }

    pub fn handle(&self, request: &HttpRequest, response: &mut HttpResponse) {
        let method = match Method::from_str(request.method.as_str()) {
            Ok(method) => method,
            Err(_) => {
                self.handle_error(
                    "405.html".to_string(),
                    StatusType::MethodNotAllowed,
                    &request,
                    response,
                );
                return;
            }
        };

        let inner_map = match self.routes.get(&method) {
            Some(inner_map) => inner_map,
            None => {
                self.handle_error(
                    "405.html".to_string(),
                    StatusType::MethodNotAllowed,
                    &request,
                    response,
                );
                return;
            }
        };

        let handler = match inner_map.get(&request.uri) {
            Some(handler) => handler,
            None => {
                self.handle_error(
                    "404.html".to_string(),
                    StatusType::NotFound,
                    &request,
                    response,
                );
                return;
            }
        };

        handler(request, response);
    }

    pub fn add_route<H>(&mut self, method: Method, uri: String, handler: H) -> &mut Self
    where
        H: Fn(&HttpRequest, &mut HttpResponse) + Send + Sync + 'static,
    {
        self.routes
            .entry(method)
            .or_insert_with(HashMap::new)
            .insert(uri, Box::new(handler));
        self
    }

    fn handle_error(
        &self,
        error_file: String,
        status: StatusType,
        _: &HttpRequest,
        response: &mut HttpResponse,
    ) {
        let file_content = match self.file_storage.get(&error_file) {
            Some(content) => content,
            None => unreachable!("404.html and 405.html should be already verified and cached."),
        };

        let mut headers = Vec::new();
        headers.push(HttpHeader::new("Content-Type", "text/html"));

        *response = HttpResponse::new(status, headers, file_content.to_vec());
    }
}
