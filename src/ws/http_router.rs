use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use crate::ws::file_storage::FileStorage;
use crate::ws::handler::Handler;
use crate::ws::http_header::HttpHeader;
use crate::ws::http_request::HttpRequest;
use crate::ws::http_response::{HttpResponse, StatusType};
use crate::ws::method::Method;

macro_rules! check_or_handle_error {
    ($expr:expr, $error_page:expr, $status:expr, $request:expr, $response:expr, $self:expr) => {
        if let Some(value) = $expr {
            value
        } else {
            $self.handle_error($error_page.to_string(), $status, &$request, $response);
            return;
        }
    };
}

pub struct HttpRouter {
    routes: HashMap<Method, HashMap<String, Box<dyn Handler + Sync + Send>>>,
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
        let method = if let Ok(method) = Method::from_str(request.method.as_str()) {
            method
        } else {
            self.handle_error(
                "405.html".to_string(),
                StatusType::MethodNotAllowed,
                &request,
                response,
            );
            return;
        };

        let inner_map = check_or_handle_error!(
            self.routes.get(&method),
            "405.html",
            StatusType::MethodNotAllowed,
            &request,
            response,
            self
        );

        let handler = check_or_handle_error!(
            inner_map.get(&request.uri),
            "404.html",
            StatusType::NotFound,
            &request,
            response,
            self
        );

        handler.handle(request, response);
    }

    pub fn add_route<H>(&mut self, method: Method, uri: String, handler: H) -> &mut Self
    where
        H: Handler + Send + Sync + 'static,
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
