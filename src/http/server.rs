use std::{net::TcpListener, thread::JoinHandle};

use crate::{errors::ZeroErr, http::request::Request, parsing::StreamParser};

use super::routing::Router;
use std::pin::Pin;
use std::sync::Arc;

pub struct HttpServer<T: Send + Sync + 'static> {
    router: Arc<Router<T>>,
    handles: Vec<JoinHandle<Task>>,
}

type Task = Pin<Box<dyn Future<Output = ()> + Send>>;

impl<T: Send + Sync> HttpServer<T> {
    pub fn from_router(router: Router<T>) -> Self {
        HttpServer {
            router: router.into(),
            handles: Vec::new(),
        }
    }
    pub async fn serve<IP>(&mut self, ip: IP) -> Result<(), ZeroErr>
    where
        IP: std::fmt::Display,
    {
        let listener = TcpListener::bind(ip.to_string()).map_err(|e| ZeroErr::FailedToOpen)?;

        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let router = self.router.clone();
                    match Request::parse_from_stream(&mut stream) {
                        Ok(request) => {
                            router.apply_request(request).await;
                        }
                        Err(_) => {}
                    }
                }
                Err(e) => eprintln!("connection failed: {}", e),
            }
        }

        Ok(())
    }
}
