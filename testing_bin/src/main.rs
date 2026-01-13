// use std::net::TcpListener;
// use zero::http::response::Response;
use std::collections::HashMap;
use zero::http::{
    request::Method,
    routing::{Query, ResponseResult, Router},
    server::HttpServer,
};
use zero::serializer::FromMap;
use zero::{Deserialize, html};

// async_main!(() -> Result<(), ZeroErr> {
//     // let listener = TcpListener::bind("127.0.0.1:8000").map_err(|e| ZeroErr::FailedToOpen)?;
//     //

//     Ok(())
// });
pub struct Usize {
    inner: usize,
}

#[derive(Deserialize)]
pub struct TestStruct<T> {
    some: Vec<T>,
}

impl FromMap for Usize {
    fn from_map(map: HashMap<String, String>) -> Result<Self, ()> {
        eprintln!("hit inner parse");
        let i = map.get("inner").map(|i| Ok(i)).unwrap_or(Err(()))?;

        let inner = usize::from_str_radix(i, 10).map_err(|_| ())?;

        Ok(Self { inner })
    }
}

pub async fn content(Query(i): Query<Usize>) -> ResponseResult {
    let i = (i.inner + 1).to_string();

    Ok(html! {
        BUTTON(
            id:"output",
            fx-action:(format!("/content?inner={}",i)),
            fx-method:"get",
            fx-trigger:"click",
            fx-target:"#output",
            fx-swap:"outerHTML",
        ){
            (i.into())
        }
    }
    .into())
}

pub async fn index() -> ResponseResult {
    let i = 0.to_string();
    Ok(html! {
        BUTTON(
            id:"output",
            fx-action:"/content?inner=0",
            fx-method:"get",
            fx-trigger:"click",
            fx-target:"#output",
            fx-swap:"outerHTML",
        ){
            (i.into())
        }
        SCRIPT(
            src:"/zero.js"
        ){}
    }
    .into())
}

#[zero::main]
async fn main() -> Result<(), i32> {
    let router = Router::new(())
        .route(Method::Get, "/", index)
        .route(Method::Get, "/content", content)
        .include_zero_js();

    let mut server = HttpServer::from_router(router);

    let serve = server.serve("127.0.0.1:8000").await;
    Err(1)
}
