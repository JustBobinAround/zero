use super::{
    HTTPVersion,
    request::{Method, Request, RequestBody, RequestHeaders},
    response::Response,
    uri::{PathType, RequestQuery, URIPath},
};
use crate::{
    http::request::RequestHeaderType,
    parsing::{StrParser, prelude::*},
};
use std::{
    collections::{HashMap, HashSet},
    io::Read,
    sync::Arc,
};

#[derive(Debug, PartialEq, Eq)]
pub struct InstanceRequest<T> {
    instance: Arc<T>,
    method: Method,
    path: URIPath,
    query: RequestQuery,
    http_version: HTTPVersion,
    headers: RequestHeaders,
    body: RequestBody,
}

impl<T> InstanceRequest<T> {
    pub fn from_request(instance: Arc<T>, r: Request) -> Self {
        InstanceRequest {
            instance,
            method: r.method,
            path: r.path,
            query: r.query,
            http_version: r.http_version,
            headers: r.headers,
            body: r.body,
        }
    }
}

use std::{future::Future, marker::PhantomData, pin::Pin};
type ResponseResult = Result<(), ()>;

struct Endpoint<F, A> {
    f: F,
    _marker: PhantomData<A>,
}

impl<F, A> Endpoint<F, A> {
    fn new(f: F) -> Self {
        Self {
            f,
            _marker: PhantomData,
        }
    }
}

pub trait Handler<A, T> {
    type Fn: Send + Sync + 'static;

    fn into_endpoint(self) -> Arc<dyn FromRequest<T>>;
}

// [x] method: Method,
// [x] path: URIPath,
// [x] query: RequestQuery,
// [x] http_version: HTTPVersion,
// [x] headers: RequestHeaders,
// [ ] body: RequestBody,
//
// TODO: T should be restricted to Parsable
//
pub type Instance<T> = Arc<T>;
//rust was being dumb, idk
pub struct Path<T>(T);
pub trait ToPath: Sized {
    fn into_path(path: URIPath) -> Result<Path<Self>, ()>;
}
impl ToPath for String {
    fn into_path(path: URIPath) -> Result<Path<Self>, ()> {
        Ok(Path(path.into_entire_path()))
    }
}
impl ToPath for Vec<String> {
    fn into_path(path: URIPath) -> Result<Path<Self>, ()> {
        Ok(Path(path.into_segments()))
    }
}
impl ToPath for HashSet<String> {
    fn into_path(path: URIPath) -> Result<Path<Self>, ()> {
        Ok(Path(path.into_segments().into_iter().map(|s| s).collect()))
    }
}
pub struct Query<T: ToQuery>(T);
pub trait ToQuery: Sized {
    fn into_query(query: RequestQuery) -> Result<Query<Self>, ()>;
}
impl ToQuery for String {
    fn into_query(query: RequestQuery) -> Result<Query<Self>, ()> {
        Ok(Query(query.to_string()))
    }
}
impl ToQuery for HashMap<String, String> {
    fn into_query(query: RequestQuery) -> Result<Query<Self>, ()> {
        Ok(Query(query.parameters))
    }
}
pub struct Body<T: ToBody>(T);
pub trait ToBody: Sized {
    fn into_body(body: RequestBody) -> Result<Body<Self>, ()>;
}
impl ToBody for String {
    fn into_body(body: RequestBody) -> Result<Body<Self>, ()> {
        Ok(Body(body))
    }
}
impl ToBody for HashMap<String, String> {
    fn into_body(body: RequestBody) -> Result<Body<Self>, ()> {
        let mut parser = StrParser::from_str(body.as_str());
        let query = RequestQuery::parse(&mut parser).map_err(|_| ())?;
        Ok(Body(query.parameters))
    }
}
pub trait Extract<T, A, B>: Sized {
    fn from_request(instance: PhantomData<T>, parts: A) -> Result<Self, ()>;
}

impl<T> Extract<T, Instance<T>, Instance<T>> for Instance<T> {
    fn from_request(instance: PhantomData<T>, req: Instance<T>) -> Result<Self, ()> {
        Ok(req)
    }
}

impl<T> Extract<T, Method, Method> for Method {
    fn from_request(instance: PhantomData<T>, req: Method) -> Result<Self, ()> {
        Ok(req)
    }
}

impl<T, A: ToPath> Extract<T, URIPath, URIPath> for Path<A> {
    fn from_request(instance: PhantomData<T>, path: URIPath) -> Result<Self, ()> {
        A::into_path(path)
    }
}

impl<T, A: ToQuery> Extract<T, RequestQuery, RequestQuery> for Query<A> {
    fn from_request(instance: PhantomData<T>, query: RequestQuery) -> Result<Self, ()> {
        A::into_query(query)
    }
}

impl<T> Extract<T, HTTPVersion, HTTPVersion> for HTTPVersion {
    fn from_request(instance: PhantomData<T>, version: HTTPVersion) -> Result<Self, ()> {
        Ok(version)
    }
}

impl<T> Extract<T, RequestHeaders, RequestHeaders> for RequestHeaders {
    fn from_request(instance: PhantomData<T>, headers: RequestHeaders) -> Result<Self, ()> {
        Ok(headers)
    }
}

impl<T, A: ToBody> Extract<T, RequestBody, RequestBody> for Body<A> {
    fn from_request(_instance: PhantomData<T>, body: RequestBody) -> Result<Self, ()> {
        A::into_body(body)
    }
}

macros::impl_extract_permutations!();

type BoxFuture = Pin<Box<dyn Future<Output = ResponseResult> + Send>>;

pub trait FromRequest<T> {
    fn apply_request(&self, req: InstanceRequest<T>) -> Result<BoxFuture, ()>;
}

impl<FF, Fut, A, T> FromRequest<T> for Endpoint<FF, (A,)>
where
    FF: Fn(A) -> Fut + Send + Sync + 'static,
    (A,): Extract<T, InstanceRequest<T>, (A,)> + 'static,
    Fut: Future<Output = ResponseResult> + Send + 'static,
{
    fn apply_request(&self, req: InstanceRequest<T>) -> Result<BoxFuture, ()> {
        let (a,) = <(A,)>::from_request(PhantomData, req)?;
        Ok(Box::pin((&self.f)(a)))
    }
}
impl<FF, Fut, A, B, T> FromRequest<T> for Endpoint<FF, (A, B)>
where
    FF: Fn(A, B) -> Fut + Send + Sync + 'static,
    (A, B): Extract<T, InstanceRequest<T>, (A, B)> + 'static,
    Fut: Future<Output = ResponseResult> + Send + 'static,
{
    fn apply_request(&self, req: InstanceRequest<T>) -> Result<BoxFuture, ()> {
        let (a, b) = <(A, B)>::from_request(PhantomData, req)?;
        Ok(Box::pin((self.f)(a, b)))
    }
}

impl<FF, Fut, A, T> Handler<(A,), T> for FF
where
    A: Extract<T, A, A>,
    FF: Fn(A) -> Fut + Send + Sync + 'static,
    (A,): Extract<T, InstanceRequest<T>, (A,)> + 'static,
    Fut: Future<Output = ResponseResult> + Send + 'static,
{
    type Fn = FF;

    fn into_endpoint(self) -> Arc<dyn FromRequest<T>> {
        Arc::new(Endpoint::new(self))
    }
}

impl<FF, Fut, A, B, T> Handler<(A, B), T> for FF
where
    FF: Fn(A, B) -> Fut + Send + Sync + 'static,
    (A, B): Extract<T, InstanceRequest<T>, (A, B)> + 'static,
    Fut: Future<Output = ResponseResult> + Send + 'static,
{
    type Fn = FF;

    fn into_endpoint(self) -> Arc<dyn FromRequest<T>> {
        Arc::new(Endpoint::new(self))
    }
}

pub struct Router<T> {
    instance: Arc<T>,
    routes: HashMap<(&'static Method, &'static str), Arc<dyn FromRequest<T>>>,
}

impl<T> Router<T> {
    pub fn new(instance: T) -> Self {
        Router {
            instance: instance.into(),
            routes: HashMap::new(),
        }
    }

    const OPTIONS: &'static Method = &Method::Options;
    const GET: &'static Method = &Method::Get;
    const HEAD: &'static Method = &Method::Head;
    const POST: &'static Method = &Method::Post;
    const PUT: &'static Method = &Method::Put;
    const DELETE: &'static Method = &Method::Delete;
    const TRACE: &'static Method = &Method::Trace;
    const CONNECT: &'static Method = &Method::Connect;

    pub fn route<A>(mut self, method: Method, s: &'static str, f: impl Handler<A, T>) -> Self {
        let m = match method {
            Method::Options => Self::OPTIONS,
            Method::Get => Self::GET,
            Method::Head => Self::HEAD,
            Method::Post => Self::POST,
            Method::Put => Self::PUT,
            Method::Delete => Self::DELETE,
            Method::Trace => Self::TRACE,
            Method::Connect => Self::CONNECT,
        };
        self.routes.insert((m, s), f.into_endpoint());
        self
    }

    pub async fn apply_request(&self, req: Request) -> ResponseResult {
        let handle = match self.routes.get(&req.method_path()) {
            Some(handle) => handle.clone(),
            None => return Err(()),
        };

        let req = InstanceRequest::from_request(self.instance.clone(), req);

        handle.apply_request(req)?.await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_router() {
        // async fn body_handler(Body(s): Body<String>) -> Result<(), ()> {
        //     dbg!(s);
        //     Ok(())
        // }

        async fn method_handler(method: Method) -> Result<(), ()> {
            dbg!(method);
            Ok(())
        }
        async fn method_handler2(instance: Arc<usize>) -> Result<(), ()> {
            dbg!(instance);
            Ok(())
        }
        // async fn query_handler(Path(s): Path<String>) -> Result<(), ()> {
        //     dbg!(s);
        //     Ok(())
        // }

        // async fn qb_handler(Path(s): Path<String>, Body(b): Body<String>) -> Result<(), ()> {
        //     dbg!(s);
        //     dbg!(b);
        //     Ok(())
        // }
        let router = Router::new(1_usize)
            .route(Method::Get, "some_route", method_handler)
            .route(Method::Get, "some_route2", method_handler2);

        let mut parser = StrParser::from_str(
            "GET /some_route?some=query HTTP/1.1\r\nHost: 127.0.0.1:8000\r\nUser-Agent: curl/8.14.1\r\nContent-Length: 14\r\nAccept: */*\r\n\r\nthis is a test    ",
        );
        let req = Request::parse(&mut parser).unwrap();

        // router.apply_request(req);
        let i = InstanceRequest::from_request(0.into(), req);

        // assert!(false);
    }
}
