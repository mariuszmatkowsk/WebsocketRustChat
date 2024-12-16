use crate::ws::http_header::HttpHeader;
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
    Uri,
    HttpVersionH,
    HttpVersionT1,
    HttpVersionT2,
    HttpVersionP,
    HttpVersionSlash,
    HttpVersionMajorStart,
    HttpVersionMajor,
    HttpVersionMinorStart,
    HttpVersionMinor,
    NewLine1,
    HeaderLineStart,
    HeaderName,
    SpaceBeforeHeaderValue,
    HeaderValue,
    NewLine2,
    NewLine3,
}

fn is_tspecial(c: char) -> bool {
    match c {
        '(' | ')' | '<' | '>' | '@' | ',' | ';' | ':' | '\\' | '"' | '/' | '[' | ']' | '?'
        | '=' | '{' | '}' | ' ' | '\t' => true,
        _ => false,
    }
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

    fn consume(&mut self, request: &mut HttpRequest, c: char) -> ParseResult {
        match self.state {
            ParserState::MethodStart => {
                if !c.is_ascii_alphabetic() {
                    return ParseResult::Bad;
                }

                request.method.push(c);
                self.state = ParserState::Method;
                return ParseResult::Indeterminate;
            }
            ParserState::Method => {
                if c == ' ' {
                    self.state = ParserState::Uri;
                    return ParseResult::Indeterminate;
                }

                if !c.is_ascii_alphabetic() {
                    return ParseResult::Bad;
                }

                request.method.push(c);
                self.state = ParserState::Method;
                return ParseResult::Indeterminate;
            }
            ParserState::Uri => {
                if c == ' ' {
                    self.state = ParserState::HttpVersionH;
                    return ParseResult::Indeterminate;
                }

                if c.is_ascii_control() {
                    return ParseResult::Bad;
                }

                request.uri.push(c);
                return ParseResult::Indeterminate;
            }
            ParserState::HttpVersionH => {
                if c == 'H' {
                    self.state = ParserState::HttpVersionT1;
                    return ParseResult::Indeterminate;
                }

                return ParseResult::Bad;
            }
            ParserState::HttpVersionT1 => {
                if c == 'T' {
                    self.state = ParserState::HttpVersionT2;
                    return ParseResult::Indeterminate;
                }

                return ParseResult::Bad;
            }
            ParserState::HttpVersionT2 => {
                if c == 'T' {
                    self.state = ParserState::HttpVersionP;
                    return ParseResult::Indeterminate;
                }

                return ParseResult::Bad;
            }
            ParserState::HttpVersionP => {
                if c == 'P' {
                    self.state = ParserState::HttpVersionSlash;
                    return ParseResult::Indeterminate;
                }

                return ParseResult::Bad;
            }
            ParserState::HttpVersionSlash => {
                if c == '/' {
                    request.version_major = 0;
                    request.version_minor = 0;
                    self.state = ParserState::HttpVersionMajorStart;
                    return ParseResult::Indeterminate;
                }

                return ParseResult::Bad;
            }
            ParserState::HttpVersionMajorStart => {
                if !c.is_ascii_digit() {
                    return ParseResult::Bad;
                }

                request.version_major = c.to_digit(10).unwrap() as u8;
                self.state = ParserState::HttpVersionMajor;
                return ParseResult::Indeterminate;
            }
            ParserState::HttpVersionMajor => {
                if c == '.' {
                    self.state = ParserState::HttpVersionMinorStart;
                    return ParseResult::Indeterminate;
                }

                if !c.is_ascii_digit() {
                    return ParseResult::Bad;
                }

                request.version_major = request.version_major * 10 + c.to_digit(10).unwrap() as u8;
                return ParseResult::Indeterminate;
            }
            ParserState::HttpVersionMinorStart => {
                if !c.is_ascii_digit() {
                    return ParseResult::Bad;
                }

                request.version_minor = c.to_digit(10).unwrap() as u8;
                self.state = ParserState::HttpVersionMinor;
                return ParseResult::Indeterminate;
            }
            ParserState::HttpVersionMinor => {
                if c == '\r' {
                    self.state = ParserState::NewLine1;
                    return ParseResult::Indeterminate;
                }

                if !c.is_ascii_digit() {
                    return ParseResult::Bad;
                }

                request.version_minor = request.version_minor * 10 + c.to_digit(10).unwrap() as u8;
                return ParseResult::Indeterminate;
            }
            ParserState::NewLine1 => {
                if c != '\n' {
                    return ParseResult::Bad;
                }

                self.state = ParserState::HeaderLineStart;
                return ParseResult::Indeterminate;
            }
            ParserState::HeaderLineStart => {
                if c == '\r' {
                    self.state = ParserState::NewLine3;
                    return ParseResult::Indeterminate;
                }

                if !c.is_ascii_graphic() || c.is_ascii_control() || is_tspecial(c) {
                    return ParseResult::Bad;
                }

                request.headers.push(HttpHeader::default());
                request.headers.last_mut().unwrap().name.push(c);
                self.state = ParserState::HeaderName;
                return ParseResult::Indeterminate;
            }
            ParserState::HeaderName => {
                if c == ':' {
                    self.state = ParserState::SpaceBeforeHeaderValue;
                    return ParseResult::Indeterminate;
                }

                if !c.is_ascii_graphic() || c.is_ascii_control() || is_tspecial(c) {
                    return ParseResult::Bad;
                }

                request.headers.last_mut().unwrap().name.push(c);
                return ParseResult::Indeterminate;
            }
            ParserState::SpaceBeforeHeaderValue => {
                if c == ' ' {
                    self.state = ParserState::HeaderValue;
                    return ParseResult::Indeterminate;
                }

                return ParseResult::Indeterminate;
            }
            ParserState::HeaderValue => {
                if c == '\r' {
                    self.state = ParserState::NewLine2;
                    return ParseResult::Indeterminate;
                }

                if c.is_ascii_control() {
                    return ParseResult::Bad;
                }

                request.headers.last_mut().unwrap().value.push(c);
                return ParseResult::Indeterminate;
            }
            ParserState::NewLine2 => {
                if c == '\n' {
                    self.state = ParserState::HeaderLineStart;
                    return ParseResult::Indeterminate;
                }

                return ParseResult::Bad;
            }
            ParserState::NewLine3 => {
                if c == '\n' {
                    return ParseResult::Ok;
                }
                return ParseResult::Bad;
            }
        }
    }

    pub fn parse(
        &mut self,
        request: &mut HttpRequest,
        input: impl Iterator<Item = char>,
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
    fn test_parse_method_indeterminate() {
        let mut parser = HttpRequestParser::new();
        let mut request = HttpRequest::default();

        let input = "GET ";
        let result = parser.parse(&mut request, input.chars());

        assert!(matches!(result, ParseResult::Indeterminate));
        assert!(matches!(request.method.as_str(), "GET"));
    }

    #[test]
    fn test_parse_method_uri_indeterminate() {
        let mut parser = HttpRequestParser::new();
        let mut request = HttpRequest::default();

        let input = "POST /local ";
        let result = parser.parse(&mut request, input.chars());

        assert!(matches!(result, ParseResult::Indeterminate));
        assert!(matches!(request.method.as_str(), "POST"));
        assert!(matches!(request.uri.as_str(), "/local"));
    }

    #[test]
    fn test_parse_whole_request_line() {
        let mut parser = HttpRequestParser::new();
        let mut request = HttpRequest::default();

        let input = "POST /localhost HTTP/1.1\r\n\r\n";
        let result = parser.parse(&mut request, input.chars());

        assert!(matches!(result, ParseResult::Ok));
        assert!(matches!(request.method.as_str(), "POST"));
        assert!(matches!(request.uri.as_str(), "/localhost"));
        assert!(matches!(request.version_major, 1));
        assert!(matches!(request.version_minor, 1));
    }

    #[test]
    fn test_parse_request_line_with_header() {
        let mut parser = HttpRequestParser::new();
        let mut request = HttpRequest::default();

        let input = "POST /localhost HTTP/1.1\r\nContent-Length: 37\r\n\r\n";
        let result = parser.parse(&mut request, input.chars());

        assert!(matches!(result, ParseResult::Ok));
        assert!(matches!(request.method.as_str(), "POST"));
        assert!(matches!(request.uri.as_str(), "/localhost"));
        assert!(matches!(request.version_major, 1));
        assert!(matches!(request.version_minor, 1));
        println!("headers size: {}", request.headers.len());
        assert!(matches!(request.headers.len(), 1));
    }
}
