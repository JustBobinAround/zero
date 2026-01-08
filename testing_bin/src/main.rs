// use std::net::TcpListener;
// use zero::http::response::Response;
use zero::http::{routing::Router, server::HttpServer};

// async_main!(() -> Result<(), ZeroErr> {
//     // let listener = TcpListener::bind("127.0.0.1:8000").map_err(|e| ZeroErr::FailedToOpen)?;
//     //

//     Ok(())
// });

#[zero::main]
async fn main() -> Result<(), ()> {
    let router = Router::new(());

    let mut server = HttpServer::from_router(router);

    let serve = server.serve("127.0.0.1:8000").await;

    Ok(())
}
