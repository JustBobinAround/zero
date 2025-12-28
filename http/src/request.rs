use std::io::{Read, Seek};
use zero::parsing::{Parsable, ParseErr, ParseResult, Parser};
// GET / HTTP/1.1
// Host: 127.0.0.1:42069
// User-Agent: curl/8.14.1
// Accept: */*

pub struct VersionedProtocol {
    name: String,
    version: String,
}

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
            _ => Err(ParseErr::InvalidRequestOption),
        }
    }
}

pub struct URI {
    path: String,
    query: String,
}

impl<R: Read + Seek> Parsable<R> for URI {
    fn parse(parser: &mut Parser<R>) -> ParseResult<Self> {
        parser.skip_whitespace();
        while let Some(c) = parser.peek() {
            parser.consume();
        }
        unimplemented!()
    }
}

pub struct Request {
    method: RequestMethod,
    uri: URI,
    // http_version:
    host: String,
    user_agent: String,
    // accept:
}

impl Request {}

impl<R: Read + Seek> Parsable<R> for Request {
    fn parse(parser: &mut Parser<R>) -> ParseResult<Self> {
        let method = RequestMethod::parse(parser)?;
        unimplemented!()
    }
}
