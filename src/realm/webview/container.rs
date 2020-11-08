use std::any::Any;
use std::any::TypeId;
use std::sync::Arc;

use futures::{Future, future::BoxFuture, task::{Spawn, SpawnError, SpawnExt}};
use tokio::sync::RwLock;

use super::super::BoxedApi;

use super::super::middleware::compose;
use super::super::middleware::Middleware;
use super::Api;
use super::DataContainer;
use super::InvokeRequest;
use super::{super::middleware::Compose, ApiContainer, ApiResponse};

pub struct WVDataContainer(pub(crate) Arc<RwLock<DataContainer>>);

impl Clone for WVDataContainer {
    fn clone(&self) -> Self {
        WVDataContainer(Arc::clone(&self.0))
    }
}

impl WVDataContainer {
    pub async fn get<T>(&self) -> Option<Arc<T>>
    where
        T: Any + Send + 'static + Sync,
    {
        let ptr = Arc::clone(&self.0);
        let ref true_container = ptr.read().await;
        true_container
            .get(&TypeId::of::<T>())
            .and_then(|arc| Arc::clone(arc).downcast().ok())
    }

    pub async fn register<T>(&mut self, t: T)
    where
        T: Send + 'static + Sync,
    {
        let container = Arc::clone(&self.0);
        let ref mut container = container.write().await;
        container.insert(TypeId::of::<T>(), Arc::new(t));
    }

    pub async fn get_or_register<T, Fut, F>(&mut self, f: F) -> Arc<T>
    where
        Fut: Future<Output = T> + Send + 'static,
        F: FnOnce() -> Fut,
        T: Any + Send + 'static + Sync,
    {
        if self.get::<T>().await.is_none() {
            let true_container = self.0.clone();
            let mut true_container = true_container.write().await;
            if !true_container.contains_key(&TypeId::of::<T>()) {
                true_container.insert(TypeId::of::<T>(), Arc::new(f().await));
            }
        }

        self.get().await.unwrap()
    }
}

pub(crate) struct WVPreContainer<
    'a,
    Mid: Middleware<Message = InvokeRequest, Response = BoxFuture<'a, ApiResponse>>,
> {
    pub(crate) middleware_container: Mid,
    pub(crate) api_container: ApiContainer<'a>,
}

pub struct WVContainer<'a, Mid, Spawner>
where
    Mid: Middleware<Message = InvokeRequest, Response = BoxFuture<'a, ApiResponse>>,
    Spawner: Spawn,
{
    api_container: ApiContainer<'a>,
    context: Spawner,
    data_container: Arc<RwLock<DataContainer>>,
    middleware: Mid,
}

impl<'a, Mid, Spawner> WVContainer<'a, Mid, Spawner>
where
    Mid: Middleware<Message = InvokeRequest, Response = BoxFuture<'a, ApiResponse>>,
    Spawner: Spawn,
{
    pub fn get(&self, key: String) -> Option<&BoxedApi<InvokeRequest, BoxFuture<'a, ApiResponse>>> {
        self.api_container.get(&*key)
    }

    pub fn solve_with_middleware(
        &self,
        data: InvokeRequest,
        api: &BoxedApi<InvokeRequest, BoxFuture<'a, ApiResponse>>,
    ) -> BoxFuture<'a, ApiResponse> {
        self.middleware.manage(data, &|ib| api.handle(ib))
    }

    pub fn dispatch<F>(&self, comput: F) -> Result<(), SpawnError>
    where
        F: Future<Output = ()> + 'static + Send,
    {
        self.context.spawn(comput)
    }

    pub fn data(&self) -> WVDataContainer {
        WVDataContainer(self.data_container.clone())
    }
}

impl<'a, Mid> WVPreContainer<'a, Mid>
where
    Mid: Middleware<Message = InvokeRequest, Response = BoxFuture<'a, ApiResponse>>,
{
    pub fn insert(
        &mut self,
        key: &'a str,
        api: BoxedApi<InvokeRequest, BoxFuture<'a, ApiResponse>>,
    ) {
        self.api_container.insert(key, api);
    }

    pub fn push<M: Middleware<Message = Mid::Message, Response = Mid::Response> + 'a>(
        self,
        mid: M,
    ) -> WVPreContainer<'a, Compose<Mid, M>> {
        WVPreContainer {
            middleware_container: compose(self.middleware_container, mid),
            api_container: self.api_container,
        }
    }

    pub fn resolve_middleware<Spawner: Spawn>(
        self,
        context: Spawner,
        data_container: DataContainer,
    ) -> WVContainer<'a, Mid, Spawner> {
        let WVPreContainer {
            api_container,
            middleware_container: middleware,
        } = self;
        WVContainer {
            api_container,
            middleware,
            context,
            data_container: Arc::new(RwLock::new(data_container)),
        }
    }
}
