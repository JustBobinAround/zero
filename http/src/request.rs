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

pub enum GeneralHeader {
    CacheControl(String),
    Connection(String),
    Date(String),
    Pragma(String),
    Trailer(String),
    TransferEncoding(String),
    Upgrade(String),
    Via(String),
    Warning(String),
}

pub enum RequestHeader {}

pub enum Header {
    GeneralHeader(GeneralHeader),
    RequestHeader,
    EntityHeader,
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
