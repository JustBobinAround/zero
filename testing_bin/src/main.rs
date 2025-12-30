use std::net::TcpListener;
use zero::errors::ZeroErr;
use zero::http::request::Request;
use zero::parsing::{Parsable, StreamParser};

fn main() -> Result<(), ZeroErr> {
    let listener = TcpListener::bind("127.0.0.1:8000").map_err(|e| ZeroErr::FailedToOpen)?;

    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        dbg!(Request::parse_from_stream(&mut stream));
    }

    Ok(())
}
