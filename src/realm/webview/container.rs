use std::any::Any;
use std::any::TypeId;
use std::sync::Arc;

use futures::future::ready;
use futures::lock::Mutex;
use futures::FutureExt;
use futures::{
    future::BoxFuture,
    task::{Spawn, SpawnError, SpawnExt},
    Future,
};

use super::super::BoxedApi;

use super::super::middleware::compose;
use super::super::middleware::Middleware;
use super::Api;
use super::DataContainer;
use super::InvokeRequest;
use super::{super::middleware::Compose, ApiContainer, ApiResponse};

pub struct WVDataContainer(pub(crate) Arc<Mutex<DataContainer>>);

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
        let ref true_container = ptr.lock().await;
        true_container
            .get(&TypeId::of::<T>())
            .and_then(|arc| Arc::clone(arc).downcast().ok())
    }

    pub async fn register<T>(&mut self, t: T)
    where
        T: Send + 'static + Sync,
    {
        let container = Arc::clone(&self.0);
        let ref mut container = container.lock().await;
        container.insert(TypeId::of::<T>(), Arc::new(t));
    }

    pub async fn get_or_register<T, Fut, F>(&mut self, f: F) -> Arc<T>
    where
        Fut: Future<Output = T> + Send + 'static,
        F: FnOnce() -> Fut,
        T: Any + Send + 'static + Sync,
    {
        let true_container = Arc::clone(&self.0);
        let id = TypeId::of::<T>();
        let mut handle = true_container.lock().await;
        if handle.contains_key(&id) {
            let ptr = handle.get(&id).unwrap().clone();
            ready(ptr.downcast().unwrap()).left_future()
        } else {
            f().map(|f| Arc::new(f))
                .then(|r| {
                    handle.insert(id, r.clone());
                    ready(r)
                })
                .right_future()
        }
        .await
    }
}

pub(crate) struct WVPreContainer<
    Mid: Middleware<Message = InvokeRequest, Response = BoxFuture<'static, ApiResponse>>,
> {
    pub(crate) middleware_container: Mid,
    pub(crate) api_container: ApiContainer,
}

pub struct WVContainer<Mid, Spawner>
where
    Mid: Middleware<Message = InvokeRequest, Response = BoxFuture<'static, ApiResponse>>,
    Spawner: Spawn,
{
    api_container: ApiContainer,
    context: Spawner,
    data_container: Arc<Mutex<DataContainer>>,
    middleware: Mid,
}

impl<Mid, Spawner> WVContainer<Mid, Spawner>
where
    Mid: Middleware<Message = InvokeRequest, Response = BoxFuture<'static, ApiResponse>>,
    Spawner: Spawn,
{
    pub fn get(
        &self,
        key: String,
    ) -> Option<&dyn Api<Message = InvokeRequest, Response = BoxFuture<'static, ApiResponse>>> {
        self.api_container.get(&*key).map(AsRef::as_ref)
    }

    pub fn solve_with_middleware(
        &self,
        data: InvokeRequest,
        api: &dyn Api<Message = InvokeRequest, Response = BoxFuture<'static, ApiResponse>>,
    ) -> impl Future<Output = ApiResponse> {
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

impl<'a, Mid> WVPreContainer<Mid>
where
    Mid: Middleware<Message = InvokeRequest, Response = BoxFuture<'static, ApiResponse>>,
{
    pub fn insert(
        &mut self,
        key: &'static str,
        api: BoxedApi<InvokeRequest, BoxFuture<'static, ApiResponse>>,
    ) {
        self.api_container.insert(key, api);
    }

    pub fn push<M: Middleware<Message = Mid::Message, Response = Mid::Response> + 'a>(
        self,
        mid: M,
    ) -> WVPreContainer<Compose<Mid, M>> {
        WVPreContainer {
            middleware_container: compose(self.middleware_container, mid),
            api_container: self.api_container,
        }
    }

    pub fn resolve_middleware<Spawner: Spawn>(
        self,
        context: Spawner,
        data_container: Arc<Mutex<DataContainer>>,
    ) -> WVContainer<Mid, Spawner> {
        let WVPreContainer {
            api_container,
            middleware_container: middleware,
        } = self;
        WVContainer {
            api_container,
            middleware,
            context,
            data_container,
        }
    }
}
