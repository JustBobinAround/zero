// use std::net::TcpListener;
// use zero::http::response::Response;
use zero::html;
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

pub async fn content() -> ResponseResult {
    Ok("this is a test".into())
}
pub async fn index() -> ResponseResult {
    Ok(html! {
    BUTTON(
        fx-action:"/content",
            fx-method:"get",
            fx-trigger:"click",
            fx-target:"#output",
            fx-swap:"innerHTML",
    ){
        "Get Content"
    }
    OUTPUT(
        id:"output"
    ){}
    SCRIPT(src:"/zero.js"){}
        }
    .into())
}
#[zero::main]
async fn main() -> Result<(), ()> {
    let router = Router::new(())
        .route(Method::Get, "/", index)
        .route(Method::Get, "/content", content)
        .include_zero_js();

    let mut server = HttpServer::from_router(router);

    let serve = server.serve("127.0.0.1:8000").await;
    Ok(())
}
