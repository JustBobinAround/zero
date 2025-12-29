use std::io::BufReader;
use std::io::prelude::*;
use std::net::TcpListener;
use zero::errors::ZeroErr;

fn main() -> Result<(), ZeroErr> {
    let listener = TcpListener::bind("127.0.0.1:8000").map_err(|e| ZeroErr::FailedToOpen)?;

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let mut lines = BufReader::new(stream).lines();

        while let Some(Ok(line)) = lines.next() {
            println!("{}", line);
        }
    }

    Ok(())
}
