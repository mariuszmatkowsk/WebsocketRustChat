use crate::ws::handler::Handler;
use crate::ws::http_request::HttpRequest;
use crate::ws::http_response::HttpResponse;

pub struct Middleware {
    action: Box<dyn Handler + Send + Sync>,
    wrapped: Box<dyn Handler + Send + Sync>,
}

impl Middleware {
    pub fn new<A, H>(action: A, wrapped: H) -> Self
    where
        A: Handler + Sync + Send + 'static,
        H: Handler + Sync + Send + 'static,
    {
        Self {
            action: Box::new(action),
            wrapped: Box::new(wrapped),
        }
    }
}

impl Handler for Middleware {
    fn handle(&self, request: &HttpRequest, response: &mut HttpResponse) {
        self.action.handle(request, response);
        self.wrapped.handle(request, response);
    }
}

pub struct RequestLogger {}

impl RequestLogger {
    pub fn new() -> Self {
        Self {}
    }
}

impl Handler for RequestLogger {
    fn handle(&self, request: &HttpRequest, _response: &mut HttpResponse) {
        println!("**************************** Request **********************************");
        println!("Method: {}", request.method);
        println!("Uri: {}", request.uri);
        println!("Version: HTTP/{}.{}", request.version_major, request.version_minor);
        println!("Headers:");
        for h in request.headers.iter() {
            println!("\t{}: {}", h.name, h.value);
        }
        println!("***********************************************************************");
    }
}

