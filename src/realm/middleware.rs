use std::marker::PhantomData;
use std::sync::{Arc, Mutex};

pub trait Middleware {
    type Message;
    type Response;

    fn manage(
        &self,
        data: Self::Message,
        callback: &dyn Fn(Self::Message) -> Self::Response,
    ) -> Self::Response;
}

pub type BoxedMiddleware<Msg, Rsp> = Box<dyn Middleware<Message = Msg, Response = Rsp>>;

impl<Msg, Rsp> Middleware for BoxedMiddleware<Msg, Rsp> {
    type Message = Msg;
    type Response = Rsp;

    fn manage(
        &self,
        data: Self::Message,
        callback: &dyn Fn(Self::Message) -> Self::Response,
    ) -> Self::Response {
        self.as_ref().manage(data, callback)
    }
}

pub trait LoadMiddleware<Mid: Middleware> {
    type Result;
    fn load_middleware(self, middleware: Mid) -> Self::Result;
}

pub struct AdHocMid<F, M, R>(F, PhantomData<M>, PhantomData<R>);

pub struct AdHocFor<Key, Mid, F>(Mid, Vec<Key>, F);

impl<Key, Mid, F> Middleware for AdHocFor<Key, Mid, F>
where
    Key: PartialEq,
    Mid: Middleware,
    F: Fn(&Mid::Message) -> &Key,
{
    type Message = Mid::Message;

    type Response = Mid::Response;

    fn manage(
        &self,
        data: Self::Message,
        callback: &dyn Fn(Self::Message) -> Self::Response,
    ) -> Self::Response {
        let key = self.2(&data);
        if self.1.iter().any(|akey| akey == key) {
            self.0.manage(data, callback)
        } else {
            callback(data)
        }
    }
}

pub trait InvokeFor<Keys>: Middleware + Sized
where
    Keys: PartialEq,
{
    fn invoke_for<F: Fn(&Self::Message) -> &Keys>(
        self,
        keys: Vec<Keys>,
        lense: F,
    ) -> AdHocFor<Keys, Self, F> {
        AdHocFor(self, keys, lense)
    }
}

impl<S, Keys> InvokeFor<Keys> for S
where
    S: Middleware,
    Keys: PartialEq,
{
}

impl<F, M, R> Middleware for AdHocMid<F, M, R>
where
    F: Fn(M, &dyn Fn(M) -> R) -> R,
{
    type Message = M;

    type Response = R;

    fn manage(&self, data: Self::Message, callback: &dyn Fn(M) -> R) -> Self::Response {
        self.0(data, callback)
    }
}

pub struct Compose<Mid, Mid1>(Mid, Mid1);

impl<Mid, Mid1> Middleware for Compose<Mid, Mid1>
where
    Mid: Middleware,
    Mid1: Middleware<Message = Mid::Message, Response = Mid::Response>,
{
    type Message = Mid::Message;

    type Response = Mid::Response;

    fn manage(
        &self,
        data: Self::Message,
        callback: &dyn Fn(Self::Message) -> Self::Response,
    ) -> Self::Response {
        let mid = &self.0;
        let mid1 = &self.1;

        mid.manage(data, &|ib| mid1.manage(ib, callback))
    }
}

pub fn compose<Mid, Mid1>(m: Mid, m1: Mid1) -> Compose<Mid, Mid1>
where
    Mid: Middleware,
    Mid1: Middleware<Message = Mid::Message, Response = Mid::Response>,
{
    Compose(m, m1)
}

pub fn adhoc<M, R, F: Fn(M, &dyn Fn(M) -> R) -> R + Send + 'static>(f: F) -> AdHocMid<F, M, R> {
    AdHocMid(f, PhantomData, PhantomData)
}

pub fn none<M, R>() -> AdHocMid<impl Fn(M, &dyn Fn(M) -> R) -> R, M, R> {
    AdHocMid(
        |ib: M, cb: &dyn Fn(M) -> R| cb(ib),
        PhantomData,
        PhantomData,
    )
}
