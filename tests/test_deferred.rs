mod common;

use common::utils::find_any;
use hegel::TestCase;
use hegel::generators::{self as gs, Generator};

#[hegel::test]
fn test_deferred_delegates_to_inner(tc: TestCase) {
    let d = gs::deferred();
    d.set(gs::integers::<i32>().min_value(0).max_value(10).boxed());
    let value = tc.draw(d);
    assert!((0..=10).contains(&value));
}

#[test]
fn test_deferred_can_generate_both_true_and_false() {
    let d = gs::deferred();
    d.set(gs::booleans().boxed());
    find_any(d.clone(), |v| *v);
    find_any(d, |v| !v);
}

#[hegel::test]
fn test_deferred_clone_shares_definition(tc: TestCase) {
    let d = gs::deferred::<i32>();
    let d2 = d.clone();
    d.set(gs::integers().min_value(0).max_value(10).boxed());
    let value = tc.draw(d2);
    assert!((0..=10).contains(&value));
}

#[test]
#[should_panic(expected = "has already been set")]
fn test_deferred_set_twice_panics() {
    let d = gs::deferred::<bool>();
    d.set(gs::booleans().boxed());
    d.set(gs::booleans().boxed());
}

#[test]
#[should_panic(expected = "has not been set")]
fn test_deferred_draw_before_set_panics() {
    hegel::hegel(|tc| {
        let d = gs::deferred::<bool>();
        tc.draw(d);
    });
}

#[hegel::test]
fn test_deferred_works_with_map(tc: TestCase) {
    let d = gs::deferred();
    d.set(gs::integers::<i32>().min_value(0).max_value(100).boxed());
    let value = tc.draw(d.map(|n| n * 2));
    assert!(value % 2 == 0);
    assert!((0..=200).contains(&value));
}

#[hegel::test]
fn test_deferred_self_recursive(tc: TestCase) {
    let d = gs::deferred::<Vec<bool>>();
    d.set(
        hegel::one_of!(
            gs::just(vec![]),
            d.clone().map(|mut v| {
                v.push(true);
                v
            }),
        )
        .boxed(),
    );
    let value = tc.draw(d);
    assert!(value.iter().all(|b| *b));
}

#[test]
fn test_deferred_mutual_recursion() {
    let x = gs::deferred::<i32>();
    let y = gs::deferred::<i32>();

    y.set(hegel::one_of!(gs::integers::<i32>().min_value(0).max_value(10), x.clone(),).boxed());

    x.set(
        hegel::one_of!(
            gs::integers::<i32>().min_value(100).max_value(110),
            y.clone(),
        )
        .boxed(),
    );

    find_any(x.clone(), |v| (0..=10).contains(v));
    find_any(x, |v| (100..=110).contains(v));
}
