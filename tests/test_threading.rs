//! Tests for sharing a TestCase across threads.
//!
//! `TestCase` is `Send` (but not `Sync`): a clone may be moved to another
//! thread, and data generation works from that thread. A shared reentrant
//! lock on the shared state serialises top-level interactions, so a single
//! `draw` always runs atomically.
//!
//! These tests cover *fully deterministic* uses where the test interleaves
//! thread work but does not race — that is, one thread does work, another
//! picks up after it, and the outcome is independent of scheduling.

use hegel::TestCase;
use hegel::generators as gs;
use std::sync::{Arc, Mutex};

/// Compile-time check: `TestCase` implements `Send`.
fn _assert_test_case_is_send()
where
    TestCase: Send,
{
}

/// Compile-time check: `TestCase` does *not* implement `Sync`.
///
/// Uses the standard trick of two blanket impls that would be ambiguous iff
/// `TestCase: Sync`. If someone accidentally makes `TestCase: Sync`, the
/// `not_sync` function below fails to compile with an ambiguity error.
#[allow(dead_code)]
fn _assert_test_case_is_not_sync() {
    trait AmbiguousIfSync<A> {
        fn some_item() {}
    }
    impl<T: ?Sized> AmbiguousIfSync<()> for T {}
    impl<T: ?Sized + Sync> AmbiguousIfSync<u8> for T {}
    <TestCase as AmbiguousIfSync<_>>::some_item();
}
#[hegel::test(test_cases = 20)]
fn test_spawn_thread_with_clone_does_generation(tc: TestCase) {
    let tc_clone = tc.clone();
    let handle = std::thread::spawn(move || {
        let n: u32 = tc_clone.draw(gs::integers());
        n
    });
    let thread_value = handle.join().expect("thread panicked");

    // After the thread has joined, the main thread resumes generation.
    let main_value: bool = tc.draw(gs::booleans());
    let _ = (thread_value, main_value);
}

#[hegel::test(test_cases = 20)]
fn test_main_then_thread_then_main(tc: TestCase) {
    let _a: u32 = tc.draw(gs::integers());

    let tc_clone = tc.clone();
    let handle = std::thread::spawn(move || {
        let b: String = tc_clone.draw(gs::text());
        let c: bool = tc_clone.draw(gs::booleans());
        (b, c)
    });
    let _ = handle.join().expect("thread panicked");

    let _d: u32 = tc.draw(gs::integers());
}

#[hegel::test(test_cases = 10)]
fn test_sequential_threads_each_do_one_draw(tc: TestCase) {
    // Spawn threads one at a time, fully joining each before the next.
    // This is deterministic: no concurrent generation ever happens.
    for _ in 0..3 {
        let tc_clone = tc.clone();
        let handle = std::thread::spawn(move || {
            let v: u32 = tc_clone.draw(gs::integers());
            v
        });
        let _ = handle.join().expect("thread panicked");
    }

    let _: bool = tc.draw(gs::booleans());
}

#[hegel::test(test_cases = 5)]
fn test_nested_generators_work_across_thread_boundary(tc: TestCase) {
    // Composed generators (map, flat_map, vecs) must still work when
    // drawn from another thread.
    let tc_clone = tc.clone();
    let captured: Arc<Mutex<Option<Vec<i32>>>> = Arc::new(Mutex::new(None));
    let captured_clone = Arc::clone(&captured);

    let handle = std::thread::spawn(move || {
        let xs: Vec<i32> = tc_clone.draw(gs::vecs(gs::integers::<i32>()).max_size(5));
        *captured_clone.lock().unwrap() = Some(xs);
    });
    handle.join().expect("thread panicked");

    let xs = captured
        .lock()
        .unwrap()
        .take()
        .expect("thread produced no value");
    assert!(xs.len() <= 5);
}
