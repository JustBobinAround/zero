use std::io::{Read, Seek};
use zero::parsing::{Parsable, ParseErr, ParseResult, Parser};

/// Based on rfc3986 Section 3.1
///
/// # Augmented Backus-Naur Form
/// ```
/// scheme      = ALPHA *( ALPHA / DIGIT / "+" / "-" / "." )
/// ```
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Scheme(String);

impl Scheme {
    pub fn new(s: String) -> Self {
        Scheme(s)
    }
    pub fn from(s: &str) -> Self {
        Scheme(String::from(s))
    }
    pub fn is_valid_char(c: u8) -> bool {
        c.is_ascii_alphanumeric() || c == b'+' || c == b'-' || c == b'.'
    }
}

impl<R: Read + Seek> Parsable<R> for Scheme {
    fn parse(parser: &mut Parser<R>) -> ParseResult<Self> {
        if parser.matches(|c| c.is_ascii_alphabetic()) {
            Ok(Scheme(
                parser.consume_while(|p| p.matches(Self::is_valid_char)),
            ))
        } else {
            Err(ParseErr::InvalidScheme)
        }
    }
}

/// Based on rfc3986 Section 2.1
///
/// # Augmented Backus-Naur Form
/// ```
/// pct-encoded = "%" HEXDIG HEXDIG
/// ```
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PctEncoding(char);
impl PctEncoding {
    pub fn from(c: char) -> Self {
        Self(c)
    }
}

impl<R: Read + Seek> Parsable<R> for PctEncoding {
    fn parse(parser: &mut Parser<R>) -> ParseResult<Self> {
        parser.consume_or_err(|c| c == b'%')?;
        let b1 = parser.consume_or_err(|c| c.is_ascii_hexdigit())?;
        let b2 = parser.consume_or_err(|c| c.is_ascii_hexdigit())?;
        let hex = format!("{}{}", b1 as char, b2 as char);
        if hex.len() == 2 {
            let radix = 16;
            let byte = u8::from_str_radix(hex.as_str(), radix)
                .map_err(|_| ParseErr::FailedToParseNum { found: hex, radix })?;
            Ok(PctEncoding(byte as char))
        } else {
            Err(ParseErr::InvalidPctEncoding { found: hex })
        }
    }
}

/// Based on rfc3986 Section 3.2.1
///
/// # Augmented Backus-Naur Form
/// ```
/// userinfo    = *( unreserved / pct-encoded / sub-delims / ":" )
/// ```
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct UserInfo(String);

impl UserInfo {
    pub fn from(s: &str) -> Self {
        Self(String::from(s))
    }
    fn maybe_parse<R: Read + Seek>(parser: &mut Parser<R>) -> ParseResult<Option<Self>> {
        let user_info = Self::parse(parser)?;
        let info = if user_info.0.len() > 0 {
            Some(user_info)
        } else {
            None
        };

        Ok(info)
    }
}

impl<R: Read + Seek> Parsable<R> for UserInfo {
    fn parse(parser: &mut Parser<R>) -> ParseResult<Self> {
        let mut s = String::new();
        parser.push();

        let mut found_at = false;

        while let Some(c) = parser.peek() {
            if URI::is_unreserved(c) {
                s.push(c as char);
                parser.consume();
            } else if URI::is_sub_delim(c) {
                s.push(c as char);
                parser.consume();
            } else if c == b':' {
                s.push(c as char);
                parser.consume();
            } else if c == b'%' {
                let pct_encoding = PctEncoding::parse(parser)?;
                s.push(pct_encoding.0);
            } else if c == b'@' {
                found_at = true;
                parser.consume();
                parser.pop_no_seek();
                break;
            } else {
                parser.pop()?;
                break;
            }
        }

        if found_at {
            Ok(UserInfo(s))
        } else {
            Ok(UserInfo(String::new()))
        }
    }
}

/// Based on rfc3986 Section 3.2.2
///
/// # Augmented Backus-Naur Form
/// ```
/// userinfo    = *( unreserved / pct-encoded / sub-delims / ":" )
/// ```
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Host {
    IPvFuture(String),
    IPv6(String),
    IPv4((u8, u8, u8, u8)),
    Domain(String),
}

impl Host {
    fn parse_ip_lit<R: Read + Seek>(parser: &mut Parser<R>) -> ParseResult<Self> {
        unimplemented!("Haven't worked on ipv6 yet, sorry")
        // parser.consume_or_err(|c| c == b'[')?;
        // parser.consume_or_err(|c| c == b']')?;
    }

    fn ipv4_from_str(s: &str) -> ParseResult<Self> {
        let mut nums = Vec::new();
        let mut s_num = String::new();
        for c in s.chars() {
            if c == '.' {
                if let Ok(num) = u8::from_str_radix(s_num.as_str(), 10) {
                    nums.push(num);
                    s_num = String::new();
                } else {
                    return Err(ParseErr::InvalidIPv4Num { found: s_num });
                }
            } else if c.is_ascii_digit() {
                s_num.push(c);
            } else {
                return Err(ParseErr::InvalidIPv4Char { found: c });
            }
        }

        if let Ok(num) = u8::from_str_radix(s_num.as_str(), 10) {
            nums.push(num);
        } else {
            return Err(ParseErr::InvalidIPv4Num { found: s_num });
        }

        if nums.len() == 4 {
            Ok(Self::IPv4((nums[0], nums[1], nums[2], nums[3])))
        } else {
            Err(ParseErr::InvalidIPv4Len { found: nums.len() })
        }
    }

    fn parse_ipv4_or_domain<R: Read + Seek>(parser: &mut Parser<R>) -> ParseResult<Self> {
        let mut s = String::new();

        let mut is_ipv4 = true;
        while let Some(c) = parser.peek() {
            if URI::is_unreserved(c) {
                if !c.is_ascii_digit() && c != b'.' {
                    is_ipv4 = false;
                }
                s.push(c as char);
                parser.consume();
            } else if URI::is_sub_delim(c) {
                is_ipv4 = false;
                s.push(c as char);
                parser.consume();
            } else if c == b'%' {
                is_ipv4 = false;
                let pct_encoding = PctEncoding::parse(parser)?;
                s.push(pct_encoding.0);
            } else {
                break;
            }
        }

        if is_ipv4 {
            Self::ipv4_from_str(s.as_str())
        } else {
            Ok(Host::Domain(s))
        }
    }
}

impl<R: Read + Seek> Parsable<R> for Host {
    fn parse(parser: &mut Parser<R>) -> ParseResult<Self> {
        if parser.matches(|c| c == b'[') {
            Self::parse_ip_lit(parser)
        } else {
            Self::parse_ipv4_or_domain(parser)
        }
    }
}

/// Based on rfc3986 Section 3.2.3
///
/// # Augmented Backus-Naur Form
/// ```
/// port        = *DIGIT
/// ```
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Port(u16);

impl<R: Read + Seek> Parsable<R> for Port {
    fn parse(parser: &mut Parser<R>) -> ParseResult<Self> {
        parser.consume_or_err(|c| c == b':')?;
        let port_num_str = parser.consume_while(|p| p.is_digit());
        let radix = 10;
        let port = u16::from_str_radix(port_num_str.as_str(), radix).map_err(|_| {
            ParseErr::FailedToParseNum {
                found: port_num_str,
                radix,
            }
        })?;
        Ok(Port(port))
    }
}

/// Based on rfc3986 Section 3.2
///
/// # Augmented Backus-Naur Form
/// ```
/// authority   = [ userinfo "@" ] host [ ":" port ]
/// ```
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Authority {
    user_info: Option<UserInfo>,
    host: Host,
    port: Option<Port>,
}

impl<R: Read + Seek> Parsable<R> for Authority {
    fn parse(parser: &mut Parser<R>) -> ParseResult<Self> {
        parser.skip_whitespace();
        let user_info = UserInfo::maybe_parse(parser)?;
        let host = Host::parse(parser)?;
        let port = if parser.matches(|c| c == b':') {
            let port = Port::parse(parser)?;
            Some(port)
        } else {
            None
        };

        Ok(Authority {
            user_info,
            host,
            port,
        })
    }
}

/// Based on See rfc3986 - Mainly Section 3
///
/// # Augmented Backus-Naur Form
/// ```text
/// URI         = scheme ":" hier-part [ "?" query ] [ "#" fragment ]
///
/// hier-part   = "//" authority path-abempty
///   / path-absolute
///   / path-rootless
///   / path-empty
/// ```
///
/// # Example
/// ```text
/// foo://example.com:8042/over/there?name=ferret#nose
/// \_/   \______________/\_________/ \_________/ \__/
///  |           |            |            |        |
/// scheme     authority       path        query   fragment
/// ```
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct URI {
    scheme: Scheme,
    authority: Authority,
}

impl URI {
    // pub fn pct_decode() -> u8 {
    //     let mut
    //     u8::from_str_radix("A", 16)
    // }

    pub fn is_unreserved(c: u8) -> bool {
        c.is_ascii_alphanumeric() || c == b'-' || c == b'.' || c == b'_' || c == b'~'
    }
    pub fn is_reserved(c: u8) -> bool {
        Self::is_gen_delim(c) || Self::is_sub_delim(c)
    }

    pub fn is_gen_delim(c: u8) -> bool {
        c == b':' || c == b',' || c == b'?' || c == b'#' || c == b'[' || c == b']' || c == b'@'
    }

    pub fn is_sub_delim(c: u8) -> bool {
        c == b'!'
            || c == b'$'
            || c == b'&'
            || c == b'\''
            || c == b'('
            || c == b')'
            || c == b'*'
            || c == b'+'
            || c == b','
            || c == b';'
            || c == b'='
    }
}

impl<R: Read + Seek> Parsable<R> for URI {
    fn parse(parser: &mut Parser<R>) -> ParseResult<Self> {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use zero::parsing::StrParser;

    use super::*;

    #[test]
    fn test_valid_scheme() {
        let mut parser = StrParser::from_str("https://");
        assert_eq!(Scheme::parse(&mut parser), Ok(Scheme::from("https")));
        let mut parser = StrParser::from_str("http://");
        assert_eq!(Scheme::parse(&mut parser), Ok(Scheme::from("http")));
    }

    #[test]
    fn test_invalid_scheme() {
        let mut parser = StrParser::from_str("1https://");
        assert_eq!(Scheme::parse(&mut parser), Err(ParseErr::InvalidScheme));
    }

    #[test]
    fn test_pct_encoding() {
        let mut parser = StrParser::from_str("%3D");
        assert_eq!(PctEncoding::parse(&mut parser), Ok(PctEncoding::from('=')));
        let mut parser = StrParser::from_str("%3d");
        assert_eq!(PctEncoding::parse(&mut parser), Ok(PctEncoding::from('=')));
        let mut parser = StrParser::from_str("%3J");
        assert_eq!(
            PctEncoding::parse(&mut parser),
            Err(ParseErr::FailedToConsume { found: Some(74) })
        );
    }

    #[test]
    fn test_user_info() {
        let mut parser = StrParser::from_str("someuser@some_domain.com");
        assert_eq!(
            UserInfo::maybe_parse(&mut parser),
            Ok(Some(UserInfo::from("someuser")))
        );
        let mut parser = StrParser::from_str("someusersome_domain.com");
        assert_eq!(UserInfo::maybe_parse(&mut parser), Ok(None));
    }

    #[test]
    fn test_valid_ipv4_host() {
        let mut parser = StrParser::from_str("127.0.0.1");
        assert_eq!(Host::parse(&mut parser), Ok(Host::IPv4((127, 0, 0, 1))));
    }

    #[test]
    fn test_invalid_ipv4_host() {
        let mut parser = StrParser::from_str("257.0.0.1");
        assert_eq!(
            Host::parse(&mut parser),
            Err(ParseErr::InvalidIPv4Num {
                found: String::from("257")
            })
        );
    }

    #[test]
    fn test_valid_domain_host() {
        let mut parser = StrParser::from_str("www.example.com");
        assert_eq!(
            Host::parse(&mut parser),
            Ok(Host::Domain(String::from("www.example.com")))
        );
    }

    #[test]
    fn test_valid_authority() {
        let mut parser = StrParser::from_str("someuser@someemaildomain.com:8000");
        assert_eq!(
            Authority::parse(&mut parser),
            Ok(Authority {
                user_info: Some(UserInfo::from("someuser")),
                host: Host::Domain(String::from("someemaildomain.com")),
                port: Some(Port(8000)),
            })
        );
    }

    #[test]
    fn test_valid_authority_pct() {
        let mut parser = StrParser::from_str("some%3duser@some%3demaildomain.com:8000");
        assert_eq!(
            Authority::parse(&mut parser),
            Ok(Authority {
                user_info: Some(UserInfo::from("some=user")),
                host: Host::Domain(String::from("some=emaildomain.com")),
                port: Some(Port(8000)),
            })
        );
    }
}
