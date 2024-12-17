use crate::ws::http_request::HttpRequest;
use crate::ws::http_response::HttpResponse;
use crate::ws::file_storage::FileStorage;
use crate::ws::method::Method;
use std::collections::HashMap;
use std::str::FromStr;

type Handler = fn(&HttpRequest, &mut HttpResponse);

pub struct HttpRouter<'a> {
    routes: HashMap<Method, HashMap<String, Handler>>,
    file_storage: &'a FileStorage,
}

impl<'a> HttpRouter<'a> {
    pub fn new(file_storage: &'a FileStorage) -> Self {
        Self {
            routes: HashMap::new(),
            file_storage,
        }
    }

    pub fn handle(&self, request: &HttpRequest, response: &mut HttpResponse) {
        let method = match Method::from_str(request.method.as_str()) {
            Ok(method) => method,
            Err(_) => {
                eprintln!("Not supported method in http request");
                todo!("Return method not supported");
            }
        };

        let inner_map = match self.routes.get(&method) {
            Some(inner_map) => inner_map,
            None => {
                todo!("Prepare response method not allowd");
            }
        };

        let handler = match inner_map.get(&request.uri) {
            Some(handler) => handler,
            None => {
                todo!("Resource not found");
            }
        };

        handler(request, response);
    }

    pub fn add_route(&mut self, method: Method, uri: String, handler: Handler) -> &mut Self  {
        self.routes
            .entry(method)
            .or_insert_with(HashMap::new)
            .insert(uri, handler);
        self
    }
}
