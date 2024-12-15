use crate::ws::http_request::HttpRequest;

pub enum ParseResult {
    Ok,
    Bad,
    Indeterminate,
}

#[derive(Clone)]
enum ParserState {
    MethodStart,
    Method,
}

#[derive(Clone)]
pub struct HttpRequestParser {
    state: ParserState,
}

impl HttpRequestParser {
    pub fn new() -> Self {
        Self {
            state: ParserState::MethodStart,
        }
    }

    fn consume(&mut self, _request: &mut HttpRequest, b: u8) -> ParseResult {
        match self.state {
            ParserState::MethodStart => {
                self.state = ParserState::Method;
                return ParseResult::Indeterminate;
            }
            ParserState::Method => {
                todo!();
            }
        }
    }

    pub fn parse(
        &mut self,
        request: &mut HttpRequest,
        input: impl Iterator<Item = u8>,
    ) -> ParseResult {
        for x in input {
            let result = self.consume(request, x);
            match result {
                ParseResult::Ok | ParseResult::Bad => return result,
                ParseResult::Indeterminate => {}
            }
        }

        ParseResult::Indeterminate
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ok() {
        let mut parser = HttpRequestParser::new();
        let mut request = HttpRequest::default();

        let input = b"GET / HTTP/1.1\r\n";
        let result = parser.parse(&mut request, input.iter().cloned());

        assert!(matches!(result, ParseResult::Ok));
    }
}
