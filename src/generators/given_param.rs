use std::marker::PhantomData;

use super::{BasicGenerator, Generate, TestCaseData};

/// A wrapper generator that marks a draw as a `#[given]` parameter.
///
/// This is used internally by the `#[given]` macro so that parameter draws
/// appear in the function-call output format (`test_foo(x: 42, s: "hello")`)
/// instead of the manual-draw format (`Draw 1: 42`).
#[doc(hidden)]
pub struct GivenParam<G, T> {
    inner: G,
    name: &'static str,
    _phantom: PhantomData<fn() -> T>,
}

impl<G, T> GivenParam<G, T> {
    pub fn new(inner: G, name: &'static str) -> Self {
        GivenParam {
            inner,
            name,
            _phantom: PhantomData,
        }
    }
}

impl<G: Generate<T> + Send + Sync, T> Generate<T> for GivenParam<G, T> {
    fn do_draw(&self, data: &TestCaseData) -> T {
        data.set_given_param_name(self.name.to_owned());
        self.inner.do_draw(data)
    }

    fn as_basic(&self) -> Option<BasicGenerator<'_, T>> {
        self.inner.as_basic()
    }
}

// SAFETY: GivenParam is Send+Sync if G is
unsafe impl<G: Send, T> Send for GivenParam<G, T> {}
unsafe impl<G: Sync, T> Sync for GivenParam<G, T> {}
