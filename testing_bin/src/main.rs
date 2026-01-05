use std::net::TcpListener;
use zero::errors::ZeroErr;
use zero::http::{request::Request, response::Response};
use zero::{
    parsing::{Parsable, StreamParser},
    stream_writer::StreamWritable,
};

fn main() -> Result<(), ZeroErr> {
    let listener = TcpListener::bind("127.0.0.1:8000").map_err(|e| ZeroErr::FailedToOpen)?;

    for mut stream in listener.incoming() {
        let mut stream = stream.unwrap();
        let test_response = Response::test_response();
    }

    Ok(())
}
