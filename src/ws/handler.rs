use crate::ws::http_request::HttpRequest;
use crate::ws::http_response::HttpResponse;

pub trait Handler {
    fn handle(&self, request: &HttpRequest, response: &mut HttpResponse);
}
