use crate::parsing::prelude::*;
use std::io::Read;

/// Based on RFC 2616 section 6.1.1
///
/// - **1xx**: Informational - Request received, continuing process
/// - **2xx**: Success - The action was successfully received, understood, and accepted
/// - **3xx**: Redirection - Further action must be taken in order to complete the request
/// - **4xx**: Client Error - The request contains bad syntax or cannot be fulfilled
/// - **5xx**: Server Error - The server failed to fulfill an apparently valid request
///
/// # Augmented Backus-Naur Form
/// ```text
///
/// Status-Code    =
///             "100"  ; Section 10.1.1: Continue
///           | "101"  ; Section 10.1.2: Switching Protocols
///           | "200"  ; Section 10.2.1: OK
///           | "201"  ; Section 10.2.2: Created
///           | "202"  ; Section 10.2.3: Accepted
///           | "203"  ; Section 10.2.4: Non-Authoritative Information
///           | "204"  ; Section 10.2.5: No Content
///           | "205"  ; Section 10.2.6: Reset Content
///           | "206"  ; Section 10.2.7: Partial Content
///           | "300"  ; Section 10.3.1: Multiple Choices
///           | "301"  ; Section 10.3.2: Moved Permanently
///           | "302"  ; Section 10.3.3: Found
///           | "303"  ; Section 10.3.4: See Other
///           | "304"  ; Section 10.3.5: Not Modified
///           | "305"  ; Section 10.3.6: Use Proxy
///           | "307"  ; Section 10.3.8: Temporary Redirect
///           | "400"  ; Section 10.4.1: Bad Request
///           | "401"  ; Section 10.4.2: Unauthorized
///           | "402"  ; Section 10.4.3: Payment Required
///           | "403"  ; Section 10.4.4: Forbidden
///           | "404"  ; Section 10.4.5: Not Found
///           | "405"  ; Section 10.4.6: Method Not Allowed
///           | "406"  ; Section 10.4.7: Not Acceptable
///           | "407"  ; Section 10.4.8: Proxy Authentication Required
///           | "408"  ; Section 10.4.9: Request Time-out
///           | "409"  ; Section 10.4.10: Conflict
///           | "410"  ; Section 10.4.11: Gone
///           | "411"  ; Section 10.4.12: Length Required
///           | "412"  ; Section 10.4.13: Precondition Failed
///           | "413"  ; Section 10.4.14: Request Entity Too Large
///           | "414"  ; Section 10.4.15: Request-URI Too Large
///           | "415"  ; Section 10.4.16: Unsupported Media Type
///           | "416"  ; Section 10.4.17: Requested range not satisfiable
///           | "417"  ; Section 10.4.18: Expectation Failed
///           | "500"  ; Section 10.5.1: Internal Server Error
///           | "501"  ; Section 10.5.2: Not Implemented
///           | "502"  ; Section 10.5.3: Bad Gateway
///           | "503"  ; Section 10.5.4: Service Unavailable
///           | "504"  ; Section 10.5.5: Gateway Time-out
///           | "505"  ; Section 10.5.6: HTTP Version not supported
///           | extension-code
///
///       extension-code = 3DIGIT
/// ```
pub enum StatusCode {
    Continue,                     // "100"  ; Section 10.1.1:
    SwitchingProtocols,           // "101"  ; Section 10.1.2:
    OK,                           // "200"  ; Section 10.2.1:
    Created,                      // "201"  ; Section 10.2.2:
    Accepted,                     // "202"  ; Section 10.2.3:
    NonAuthoritativeInformation,  // "203"  ; Section 10.2.4:
    NoContent,                    // "204"  ; Section 10.2.5:
    ResetContent,                 // "205"  ; Section 10.2.6:
    PartialContent,               // "206"  ; Section 10.2.7:
    MultipleChoices,              // "300"  ; Section 10.3.1:
    MovedPermanently,             // "301"  ; Section 10.3.2:
    Found,                        // "302"  ; Section 10.3.3:
    SeeOther,                     // "303"  ; Section 10.3.4:
    NotModified,                  // "304"  ; Section 10.3.5:
    UseProxy,                     // "305"  ; Section 10.3.6:
    TemporaryRedirect,            // "307"  ; Section 10.3.8:
    BadRequest,                   // "400"  ; Section 10.4.1:
    Unauthorized,                 // "401"  ; Section 10.4.2:
    PaymentRequired,              // "402"  ; Section 10.4.3:
    Forbidden,                    // "403"  ; Section 10.4.4:
    NotFound,                     // "404"  ; Section 10.4.5:
    MethodNotAllowed,             // "405"  ; Section 10.4.6:
    NotAcceptable,                // "406"  ; Section 10.4.7:
    ProxyAuthenticationRequired,  // "407"  ; Section 10.4.8:
    RequestTimeout,               // "408"  ; Section 10.4.9:
    Conflict,                     // "409"  ; Section 10.4.10:
    Gone,                         // "410"  ; Section 10.4.11:
    LengthRequired,               // "411"  ; Section 10.4.12:
    PreconditionFailed,           // "412"  ; Section 10.4.13:
    RequestEntityTooLarge,        // "413"  ; Section 10.4.14:
    RequestUriTooLarge,           // "414"  ; Section 10.4.15:
    UnsupportedMediaType,         // "415"  ; Section 10.4.16:
    RequestedRangeNotSatisfiable, // "416"  ; Section 10.4.17:
    ExpectationFailed,            // "417"  ; Section 10.4.18:
    InternalServerError,          // "500"  ; Section 10.5.1:
    NotImplemented,               // "501"  ; Section 10.5.2:
    BadGateway,                   // "502"  ; Section 10.5.3:
    ServiceUnavailable,           // "503"  ; Section 10.5.4:
    GatewayTimeout,               // "504"  ; Section 10.5.5:
    HTTPVersionNotSupported,      // "505"  ; Section 10.5.6:
    ExtensionCode(u16),
}

impl StatusCode {
    pub const fn from_code(n: u16) -> Result<Self, ParseErr> {
        match n {
            100 => Ok(Self::Continue),                     // "100"  ; Section 10.1.1:
            101 => Ok(Self::SwitchingProtocols),           // "101"  ; Section 10.1.2:
            200 => Ok(Self::OK),                           // "200"  ; Section 10.2.1:
            201 => Ok(Self::Created),                      // "201"  ; Section 10.2.2:
            202 => Ok(Self::Accepted),                     // "202"  ; Section 10.2.3:
            203 => Ok(Self::NonAuthoritativeInformation),  // "203"  ; Section 10.2.4:
            204 => Ok(Self::NoContent),                    // "204"  ; Section 10.2.5:
            205 => Ok(Self::ResetContent),                 // "205"  ; Section 10.2.6:
            206 => Ok(Self::PartialContent),               // "206"  ; Section 10.2.7:
            300 => Ok(Self::MultipleChoices),              // "300"  ; Section 10.3.1:
            301 => Ok(Self::MovedPermanently),             // "301"  ; Section 10.3.2:
            302 => Ok(Self::Found),                        // "302"  ; Section 10.3.3:
            303 => Ok(Self::SeeOther),                     // "303"  ; Section 10.3.4:
            304 => Ok(Self::NotModified),                  // "304"  ; Section 10.3.5:
            305 => Ok(Self::UseProxy),                     // "305"  ; Section 10.3.6:
            307 => Ok(Self::TemporaryRedirect),            // "307"  ; Section 10.3.8:
            400 => Ok(Self::BadRequest),                   // "400"  ; Section 10.4.1:
            401 => Ok(Self::Unauthorized),                 // "401"  ; Section 10.4.2:
            402 => Ok(Self::PaymentRequired),              // "402"  ; Section 10.4.3:
            403 => Ok(Self::Forbidden),                    // "403"  ; Section 10.4.4:
            404 => Ok(Self::NotFound),                     // "404"  ; Section 10.4.5:
            405 => Ok(Self::MethodNotAllowed),             // "405"  ; Section 10.4.6:
            406 => Ok(Self::NotAcceptable),                // "406"  ; Section 10.4.7:
            407 => Ok(Self::ProxyAuthenticationRequired),  // "407"  ; Section 10.4.8:
            408 => Ok(Self::RequestTimeout),               // "408"  ; Section 10.4.9:
            409 => Ok(Self::Conflict),                     // "409"  ; Section 10.4.10:
            410 => Ok(Self::Gone),                         // "410"  ; Section 10.4.11:
            411 => Ok(Self::LengthRequired),               // "411"  ; Section 10.4.12:
            412 => Ok(Self::PreconditionFailed),           // "412"  ; Section 10.4.13:
            413 => Ok(Self::RequestEntityTooLarge),        // "413"  ; Section 10.4.14:
            414 => Ok(Self::RequestUriTooLarge),           // "414"  ; Section 10.4.15:
            415 => Ok(Self::UnsupportedMediaType),         // "415"  ; Section 10.4.16:
            416 => Ok(Self::RequestedRangeNotSatisfiable), // "416"  ; Section 10.4.17:
            417 => Ok(Self::ExpectationFailed),            // "417"  ; Section 10.4.18:
            500 => Ok(Self::InternalServerError),          // "500"  ; Section 10.5.1:
            501 => Ok(Self::NotImplemented),               // "501"  ; Section 10.5.2:
            502 => Ok(Self::BadGateway),                   // "502"  ; Section 10.5.3:
            503 => Ok(Self::ServiceUnavailable),           // "503"  ; Section 10.5.4:
            504 => Ok(Self::GatewayTimeout),               // "504"  ; Section 10.5.5:
            505 => Ok(Self::HTTPVersionNotSupported),      // "505"  ; Section 10.5.6:
            n if n < 1000 => {
                Ok(Self::ExtensionCode(n)) // this should technically be under 999
            }
            n => {
                Err(ParseErr::InvalidStatusCode { found: n }) // this should technically be under 999
            }
        }
    }
}
impl<R: Read> Parsable<R> for StatusCode {
    fn parse(parser: &mut Parser<R>) -> ParseResult<Self> {
        let mut num = [0; 3];
        for i in 0..3 {
            match parser.peek() {
                Some(c) => {
                    if c.is_ascii_digit() {
                        parser.consume();
                        num[i] = c;
                    } else {
                        return Err(ParseErr::InvalidStatusCodeStr { found: c });
                    }
                }
                None => return Err(ParseErr::ExpectedStatusCode),
            }
        }

        match &num {
            b"100" => Ok(Self::Continue),           // "100"  ; Section 10.1.1:
            b"101" => Ok(Self::SwitchingProtocols), // "101"  ; Section 10.1.2:
            b"200" => Ok(Self::OK),                 // "200"  ; Section 10.2.1:
            b"201" => Ok(Self::Created),            // "201"  ; Section 10.2.2:
            b"202" => Ok(Self::Accepted),           // "202"  ; Section 10.2.3:
            b"203" => Ok(Self::NonAuthoritativeInformation), // "203"  ; Section 10.2.4:
            b"204" => Ok(Self::NoContent),          // "204"  ; Section 10.2.5:
            b"205" => Ok(Self::ResetContent),       // "205"  ; Section 10.2.6:
            b"206" => Ok(Self::PartialContent),     // "206"  ; Section 10.2.7:
            b"300" => Ok(Self::MultipleChoices),    // "300"  ; Section 10.3.1:
            b"301" => Ok(Self::MovedPermanently),   // "301"  ; Section 10.3.2:
            b"302" => Ok(Self::Found),              // "302"  ; Section 10.3.3:
            b"303" => Ok(Self::SeeOther),           // "303"  ; Section 10.3.4:
            b"304" => Ok(Self::NotModified),        // "304"  ; Section 10.3.5:
            b"305" => Ok(Self::UseProxy),           // "305"  ; Section 10.3.6:
            b"307" => Ok(Self::TemporaryRedirect),  // "307"  ; Section 10.3.8:
            b"400" => Ok(Self::BadRequest),         // "400"  ; Section 10.4.1:
            b"401" => Ok(Self::Unauthorized),       // "401"  ; Section 10.4.2:
            b"402" => Ok(Self::PaymentRequired),    // "402"  ; Section 10.4.3:
            b"403" => Ok(Self::Forbidden),          // "403"  ; Section 10.4.4:
            b"404" => Ok(Self::NotFound),           // "404"  ; Section 10.4.5:
            b"405" => Ok(Self::MethodNotAllowed),   // "405"  ; Section 10.4.6:
            b"406" => Ok(Self::NotAcceptable),      // "406"  ; Section 10.4.7:
            b"407" => Ok(Self::ProxyAuthenticationRequired), // "407"  ; Section 10.4.8:
            b"408" => Ok(Self::RequestTimeout),     // "408"  ; Section 10.4.9:
            b"409" => Ok(Self::Conflict),           // "409"  ; Section 10.4.10:
            b"410" => Ok(Self::Gone),               // "410"  ; Section 10.4.11:
            b"411" => Ok(Self::LengthRequired),     // "411"  ; Section 10.4.12:
            b"412" => Ok(Self::PreconditionFailed), // "412"  ; Section 10.4.13:
            b"413" => Ok(Self::RequestEntityTooLarge), // "413"  ; Section 10.4.14:
            b"414" => Ok(Self::RequestUriTooLarge), // "414"  ; Section 10.4.15:
            b"415" => Ok(Self::UnsupportedMediaType), // "415"  ; Section 10.4.16:
            b"416" => Ok(Self::RequestedRangeNotSatisfiable), // "416"  ; Section 10.4.17:
            b"417" => Ok(Self::ExpectationFailed),  // "417"  ; Section 10.4.18:
            b"500" => Ok(Self::InternalServerError), // "500"  ; Section 10.5.1:
            b"501" => Ok(Self::NotImplemented),     // "501"  ; Section 10.5.2:
            b"502" => Ok(Self::BadGateway),         // "502"  ; Section 10.5.3:
            b"503" => Ok(Self::ServiceUnavailable), // "503"  ; Section 10.5.4:
            b"504" => Ok(Self::GatewayTimeout),     // "504"  ; Section 10.5.5:
            b"505" => Ok(Self::HTTPVersionNotSupported), // "505"  ; Section 10.5.6:
            n => {
                match u16::from_str_radix(str::from_utf8(n).map_err(|_| ParseErr::InvalidUTF8)?, 10)
                {
                    Ok(n) => Ok(Self::ExtensionCode(n)),
                    Err(_) => Err(ParseErr::ExpectedStatusCode),
                }
            }
        }
    }
}

/// Based on RFC 2616 section 6.1
///
/// # Augmented Backus-Naur Form
/// ```text
/// Status-Line = HTTP-Version SP Status-Code SP Reason-Phrase CRLF
/// ```
pub struct StatusLine {
    status_code: StatusCode,
}

impl<R: Read> Parsable<R> for StatusLine {
    fn parse(parser: &mut Parser<R>) -> ParseResult<Self> {
        let status_code = StatusCode::parse(parser)?;

        Ok(StatusLine { status_code })
    }
}

/// Based on RFC 2616 section 6
///
/// # Augmented Backus-Naur Form
/// ```text
///      Response       = Status-Line               ; Section 6.1
///                       *(( general-header        ; Section 4.5
///                        | response-header        ; Section 6.2
///                        | entity-header ) CRLF)  ; Section 7.1
///                       CRLF
///                       [ message-body ]          ; Section 7.2
/// ```
pub struct Response {}
