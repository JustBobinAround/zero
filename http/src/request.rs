use std::io::{Read, Seek};
use zero::parsing::{Parsable, ParseErr, ParseResult, Parser};

use crate::uri::URI;
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

impl<R: Read + Seek> Parsable<R> for RequestMethod {
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

impl<R: Read + Seek> Parsable<R> for HTTPVersion {
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
    fn from_extension_header(eh: MessageHeader) -> Self {
        let val = eh.value;
        let name = eh.name.as_str();
        match name {
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
    fn from_extension_header(eh: MessageHeader) -> Self {
        let val = eh.value;
        let name = eh.name.as_str();
        match name {
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
        }
    }
}

/// Based on rfc2616 Section 14
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum EntityHeader {
    Allow(String),           // Section 14.7
    ContentEncoding(String), // Section 14.11
    ContentLanguage(String), // Section 14.12
    ContentLength(String),   // Section 14.13
    ContentLocation(String), // Section 14.14
    ContentMD5(String),      // Section 14.15
    ContentRange(String),    // Section 14.16
    ContentType(String),     // Section 14.17
    Expires(String),         // Section 14.21
    LastModified(String),    // Section 14.29
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
    fn from_extension_header(eh: MessageHeader) -> Self {
        let val = eh.value;
        let name = eh.name.as_str();
        match name {
            "Allow" => Self::Allow(val),
            "Content-Encoding" => Self::ContentEncoding(val),
            "Content-Language" => Self::ContentLanguage(val),
            "Content-Length" => Self::ContentLength(val),
            "Content-Location" => Self::ContentLocation(val),
            "Content-MD5" => Self::ContentMD5(val),
            "Content-Range" => Self::ContentRange(val),
            "Content-Type" => Self::ContentType(val),
            "Expires" => Self::Expires(val),
            "Last-Modified" => Self::LastModified(val),
            _ => unreachable!(
                "Failed to convert extension header. Perhaps can_convert was not checked"
            ),
        }
    }
}

pub trait FromMessageHeader: Sized {
    fn can_convert(eh: &MessageHeader) -> bool;
    fn from_extension_header(eh: MessageHeader) -> Self;
}

/// Based on rfc2616 Section 4.2
pub struct MessageHeader {
    name: String,
    value: String,
}

impl MessageHeader {
    pub fn into_header<T: FromMessageHeader>(self) -> T {
        T::from_extension_header(self)
    }
}

impl<R: Read + Seek> Parsable<R> for MessageHeader {
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
                .consume_while(|p| !p.matches(|c| c == b'\n'))
                .as_str(),
        );

        while let Some(c) = parser.peek()
            && c == b'\n'
        {
            parser.consume();
            if parser.is_linear_whitespace() {
                parser.skip_whitespace();
                parts.push_str(
                    parser
                        .consume_while(|p| !p.matches(|c| c == b'\n'))
                        .as_str(),
                );
            }
        }

        Ok(MessageHeader { name, value: parts })
    }
}

pub enum Header {
    GeneralHeader(GeneralHeader),
    RequestHeader(RequestHeader),
    EntityHeader(EntityHeader),
    ExtensionHeader(MessageHeader),
}

impl<R: Read + Seek> Parsable<R> for Header {
    fn parse(parser: &mut Parser<R>) -> ParseResult<Self> {
        let header = MessageHeader::parse(parser)?;
        if GeneralHeader::can_convert(&header) {
            Ok(Self::GeneralHeader(header.into_header()))
        } else if RequestHeader::can_convert(&header) {
            Ok(Self::RequestHeader(header.into_header()))
        } else if EntityHeader::can_convert(&header) {
            Ok(Self::EntityHeader(header.into_header()))
        } else {
            Ok(Self::ExtensionHeader(header))
        }
    }
}

pub struct Request {
    method: RequestMethod,
    uri: URI,
    http_version: HTTPVersion,
}

impl<R: Read + Seek> Parsable<R> for Request {
    fn parse(parser: &mut Parser<R>) -> ParseResult<Self> {
        let method = RequestMethod::parse(parser)?;
        parser.skip_whitespace();
        let uri = URI::parse(parser)?;
        parser.skip_whitespace();
        let http_version = HTTPVersion::parse(parser)?;
        parser.skip_whitespace();
        parser.expect_crlf()?;

        Ok(Request {
            method,
            uri,
            http_version,
        })
    }
}
