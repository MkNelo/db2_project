use actix::Actor;
use actix::Context;
use actix::Handler;
use actix::Recipient;

use super::InvokeRequest;

pub struct WebViewApiNode<F> {
    next: Recipient<InvokeRequest>,
    handler: F,
}

impl<F> Actor for WebViewApiNode<F>
where
    F: Unpin + 'static,
{
    type Context = Context<Self>;
}

impl<F> Handler<InvokeRequest> for WebViewApiNode<F>
where
    F: Unpin + Fn(InvokeRequest) + 'static,
{
    type Result = ();

    fn handle(&mut self, msg: InvokeRequest, ctx: &mut Self::Context) -> Self::Result {
        todo!()
    }
}
