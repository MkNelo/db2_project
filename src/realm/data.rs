use std::any::Any;
use std::sync::Arc;
use std::sync::Mutex;

trait LoadData<T> {
    type Result;

    fn data(self, t: T) -> Self::Result;
}

trait SolveData<T> {
    type Data;

    fn solve(&self) -> Self::Data;
}

struct Data(Arc<dyn Any + Send + Sync + 'static>);

struct SolvedData<T>(Arc<T>);

impl<T> SolveData<T> for Data
where
    T: Sync + Send + 'static,
{
    type Data = Option<SolvedData<T>>;

    fn solve(&self) -> Self::Data {
        self.0.clone().downcast().ok().map(SolvedData)
    }
}

struct LazyData<F, T>(Mutex<Option<Arc<T>>>, F);

impl<F, T> SolveData<T> for LazyData<F, T>
where
    F: Fn() -> T,
{
    type Data = Arc<T>;

    fn solve(&self) -> Self::Data {
        let ref mut handle = self.0.lock().unwrap();
        handle.get_or_insert_with(|| Arc::new(self.1())).clone()
    }
}
