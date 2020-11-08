use std::cell::RefCell;

use futures::{
    executor::{LocalPool, LocalSpawner},
    task::Spawn,
};
use tokio::runtime::Builder;
use tokio::runtime::Runtime;

pub trait SpawnerFactory {
    type Spawner: Spawn;

    fn spawner(&mut self) -> Self::Spawner;
}

impl SpawnerFactory for LocalPool {
    type Spawner = LocalSpawner;

    fn spawner(&mut self) -> Self::Spawner {
        LocalPool::spawner(self)
    }
}

pub struct LazyFactorySpawner<SpawnerF, Factory>(Factory, RefCell<Option<SpawnerF>>);

impl<Factory, SpawnerF> SpawnerFactory for LazyFactorySpawner<SpawnerF, Factory>
where
    Factory: Fn() -> SpawnerF,
    SpawnerF: SpawnerFactory,
{
    type Spawner = SpawnerF::Spawner;

    fn spawner(&mut self) -> Self::Spawner {
        let ref mut insides = self.1.borrow_mut();
        let spawner = insides.get_or_insert_with(|| self.0());
        spawner.spawner()
    }
}

pub struct TokioBuilder(Builder);

pub struct TokioSpawner(pub(crate) Runtime);

impl SpawnerFactory for TokioBuilder {
    type Spawner = TokioSpawner;

    fn spawner(&mut self) -> Self::Spawner {
        self.0
            .worker_threads(4)
            .thread_name("Realm Thread")
            .enable_all()
            .build()
            .and_then(|result| Ok(TokioSpawner(result)))
            .unwrap()
    }
}

impl Spawn for TokioSpawner {
    fn spawn_obj(
        &self,
        future: futures::future::FutureObj<'static, ()>,
    ) -> Result<(), futures::task::SpawnError> {
        self.0.spawn(future);
        Ok(())
    }
}

pub fn tokio() -> TokioBuilder {
    TokioBuilder(Builder::new_multi_thread())
}
