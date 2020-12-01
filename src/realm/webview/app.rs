use std::mem::MaybeUninit;
use std::pin::Pin;
use std::sync::Arc;

use actix::prelude::*;
use serde::Serialize;
use web_view::Content;
use web_view::Handle;
use web_view::WVResult;
use web_view::WebView;
use web_view::WebViewBuilder;

use crate::Application;
use crate::Load;

use super::api::WebViewApiFactory;
use super::api::WebViewLoadableActor;
use super::container::WebViewBuilderContainer;
use super::container::WebViewContainer;
use super::error;
use super::AppString;
use super::InvokeBody;
use super::InvokeRequest;
use super::KeyedActor;

macro_rules! impl_extractor {
    ($($name:tt),+) => {
        impl<'a, Cont, Factory, Fut, $($name),+> Load<WebViewApiFactory<Factory,  (Fut, $($name), +) >> for AppBuilder<'a, Cont>
        where
            Factory: Fn($(Addr<$name>),+) -> Fut,
            Fut: Future + Send + 'a,
            <Factory::Output as Future>::Output: KeyedActor,
            Cont: Send + 'a,
        $(
            $name: Actor<Context = Context<$name>> + Send + Sync
        ),+
        {
            type Result = Pin<Box<dyn Future<Output = Self> + 'a>>;

            fn load(self, arg: WebViewApiFactory<Factory, ( Fut, $($name),+ )>) -> Self::Result {
                let ref container = self.container;
                let ref factory = arg.0;
                let container = container.actor_container();
                let api = factory($(container.get::<$name>().unwrap()),+);
                Box::pin(async move { self.load(api.await) })
            }
        }
    }
}

impl_extractor!(A);
impl_extractor!(A, B);
impl_extractor!(A, B, C);
impl_extractor!(A, B, C, D);
impl_extractor!(A, B, C, D, E);

pub struct App {
    handle: Handle<MaybeUninit<Addr<Self>>>,
    container: Arc<WebViewContainer>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Received(String);

impl From<String> for Received {
    fn from(x: String) -> Self {
        Received(x)
    }
}

impl Actor for App {
    type Context = SyncContext<Self>;
}

impl Handler<AppString> for App {
    type Result = ();

    fn handle(&mut self, msg: AppString, _: &mut Self::Context) -> Self::Result {
        self.dispatch_to_web_view(msg.0);
    }
}

impl Handler<Received> for App {
    type Result = ();

    fn handle(&mut self, msg: Received, ctx: &mut Self::Context) -> Self::Result {
        self.dispatch_request(&*msg.0, ctx.address());
    }
}

impl App {
    pub fn dispatch_to_web_view<Data>(&self, body: Data)
    where
        Data: Serialize + Sync + Send + 'static,
    {
        let value = serde_json::to_string(&body).unwrap();
        println!("Dispatching: {}", value);
        self.handle
            .dispatch(move |wv| wv.eval(&*format!("sendToElm({})", value)))
            .ok();
    }

    pub(crate) fn dispatch_request(&self, param: &str, addr: Addr<Self>) {
        let invoke_body: Result<InvokeBody, serde_json::Error> = serde_json::from_str(param);
        let caller = addr.clone().recipient();
        match invoke_body {
            Ok(body) => {
                let api_name = body.api_name().clone();
                let api = self.container.get(&*api_name);
                let data = self.container.shared();
                if let Some(api) = api {
                    api.do_send(InvokeRequest { body, caller, data }).ok();
                } else {
                    let result = error(
                        "request/not-found".into(),
                        &format!("Unable to find: {} api", body.api_name),
                    );
                    self.dispatch_to_web_view(result);
                }
            }
            Err(err) => {
                let err_string = err.to_string();
                let result = error("request/failed".into(), &err_string);
                self.dispatch_to_web_view(result);
            }
        };
    }
}

pub struct AppBuilder<'a, Cont> {
    container: WebViewBuilderContainer,
    builder: WebViewBuilder<
        'a,
        MaybeUninit<Addr<App>>,
        fn(&mut WebView<MaybeUninit<Addr<App>>>, &str) -> WVResult,
        Cont,
    >,
    fullscreen: bool,
}

impl<'a, Cont> AppBuilder<'a, Cont>
where
    Cont: AsRef<str>,
{
    pub(crate) fn new(capacity: usize) -> Self {
        let api_container = WebViewBuilderContainer::new(capacity);
        AppBuilder {
            container: api_container.into(),
            builder: web_view::builder(),
            fullscreen: false,
        }
    }

    pub fn content(mut self, content: Content<Cont>) -> Self {
        self.builder = self.builder.content(content);
        self
    }

    pub fn size(mut self, w: i32, h: i32) -> Self {
        self.builder = self.builder.size(w, h);
        self
    }

    pub fn frameless(mut self, frameless: bool) -> Self {
        self.builder = self.builder.frameless(frameless);

        self
    }

    pub fn fullscreen(mut self, fullscreen: bool) -> Self {
        self.fullscreen = fullscreen;

        self
    }
}

impl<'a, Cont, API> Load<API> for AppBuilder<'a, Cont>
where
    API: KeyedActor,
{
    type Result = Self;

    fn load(mut self, api: API) -> Self::Result {
        let api_key = api.api_key();
        let addr = api.start();

        self.container.load_recipient(api_key, addr);
        self
    }
}

impl<'a, Cont, A> Load<WebViewLoadableActor<A>> for AppBuilder<'a, Cont>
where
    A: Actor<Context = Context<A>>,
{
    type Result = Self;

    fn load(mut self, api: WebViewLoadableActor<A>) -> Self::Result {
        let addr = api.0;

        self.container.load_actor(addr);

        self
    }
}


impl<'a, Cont> Application for AppBuilder<'a, Cont>
where
    Cont: AsRef<str>,
{
    type Result = WebView<'a, MaybeUninit<Addr<App>>>;

    fn finish(self) -> Self::Result {
        fn message_handler(wv: &mut WebView<MaybeUninit<Addr<App>>>, msg: &str) -> WVResult {
            let data = wv.user_data();
            let data = unsafe { data.as_ptr().as_ref() }.unwrap();
            data.do_send(Received(msg.into()));
            Ok(())
        };
        let AppBuilder {
            container,
            builder,
            fullscreen,
        } = self;
        let mut webview = builder
            .user_data(MaybeUninit::uninit())
            .invoke_handler(message_handler)
            .build()
            .unwrap();

        let handle: Handle<MaybeUninit<Addr<App>>> = webview.handle();
        let arc = Arc::new(WebViewBuilderContainer::finalize(container));

        let addr = SyncArbiter::start(4, move || App {
            handle: handle.clone(),
            container: arc.clone(),
        });

        *webview.user_data_mut() = MaybeUninit::new(addr);

        webview.set_fullscreen(fullscreen);

        webview
    }
}

pub fn builder<'a, Cont>(capacity: usize) -> AppBuilder<'a, Cont>
where
    Cont: AsRef<str>,
{
    AppBuilder::new(capacity)
}
