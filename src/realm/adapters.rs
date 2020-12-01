use crate::Api;

pub struct AndThen<A, F> {
    f: F,
    api: A,
}

impl<A, F, U> Api for AndThen<A, F>
where
    A: Api,
    F: Fn(A::Output) -> U,
{
    type Input = A::Input;
    type Output = U;

    fn handle(&self, msg: Self::Input) -> Self::Output {
        let ref map = self.f;
        map(self.api.handle(msg))
    }
}

pub trait ApiAdapters: Api + Sized {
    fn and_then<U, F: Fn(Self::Output) -> U>(self, f: F) -> AndThen<Self, F> {
        AndThen { api: self, f }
    }
}

impl<S> ApiAdapters for S where S: Api {}
