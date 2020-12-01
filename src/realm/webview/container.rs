use std::any::Any;
use std::any::TypeId;
use std::collections::HashMap;
use std::sync::Arc;

use actix::fut::wrap_future;
use actix::prelude::*;

use super::AppString;
use super::InvokeRequest;

pub type ActorApiContainer = HashMap<&'static str, Recipient<InvokeRequest>>;

pub struct ActorContainer(HashMap<TypeId, Arc<dyn Any + Send + Sync + 'static>>);

impl ActorContainer {
    pub fn load<T: Actor>(&mut self, addr: Addr<T>) {
        self.0.insert(TypeId::of::<T>(), Arc::new(addr));
    }

    pub fn get<T: Actor + Any + Send + Sync>(&self) -> Option<Addr<T>> {
        self.0
            .get(&TypeId::of::<T>())
            .and_then(|b| Arc::clone(b).downcast().ok())
            .map(|arc| Addr::clone(&*arc))
            .clone()
    }
}

impl From<HashMap<TypeId, Arc<dyn Any + Send + Sync>>> for ActorContainer {
    fn from(item: HashMap<TypeId, Arc<dyn Any + Send + Sync>>) -> Self {
        ActorContainer(item)
    }
}

pub(crate) struct WebViewBuilderContainer {
    api_container: HashMap<&'static str, Recipient<InvokeRequest>>,
    actor_container: ActorContainer,
    debug_content: Vec<AppString>,
}

pub(crate) struct WebViewContainer {
    actor_container: Arc<ActorContainer>,
    api_container: ActorApiContainer,
}

impl WebViewContainer {
    pub fn get(&self, key: &str) -> Option<&Recipient<InvokeRequest>> {
        self.api_container.get(key)
    }
}

impl WebViewContainer {
    pub fn shared(&self) -> Arc<ActorContainer> {
        self.actor_container.clone()
    }
}

impl WebViewBuilderContainer {
    pub fn new(capacity: usize) -> Self {
        WebViewBuilderContainer {
            api_container: HashMap::with_capacity(capacity),
            actor_container: HashMap::new().into(),
            debug_content: vec![],
        }
    }

    pub fn load_recipient<R: Into<Recipient<InvokeRequest>>>(
        &mut self,
        key: &'static str,
        recipient: R,
    ) {
        self.api_container.entry(key).or_insert(recipient.into());
    }

    pub fn load_actor<T: Actor>(&mut self, actor: Addr<T>) {
        self.actor_container.load(actor);
    }

    pub fn actor_container(&self) -> &ActorContainer {
        &self.actor_container
    }

    pub fn finalize(
        WebViewBuilderContainer {
            actor_container,
            api_container,
            debug_content: _,
        }: Self,
    ) -> WebViewContainer {
        WebViewContainer {
            actor_container: Arc::new(actor_container),
            api_container,
        }
    }
}

impl Actor for WebViewBuilderContainer {
    type Context = Context<Self>;
}

impl Handler<AppString> for WebViewBuilderContainer {
    type Result = ();

    fn handle(&mut self, msg: AppString, _: &mut Self::Context) -> Self::Result {
        println!("{}", msg.0);
        self.debug_content.push(msg);
    }
}

impl Handler<InvokeRequest> for WebViewBuilderContainer {
    type Result = ();

    fn handle(&mut self, msg: InvokeRequest, ctx: &mut Self::Context) -> Self::Result {
        let api_name = msg.body().api_name();
        self.api_container.get(&*api_name.clone()).and_then(
            move |api: &Recipient<InvokeRequest>| {
                wrap_future::<_, Self>(api.send(msg))
                    .then(|_, _, _| async {}.actfuture())
                    .spawn(ctx);
                Some(())
            },
        );
    }
}
