use super::{
    EntityHeader, FromMessageHeader, GeneralHeader, HTTPVersion, MessageHeader,
    uri::{Path, Query},
};
use crate::parsing::prelude::*;
use std::{collections::HashMap, io::Read};
// GET / HTTP/1.1
// Host: 127.0.0.1:42069
// User-Agent: curl/8.14.1
// Accept: */*

/// Based on rfc2616 Section 5.1.1
///
/// # Augmented Backus-Naur Form
/// ```text
/// Method         = "OPTIONS"                ; Section 9.2
///   | "GET"                    ; Section 9.3
///   | "HEAD"                   ; Section 9.4
///   | "POST"                   ; Section 9.5
///   | "PUT"                    ; Section 9.6
///   | "DELETE"                 ; Section 9.7
///   | "TRACE"                  ; Section 9.8
///   | "CONNECT"                ; Section 9.9
///   | extension-method
///
/// extension-method = token
/// ```
///
/// Not supporting extension methods for now
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RequestMethod {
    Options,
    Get,
    Head,
    Post,
    Put,
    Delete,
    Trace,
    Connect,
}

impl<R: Read> Parsable<R> for RequestMethod {
    fn parse(parser: &mut Parser<R>) -> ParseResult<Self> {
        parser.skip_whitespace();
        let token = parser.consume_while(|p| p.is_alpha());
        match token.as_str() {
            "OPTIONS" => Ok(RequestMethod::Options),
            "GET" => Ok(RequestMethod::Get),
            "HEAD" => Ok(RequestMethod::Head),
            "POST" => Ok(RequestMethod::Post),
            "PUT" => Ok(RequestMethod::Put),
            "DELETE" => Ok(RequestMethod::Delete),
            "TRACE" => Ok(RequestMethod::Trace),
            "CONNECT" => Ok(RequestMethod::Connect),
            _ => Err(ParseErr::InvalidRequestOption { found: token }),
        }
    }
}

/// Based on rfc2616 Section 14
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RequestHeader {
    Accept(String),             // Section 14.1
    AcceptCharset(String),      // Section 14.2
    AcceptEncoding(String),     // Section 14.3
    AcceptLanguage(String),     // Section 14.4
    Authorization(String),      // Section 14.8
    Expect(String),             // Section 14.20
    From(String),               // Section 14.22
    Host(String),               // Section 14.23
    IfMatch(String),            // Section 14.24
    IfModifiedSince(String),    // Section 14.25
    IfNoneMatch(String),        // Section 14.26
    IfRange(String),            // Section 14.27
    IfUnmodifiedSince(String),  // Section 14.28
    MaxForwards(String),        // Section 14.31
    ProxyAuthorization(String), // Section 14.34
    Range(String),              // Section 14.35
    Referer(String),            // Section 14.36
    TE(String),                 // Section 14.39
    UserAgent(String),          // Section 14.43
}

impl RequestHeader {
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Accept(_) => "accept",
            Self::AcceptCharset(_) => "accept-charset",
            Self::AcceptEncoding(_) => "accept-encoding",
            Self::AcceptLanguage(_) => "accept-language",
            Self::Authorization(_) => "authorization",
            Self::Expect(_) => "expect",
            Self::From(_) => "from",
            Self::Host(_) => "host",
            Self::IfMatch(_) => "if-match",
            Self::IfModifiedSince(_) => "if-modified-since",
            Self::IfNoneMatch(_) => "if-none-match",
            Self::IfRange(_) => "if-range",
            Self::IfUnmodifiedSince(_) => "if-unmodified-since",
            Self::MaxForwards(_) => "max-forwards",
            Self::ProxyAuthorization(_) => "proxy-authorization",
            Self::Range(_) => "range",
            Self::Referer(_) => "referer",
            Self::TE(_) => "te",
            Self::UserAgent(_) => "user-agent",
        }
    }
}

impl FromMessageHeader for RequestHeader {
    fn can_convert(eh: &MessageHeader) -> bool {
        let name = eh.name.as_str();
        name == "accept"
            || name == "accept-charset"
            || name == "accept-encoding"
            || name == "accept-language"
            || name == "authorization"
            || name == "expect"
            || name == "from"
            || name == "host"
            || name == "if-match"
            || name == "if-modified-since"
            || name == "if-none-match"
            || name == "if-range"
            || name == "if-unmodified-since"
            || name == "max-forwards"
            || name == "proxy-authorization"
            || name == "range"
            || name == "referer"
            || name == "te"
            || name == "user-agent"
    }
    fn from_extension_header(eh: MessageHeader) -> ParseResult<(String, Self)> {
        let val = eh.value;
        let name = eh.name.as_str();
        let header = match name {
            "accept" => Self::Accept(val),
            "accept-charset" => Self::AcceptCharset(val),
            "accept-encoding" => Self::AcceptEncoding(val),
            "accept-language" => Self::AcceptLanguage(val),
            "authorization" => Self::Authorization(val),
            "expect" => Self::Expect(val),
            "from" => Self::From(val),
            "host" => Self::Host(val),
            "if-match" => Self::IfMatch(val),
            "if-modified-since" => Self::IfModifiedSince(val),
            "if-none-match" => Self::IfNoneMatch(val),
            "if-range" => Self::IfRange(val),
            "if-unmodified-since" => Self::IfUnmodifiedSince(val),
            "max-forwards" => Self::MaxForwards(val),
            "proxy-authorization" => Self::ProxyAuthorization(val),
            "range" => Self::Range(val),
            "referer" => Self::Referer(val),
            "te" => Self::TE(val),
            "user-agent" => Self::UserAgent(val),
            _ => unreachable!(
                "Failed to convert extension header. Perhaps can_convert was not checked"
            ),
        };

        Ok((eh.name, header))
    }
}

/// Based on RFC 2616 section 5
///
/// # Augmented Backus-Naur Form
/// ```text
///        Request       = Request-Line              ; Section 5.1
///                        *(( general-header        ; Section 4.5
///                         | request-header         ; Section 5.3
///                         | entity-header ) CRLF)  ; Section 7.1
///                        CRLF
///                        [ message-body ]          ; Section 4.3
/// ```
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RequestHeaderType {
    EntityHeader(EntityHeader),
    ExtensionHeader(String),
    GeneralHeader(GeneralHeader),
    RequestHeader(RequestHeader),
}

/// Abstraction used to take ownership of name to be held in header hashmap
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RequestHeaderMap {
    name: String,
    ty: RequestHeaderType,
}

impl RequestHeaderMap {
    pub fn extract_name_type(self) -> (String, RequestHeaderType) {
        (self.name, self.ty)
    }
}

impl<R: Read> Parsable<R> for RequestHeaderMap {
    fn parse(parser: &mut Parser<R>) -> ParseResult<Self> {
        let header = MessageHeader::parse(parser);
        dbg!(&header);
        let header = header?;
        if GeneralHeader::can_convert(&header) {
            let (name, header) = header.into_header()?;
            Ok(Self {
                name,
                ty: RequestHeaderType::GeneralHeader(header),
            })
        } else if RequestHeader::can_convert(&header) {
            let (name, header) = header.into_header()?;
            Ok(Self {
                name,
                ty: RequestHeaderType::RequestHeader(header),
            })
        } else if EntityHeader::can_convert(&header) {
            let (name, header) = header.into_header()?;
            Ok(Self {
                name,
                ty: RequestHeaderType::EntityHeader(header),
            })
        } else {
            let (name, value) = header.extract_name_val();
            Ok(Self {
                name,
                ty: RequestHeaderType::ExtensionHeader(value),
            })
        }
    }
}

/// Based on RFC 2616 section 5
///
/// # Augmented Backus-Naur Form
/// ```text
///        Request       = Request-Line              ; Section 5.1
///                        *(( general-header        ; Section 4.5
///                         | request-header         ; Section 5.3
///                         | entity-header ) CRLF)  ; Section 7.1
///                        CRLF
///                        [ message-body ]          ; Section 4.3
/// ```
#[derive(Debug, PartialEq, Eq)]
pub struct Request {
    method: RequestMethod,
    path: Path,
    query: Option<Query>,
    http_version: HTTPVersion,
    headers: HashMap<String, RequestHeaderType>,
    body: Option<String>,
}

impl<R: Read> Parsable<R> for Request {
    fn parse(parser: &mut Parser<R>) -> ParseResult<Self> {
        let method = RequestMethod::parse(parser)?;
        parser.skip_whitespace();
        let path = Path::parse(parser)?;
        let query = if parser.matches(|c| c == b'?') {
            Some(Query::parse(parser)?)
        } else {
            None
        };
        parser.skip_whitespace();
        let http_version = HTTPVersion::parse(parser)?;
        parser.skip_whitespace();
        parser.expect_crlf()?;

        let mut headers = HashMap::new();
        let mut body_len = None;

        while let Ok(header) = RequestHeaderMap::parse(parser) {
            let (name, ty) = header.extract_name_type();
            match ty {
                RequestHeaderType::EntityHeader(EntityHeader::ContentLength(len)) => {
                    body_len = Some(len)
                }
                _ => {}
            }
            headers.insert(name, ty);
        }

        let body = match body_len {
            Some(body_len) => {
                parser.expect_crlf()?;
                // eprintln!("{}", parser.peek().unwrap() as char);
                // parser.consume_or_err(|c| c == b'\n')?;
                // eprintln!("hit");
                Some(parser.consume_n(body_len))
            }
            None => None,
        };

        Ok(Request {
            method,
            path,
            query,
            http_version,
            headers,
            body,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::parsing::StrParser;

    use super::*;

    #[test]
    fn test_methods() {
        let mut parser = StrParser::from_str("GET");
        assert_eq!(RequestMethod::parse(&mut parser), Ok(RequestMethod::Get));
        let mut parser = StrParser::from_str("POST");
        assert_eq!(RequestMethod::parse(&mut parser), Ok(RequestMethod::Post));
    }

    #[test]
    fn test_http_version() {
        let mut parser = StrParser::from_str("HTTP/1.1");
        assert_eq!(
            HTTPVersion::parse(&mut parser),
            Ok(HTTPVersion { major: 1, minor: 1 })
        );
    }

    #[test]
    fn test_request() {
        let mut parser = StrParser::from_str("/somepath");
        let path = Path::parse(&mut parser).unwrap();
        let mut parser = StrParser::from_str("?some=query");
        let query = Query::parse(&mut parser).unwrap();

        let mut parser = StrParser::from_str(
            "GET /somepath?some=query HTTP/1.1\r\nHost: 127.0.0.1:8000\r\nUser-Agent: curl/8.14.1\r\nAccept: */*",
        );
        let mut headers = HashMap::new();
        headers.insert(
            String::from("host"),
            RequestHeaderType::RequestHeader(RequestHeader::Host(String::from("127.0.0.1:8000"))),
        );
        headers.insert(
            String::from("user-agent"),
            RequestHeaderType::RequestHeader(RequestHeader::UserAgent(String::from("curl/8.14.1"))),
        );
        headers.insert(
            String::from("accept"),
            RequestHeaderType::RequestHeader(RequestHeader::Accept(String::from("*/*"))),
        );
        assert_eq!(
            Request::parse(&mut parser),
            Ok(Request {
                method: RequestMethod::Get,
                path,
                query: Some(query),
                http_version: HTTPVersion { major: 1, minor: 1 },
                headers,
                body: None
            })
        );
    }
    #[test]
    fn test_request_body() {
        let mut parser = StrParser::from_str("/somepath");
        let path = Path::parse(&mut parser).unwrap();
        let mut parser = StrParser::from_str("?some=query");
        let query = Query::parse(&mut parser).unwrap();

        let mut parser = StrParser::from_str(
            "GET /somepath?some=query HTTP/1.1\r\nHost: 127.0.0.1:8000\r\nUser-Agent: curl/8.14.1\r\nContent-Length: 14\r\nAccept: */*\r\n\r\nthis is a test    ",
        );
        let mut headers = HashMap::new();
        headers.insert(
            String::from("host"),
            RequestHeaderType::RequestHeader(RequestHeader::Host(String::from("127.0.0.1:8000"))),
        );
        headers.insert(
            String::from("user-agent"),
            RequestHeaderType::RequestHeader(RequestHeader::UserAgent(String::from("curl/8.14.1"))),
        );
        headers.insert(
            String::from("content-length"),
            RequestHeaderType::EntityHeader(EntityHeader::ContentLength(14)),
        );
        headers.insert(
            String::from("accept"),
            RequestHeaderType::RequestHeader(RequestHeader::Accept(String::from("*/*"))),
        );
        assert_eq!(
            Request::parse(&mut parser),
            Ok(Request {
                method: RequestMethod::Get,
                path,
                query: Some(query),
                http_version: HTTPVersion { major: 1, minor: 1 },
                headers,
                body: Some(String::from("this is a test"))
            })
        );
    }
}
