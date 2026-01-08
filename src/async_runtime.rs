// use std::net::TcpListener;
// use zero::http::response::Response;
use std::{
    future::Future,
    ptr,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

type WakerData = *const ();
static ZERO_ASYNC_VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop);

pub trait Runtime: Sized {
    type Output;
    fn run_async(self) -> Self::Output;
    fn fetch_raw_waker() -> RawWaker {
        RawWaker::new(ptr::null(), &ZERO_ASYNC_VTABLE)
    }
    fn fetch_waker() -> Waker;
}

unsafe fn clone(_: WakerData) -> RawWaker {
    RawWaker::new(ptr::null(), &ZERO_ASYNC_VTABLE)
}
unsafe fn wake(_: WakerData) {}
unsafe fn wake_by_ref(_: WakerData) {}
unsafe fn drop(_: WakerData) {}

impl<F> Runtime for F
where
    F: Future,
{
    type Output = F::Output;
    fn run_async(self) -> Self::Output {
        let waker = Self::fetch_waker();
        let mut context = Context::from_waker(&waker);

        let mut t = Box::pin(self);
        let t = t.as_mut();

        loop {
            match t.poll(&mut context) {
                Poll::Ready(v) => return v,
                Poll::Pending => {
                    unimplemented!("pending futures")
                }
            }
        }
    }

    fn fetch_raw_waker() -> RawWaker {
        RawWaker::new(ptr::null(), &ZERO_ASYNC_VTABLE)
    }
    fn fetch_waker() -> Waker {
        unsafe { Waker::from_raw(Self::fetch_raw_waker()) }
    }
}
