use super::{BoxedGenerator, Generator};
use crate::test_case::TestCase;
use std::sync::{Arc, OnceLock};

struct DeferredGenerator<T> {
    inner: Arc<OnceLock<BoxedGenerator<'static, T>>>,
}

impl<T> Clone for DeferredGenerator<T> {
    fn clone(&self) -> Self {
        DeferredGenerator {
            inner: Arc::clone(&self.inner),
        }
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

/// A deferred generator definition that can produce generator handles
/// before its implementation is known.
///
/// Created by [`deferred()`]. Call [`generator()`](Self::generator) to get
/// handles that can be passed to other generators, then call [`set()`](Self::set)
/// to provide the actual implementation. `set` consumes the definition,
/// ensuring it can only be called once.
///
/// # Example
///
/// ```no_run
/// use hegel::generators::{self as gs, Generator};
///
/// let x = gs::deferred::<i32>();
/// let x_gen = x.generator();
/// let y = hegel::one_of!(gs::integers::<i32>(), x_gen);
/// x.set(y);
/// ```
pub struct DeferredGeneratorDefinition<T> {
    inner: Arc<OnceLock<BoxedGenerator<'static, T>>>,
}

impl<T: Send + Sync + 'static> DeferredGeneratorDefinition<T> {
    /// Return a generator handle that will delegate to whatever is
    /// eventually passed to [`set()`](Self::set).
    ///
    /// Can be called multiple times to produce independent handles
    /// that all share the same underlying definition.
    pub fn generator(&self) -> BoxedGenerator<'static, T> {
        DeferredGenerator {
            inner: Arc::clone(&self.inner),
        }
        .boxed()
    }

    /// Set the implementation for this deferred generator.
    ///
    /// All handles previously returned by [`generator()`](Self::generator)
    /// will delegate to the provided generator. Consumes the definition,
    /// so it can only be called once.
    ///
    /// # Panics
    ///
    /// Drawing from a handle before `set` is called will panic.
    pub fn set(self, generator: impl Generator<T> + 'static) {
        let _ = self.inner.set(generator.boxed());
    }
}

/// Create a deferred generator definition for forward references.
///
/// Returns a [`DeferredGeneratorDefinition`] that can produce generator
/// handles before the implementation is known. This enables self-recursive
/// and mutually recursive generator definitions.
///
/// # Example
///
/// ```no_run
/// use hegel::generators::{self as gs, Generator};
///
/// // Declare both generators
/// let x = gs::deferred::<i32>();
/// let y = gs::deferred::<i32>();
///
/// // Get handles for use in definitions
/// let x_gen = x.generator();
/// let y_gen = y.generator();
///
/// // Define them, referencing each other
/// y.set(hegel::one_of!(gs::integers::<i32>(), x_gen));
/// x.set(hegel::one_of!(gs::integers::<i32>(), y_gen));
/// ```
pub fn deferred<T>() -> DeferredGeneratorDefinition<T> {
    DeferredGeneratorDefinition {
        inner: Arc::new(OnceLock::new()),
    }
}
