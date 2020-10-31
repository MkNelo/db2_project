use super::InvokeRequest;
use super::SpawnerFactory;

use super::super::middleware::adhoc;
use super::super::middleware::Compose;

use super::super::middleware::Middleware;
use super::{super::middleware::LoadMiddleware, ApiResponse, WebViewApp};
use futures::future::BoxFuture;

pub struct WVFMiddleware<Mid>(Mid);

impl<Mid> Middleware for WVFMiddleware<Mid>
where
    Mid: Middleware<Message = InvokeRequest, Response = BoxFuture<'static, ApiResponse>>,
{
    type Message = InvokeRequest;

    type Response = BoxFuture<'static, ApiResponse>;

    fn manage(
        &self,
        data: Self::Message,
        callback: &dyn Fn(Self::Message) -> Self::Response,
    ) -> Self::Response {
        self.0.manage(data, callback)
    }
}

pub fn webview_middleware<Mid>(
    mid: Mid,
) -> WVFMiddleware<
    impl Middleware<Message = InvokeRequest, Response = BoxFuture<'static, ApiResponse>>,
>
where
    Mid: Fn(
            InvokeRequest,
            &dyn Fn(InvokeRequest) -> BoxFuture<'static, ApiResponse>,
        ) -> BoxFuture<'static, ApiResponse>
        + Send
        + 'static,
{
    WVFMiddleware(adhoc(mid))
}

impl<Cont, Mid, OMid, Context> LoadMiddleware<Mid>
    for WebViewApp<'static, Cont, OMid, Context>
where
    OMid: Middleware<Message = InvokeRequest, Response = BoxFuture<'static, ApiResponse>>,
    Mid: Middleware<Message = InvokeRequest, Response = BoxFuture<'static, ApiResponse>> + 'static,
    Cont: AsRef<str>,
    Context: SpawnerFactory,
{
    type Result = WebViewApp<'static, Cont, Compose<OMid, Mid>, Context>;

    fn load_middleware(self, middleware: Mid) -> Self::Result {
        let mid = self.container.push(middleware);
        let mut builder = web_view::builder()
            .title(self.webview_builder.title)
            .size(self.webview_builder.width, self.webview_builder.height)
            .frameless(self.webview_builder.frameless);

        if self.webview_builder.content.is_some() {
            builder = builder.content(self.webview_builder.content.unwrap());
        }

        WebViewApp {
            webview_builder: builder,
            container: mid,
            context: self.context,
            data_container: self.data_container,
        }
    }
}
