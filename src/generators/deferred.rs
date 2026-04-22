use super::{BoxedGenerator, Generator};
use crate::test_case::TestCase;
use std::sync::{Arc, OnceLock};

pub struct DeferredGenerator<T> {
    inner: Arc<OnceLock<BoxedGenerator<'static, T>>>,
}

impl<T> Clone for DeferredGenerator<T> {
    fn clone(&self) -> Self {
        DeferredGenerator {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<T> DeferredGenerator<T> {
    pub fn set(&self, generator: BoxedGenerator<'static, T>) {
        self.inner
            .set(generator)
            .unwrap_or_else(|_| panic!("DeferredGenerator has already been set"));
    }
}

impl<T: Send + Sync> Generator<T> for DeferredGenerator<T> {
    fn do_draw(&self, tc: &TestCase) -> T {
        self.inner
            .get()
            .unwrap_or_else(|| panic!("DeferredGenerator has not been set"))
            .do_draw(tc)
    }
}

pub fn deferred<T>() -> DeferredGenerator<T> {
    DeferredGenerator {
        inner: Arc::new(OnceLock::new()),
    }
}
