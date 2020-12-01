use std::marker::PhantomData;

extern crate actix;
extern crate actix_rt;
extern crate futures;
extern crate tokio;

pub mod adapters;
pub mod webview;

pub trait Api {
    type Input;
    type Output;

    fn handle(&self, msg: Self::Input) -> Self::Output;
}

pub trait Application {
    type Result;
    fn finish(self) -> Self::Result;
}

pub trait Load<T> {
    type Result;

    fn load(self, api: T) -> Self::Result;
}

pub struct FuncApi<F, T, O>(F, PhantomData<T>, PhantomData<O>);

impl<T, O, F> Api for FuncApi<F, T, O>
where
    F: Fn(T) -> O,
{
    type Input = T;

    type Output = O;

    fn handle(&self, msg: Self::Input) -> Self::Output {
        self.0(msg)
    }
}

pub fn api<F, T, O>(f: F) -> FuncApi<F, T, O> {
    FuncApi(f, PhantomData, PhantomData)
}

pub mod prelude {
    pub use super::adapters::*;
    pub use super::webview::middleware::*;
    pub use super::webview::prelude::*;
    pub use super::webview::*;
    pub use super::*;
    pub use futures::{executor::*, future::ready, future::BoxFuture, prelude::*};
    pub use web_view::Content::{self, *};
}
