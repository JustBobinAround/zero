// use std::net::TcpListener;
// use zero::http::response::Response;
use zero::http::{
    Body, Query,
    request::Method,
    routing::{ResponseResult, Router},
    server::HttpServer,
};
use zero::{Deserialize, html};

#[derive(Deserialize, Debug)]
pub struct Usize {
    inner: usize,
    inner2: String,
}

#[derive(Deserialize, Debug)]
pub struct Demo {
    foo: String,
}

pub struct TestStruct<'a, T> {
    some: &'a Vec<T>,
}

pub async fn content(Query(i): Query<Usize>) -> ResponseResult {
    let i = (i.inner + 1).to_string();
    // let inner = i.inner2;

    Ok(html! {
        BUTTON(
            id:"output",
            fx-action:(format!("/content?inner={}&inner2=3",i)),
            fx-method:"get",
            fx-trigger:"click",
            fx-target:"#output",
            fx-swap:"outerHTML",
        ){ (i) }
    }
    .into())
}

pub async fn index() -> ResponseResult {
    Ok(html! {
        BUTTON(
            id:"output",
            fx-action:"/content?inner=0&inner2=test",
            fx-method:"get",
            fx-trigger:"click",
            fx-target:"#output",
            fx-swap:"outerHTML",
        ){ "0" }
        FORM(fx-action:"/demo", fx-trigger:"submit", fx-method:"post"){
            INPUT(
                type:"text",
                name:"foo",
            ){}
            INPUT(
                type:"text",
                name:"bar",
            ){}
            BUTTON(type:"submit", value:"Submit"){
                "submit"
            }
        }
        SCRIPT( src:"/zero.js" )
    }
    .into())
}

// pub async fn demo(Body(s): Body<Demo>) -> ResponseResult { eprintln!("{:#?}", s);
//     Ok(html! {}.into())
// }

#[zero::main]
async fn main() -> Result<(), i32> {
    let router = Router::new(())
        .route(Method::Get, "/", index)
        .route(Method::Get, "/content", content)
        // .route(Method::Post, "/demo", demo)
        .include_zero_js();

    let mut server = HttpServer::from_router(router);

    let serve = server.serve("127.0.0.1:8000").await;
    Err(1)
}
