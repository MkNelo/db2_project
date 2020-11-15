use std::sync::Arc;
use crate::prelude::*;
mod helper;
use futures::future::{join_all, pending};
use helper::*;
use tokio::{runtime::Runtime, spawn};

#[test]
fn lazy_container_works() {
    let lazy_api = lazy("api/load", lazy_apis);
    let executor = tokio().spawner().0;
    
    let mut spawns = Vec::with_capacity(20);
    let container = container_factory();
    
    for _ in 0..20 {
        spawns.push(executor.spawn(lazy_api.handle(full_request(2, container.clone()))));
    }
    
    executor.block_on(futures::future::join_all(spawns));
}

#[test]
fn lazy_api_query_works() {
    let lazy_api = lazy("api/load", lazy_api_query);
    let executor = Runtime::new().unwrap();
    let mut spawns = Vec::with_capacity(20);
    
    let container = container_factory();

    for x in 0..100 {
        let container_clone = container.clone();
        spawns.push(executor.spawn(
            lazy_api.handle(full_request(x, container_clone))
        ));
    }

    executor.block_on(join_all(spawns));
}
