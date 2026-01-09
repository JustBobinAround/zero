// use std::net::TcpListener;
// use zero::http::response::Response;
use zero::http::{
    request::Method,
    routing::{ResponseResult, Router},
    server::HttpServer,
};

// async_main!(() -> Result<(), ZeroErr> {
//     // let listener = TcpListener::bind("127.0.0.1:8000").map_err(|e| ZeroErr::FailedToOpen)?;
//     //

//     Ok(())
// });

pub async fn index() -> ResponseResult {
    Err("this is a test".into())
}
#[zero::main]
async fn main() -> Result<(), ()> {
    let router = Router::new(()).route(Method::Get, "/", index);

    let mut server = HttpServer::from_router(router);

    let serve = server.serve("127.0.0.1:8000").await;
    // zero::html!(
    //     p (
    //         a: "asdf",
    //         b: asdf,
    //     ){
    //         lkj
    //     }
    // );

    Ok(())
}
