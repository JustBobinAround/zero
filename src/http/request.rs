use super::uri::{Path, Query, URI};
use crate::parsing::prelude::*;
use std::{
    collections::HashMap,
    io::{Read, Seek},
};
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

/// Based on rfc2616 Section 3.1
///
/// # Augmented Backus-Naur Form
/// ```text
/// HTTP-Version   = "HTTP" "/" 1*DIGIT "." 1*DIGIT
/// ```
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct HTTPVersion {
    major: u8,
    minor: u8,
}

impl<R: Read> Parsable<R> for HTTPVersion {
    fn parse(parser: &mut Parser<R>) -> ParseResult<Self> {
        parser.skip_whitespace();
        parser.expect_str("HTTP/")?;
        let major_str = parser.consume_while(|p| p.is_digit());
        let major =
            u8::from_str_radix(major_str.as_str(), 10).map_err(|_| ParseErr::FailedToParseNum {
                found: major_str,
                radix: 10,
            })?;
        parser.consume_or_err(|c| c == b'.')?;
        let minor_str = parser.consume_while(|p| p.is_digit());
        let minor =
            u8::from_str_radix(minor_str.as_str(), 10).map_err(|_| ParseErr::FailedToParseNum {
                found: minor_str,
                radix: 10,
            })?;

        Ok(HTTPVersion { major, minor })
    }
}

/// Based on rfc2616 Section 14
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum GeneralHeader {
    CacheControl(String),     // Section 14.9
    Connection(String),       // Section 14.10
    Date(String),             // Section 14.18
    Pragma(String),           // Section 14.32
    Trailer(String),          // Section 14.40
    TransferEncoding(String), // Section 14.41
    Upgrade(String),          // Section 14.42
    Via(String),              // Section 14.45
    Warning(String),          // Section 14.46
}

impl GeneralHeader {
    pub const fn name(&self) -> &'static str {
        match self {
            GeneralHeader::CacheControl(_) => "Cache-Control", // Section 14.9
            GeneralHeader::Connection(_) => "Connection",      // Section 14.10
            GeneralHeader::Date(_) => "Date",                  // Section 14.18
            GeneralHeader::Pragma(_) => "Pragma",              // Section 14.32
            GeneralHeader::Trailer(_) => "Trailer",            // Section 14.40
            GeneralHeader::TransferEncoding(_) => "Transfer-Encoding", // Section 14.41
            GeneralHeader::Upgrade(_) => "Upgrade",            // Section 14.42
            GeneralHeader::Via(_) => "Via",                    // Section 14.45
            GeneralHeader::Warning(_) => "Warning",            // Section 14.46
        }
    }
}

impl FromMessageHeader for GeneralHeader {
    fn can_convert(eh: &MessageHeader) -> bool {
        let name = eh.name.as_str();
        name == "Cache-Control"
            || name == "Connection"
            || name == "Date"
            || name == "Pragma"
            || name == "Trailer"
            || name == "Transfer-Encoding"
            || name == "Upgrade"
            || name == "Via"
            || name == "Warning"
    }

    fn from_extension_header(eh: MessageHeader) -> ParseResult<(String, Self)> {
        let val = eh.value;
        let name = eh.name.as_str();
        let header = match name {
            "Cache-Control" => Self::CacheControl(val),
            "Connection" => Self::Connection(val),
            "Date" => Self::Date(val),
            "Pragma" => Self::Pragma(val),
            "Trailer" => Self::Trailer(val),
            "Transfer-Encoding" => Self::TransferEncoding(val),
            "Upgrade" => Self::Upgrade(val),
            "Via" => Self::Via(val),
            "Warning" => Self::Warning(val),
            _ => unreachable!(
                "Failed to convert extension header. Perhaps can_convert was not checked"
            ),
        };

        Ok((eh.name, header))
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
            Self::Accept(_) => "Accept",
            Self::AcceptCharset(_) => "Accept-Charset",
            Self::AcceptEncoding(_) => "Accept-Encoding",
            Self::AcceptLanguage(_) => "Accept-Language",
            Self::Authorization(_) => "Authorization",
            Self::Expect(_) => "Expect",
            Self::From(_) => "From",
            Self::Host(_) => "Host",
            Self::IfMatch(_) => "If-Match",
            Self::IfModifiedSince(_) => "If-Modified-Since",
            Self::IfNoneMatch(_) => "If-None-Match",
            Self::IfRange(_) => "If-Range",
            Self::IfUnmodifiedSince(_) => "If-Unmodified-Since",
            Self::MaxForwards(_) => "Max-Forwards",
            Self::ProxyAuthorization(_) => "Proxy-Authorization",
            Self::Range(_) => "Range",
            Self::Referer(_) => "Referer",
            Self::TE(_) => "TE",
            Self::UserAgent(_) => "User-Agent",
        }
    }
}

impl FromMessageHeader for RequestHeader {
    fn can_convert(eh: &MessageHeader) -> bool {
        let name = eh.name.as_str();
        name == "Accept"
            || name == "Accept-Charset"
            || name == "Accept-Encoding"
            || name == "Accept-Language"
            || name == "Authorization"
            || name == "Expect"
            || name == "From"
            || name == "Host"
            || name == "If-Match"
            || name == "If-Modified-Since"
            || name == "If-None-Match"
            || name == "If-Range"
            || name == "If-Unmodified-Since"
            || name == "Max-Forwards"
            || name == "Proxy-Authorization"
            || name == "Range"
            || name == "Referer"
            || name == "TE"
            || name == "User-Agent"
    }
    fn from_extension_header(eh: MessageHeader) -> ParseResult<(String, Self)> {
        let val = eh.value;
        let name = eh.name.as_str();
        let header = match name {
            "Accept" => Self::Accept(val),
            "Accept-Charset" => Self::AcceptCharset(val),
            "Accept-Encoding" => Self::AcceptEncoding(val),
            "Accept-Language" => Self::AcceptLanguage(val),
            "Authorization" => Self::Authorization(val),
            "Expect" => Self::Expect(val),
            "From" => Self::From(val),
            "Host" => Self::Host(val),
            "If-Match" => Self::IfMatch(val),
            "If-Modified-Since" => Self::IfModifiedSince(val),
            "If-None-Match" => Self::IfNoneMatch(val),
            "If-Range" => Self::IfRange(val),
            "If-Unmodified-Since" => Self::IfUnmodifiedSince(val),
            "Max-Forwards" => Self::MaxForwards(val),
            "Proxy-Authorization" => Self::ProxyAuthorization(val),
            "Range" => Self::Range(val),
            "Referer" => Self::Referer(val),
            "TE" => Self::TE(val),
            "User-Agent" => Self::UserAgent(val),
            _ => unreachable!(
                "Failed to convert extension header. Perhaps can_convert was not checked"
            ),
        };

        Ok((eh.name, header))
    }
}

/// Based on rfc2616 Section 14
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum EntityHeader {
    Allow(String),           // Section 14.7
    ContentEncoding(String), // Section 14.11
    ContentLanguage(String), // Section 14.12
    ContentLength(usize),    // Section 14.13
    ContentLocation(String), // Section 14.14
    ContentMD5(String),      // Section 14.15
    ContentRange(String),    // Section 14.16
    ContentType(String),     // Section 14.17
    Expires(String),         // Section 14.21
    LastModified(String),    // Section 14.29
}

impl EntityHeader {
    pub const fn name(&self) -> &'static str {
        match self {
            EntityHeader::Allow(_) => "Allow", // Section 14.7
            EntityHeader::ContentEncoding(_) => "Content-Encoding", // Section 14.11
            EntityHeader::ContentLanguage(_) => "Content-Language", // Section 14.12
            EntityHeader::ContentLength(_) => "Content-Length", // Section 14.13
            EntityHeader::ContentLocation(_) => "Content-Location", // Section 14.14
            EntityHeader::ContentMD5(_) => "Content-MD5", // Section 14.15
            EntityHeader::ContentRange(_) => "Content-Range", // Section 14.16
            EntityHeader::ContentType(_) => "Content-Type", // Section 14.17
            EntityHeader::Expires(_) => "Expires", // Section 14.21
            EntityHeader::LastModified(_) => "Last-Modified", // Section 14.29
        }
    }
}

impl FromMessageHeader for EntityHeader {
    fn can_convert(eh: &MessageHeader) -> bool {
        let name = eh.name.as_str();
        name == "Allow"
            || name == "Content-Encoding"
            || name == "Content-Language"
            || name == "Content-Length"
            || name == "Content-Location"
            || name == "Content-MD5"
            || name == "Content-Range"
            || name == "Content-Type"
            || name == "Expires"
            || name == "Last-Modified"
    }
    fn from_extension_header(eh: MessageHeader) -> ParseResult<(String, Self)> {
        let val = eh.value;
        let name = eh.name.as_str();
        let header = match name {
            "Allow" => Self::Allow(val),
            "Content-Encoding" => Self::ContentEncoding(val),
            "Content-Language" => Self::ContentLanguage(val),
            "Content-Length" => {
                let length = usize::from_str_radix(val.as_str(), 10).map_err(|_| {
                    ParseErr::FailedToParseNum {
                        found: val,
                        radix: 10,
                    }
                })?;
                Self::ContentLength(length)
            }
            "Content-Location" => Self::ContentLocation(val),
            "Content-MD5" => Self::ContentMD5(val),
            "Content-Range" => Self::ContentRange(val),
            "Content-Type" => Self::ContentType(val),
            "Expires" => Self::Expires(val),
            "Last-Modified" => Self::LastModified(val),
            _ => unreachable!(
                "Failed to convert extension header. Perhaps can_convert was not checked"
            ),
        };

        Ok((eh.name, header))
    }
}

pub trait FromMessageHeader: Sized {
    fn can_convert(eh: &MessageHeader) -> bool;
    fn from_extension_header(eh: MessageHeader) -> ParseResult<(String, Self)>;
}

/// Based on rfc2616 Section 4.2
///
/// # Augmented Backus-Naur Form
/// ```text
///       message-header = field-name ":" [ field-value ]
///       field-name     = token
///       field-value    = *( field-content | LWS )
///       field-content  = <the OCTETs making up the field-value
///                        and consisting of either *TEXT or combinations
///                        of token, separators, and quoted-string>
/// ```
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct MessageHeader {
    name: String,
    value: String,
}

impl MessageHeader {
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn extract_name_val(self) -> (String, String) {
        (self.name, self.value)
    }
}

impl MessageHeader {
    pub fn into_header<T: FromMessageHeader>(self) -> ParseResult<(String, T)> {
        T::from_extension_header(self)
    }
}

impl<R: Read> Parsable<R> for MessageHeader {
    fn parse(parser: &mut Parser<R>) -> ParseResult<Self> {
        let name = parser.consume_while(|p| p.is_token_char());
        if name.len() == 0 {
            return Err(ParseErr::BlankHeaderFieldName);
        }
        parser.skip_whitespace();
        parser.consume_or_err(|c| c == b':')?;
        parser.skip_whitespace();
        let mut parts = String::new();
        parts.push_str(
            parser
                .consume_while(|p| !p.matches(|c| c == b'\r'))
                .as_str(),
        );
        parser.consume();

        while let Some(c) = parser.peek()
            && c == b'\n'
        {
            parser.consume();
            if parser.is_linear_whitespace() {
                parser.skip_whitespace();
                parts.push_str(
                    parser
                        .consume_while(|p| !p.matches(|c| c == b'\r'))
                        .as_str(),
                );
                parser.consume();
            } else {
                break;
            }
        }

        Ok(MessageHeader { name, value: parts })
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
    GeneralHeader(GeneralHeader),
    RequestHeader(RequestHeader),
    EntityHeader(EntityHeader),
    ExtensionHeader(String),
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
            String::from("Host"),
            RequestHeaderType::RequestHeader(RequestHeader::Host(String::from("127.0.0.1:8000"))),
        );
        headers.insert(
            String::from("User-Agent"),
            RequestHeaderType::RequestHeader(RequestHeader::UserAgent(String::from("curl/8.14.1"))),
        );
        headers.insert(
            String::from("Accept"),
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
            String::from("Host"),
            RequestHeaderType::RequestHeader(RequestHeader::Host(String::from("127.0.0.1:8000"))),
        );
        headers.insert(
            String::from("User-Agent"),
            RequestHeaderType::RequestHeader(RequestHeader::UserAgent(String::from("curl/8.14.1"))),
        );
        headers.insert(
            String::from("Content-Length"),
            RequestHeaderType::EntityHeader(EntityHeader::ContentLength(14)),
        );
        headers.insert(
            String::from("Accept"),
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
