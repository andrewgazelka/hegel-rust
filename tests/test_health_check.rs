mod common;

use hegel::HealthCheck;
use hegel::TestCase;
use hegel::generators as gs;



/// Macro-form suppression: with `tc.assume(n == 42)` rejecting
/// ~99.99% of an `i32` range, FilterTooMuch *would* fire if not
/// suppressed (see `native_filter_too_much_detected` above for the
/// without-suppression failure shape).  This test pairs with that
/// one to confirm `#[hegel::test(suppress_health_check =
/// [HealthCheck::FilterTooMuch])]` actually applies the suppression
/// — the body's panic-free completion under heavy filtering is the
/// behavioural claim.
#[hegel::test(suppress_health_check = [HealthCheck::FilterTooMuch])]
fn test_filter_too_much_suppressed(tc: TestCase) {
    let n: i32 = tc.draw(gs::integers::<i32>().min_value(0).max_value(1_000_000));
    tc.assume(n == 42);
}

/// Macro-form suppression with the multi-element array syntax:
/// `[FilterTooMuch, TooSlow]`.  Heavy filtering + a sleep that pushes
/// per-case time near the TooSlow threshold confirms both
/// suppressions are applied (without either, the body would fail one
/// of the two checks).
#[hegel::test(suppress_health_check = [HealthCheck::FilterTooMuch, HealthCheck::TooSlow])]
fn test_suppress_multiple(tc: TestCase) {
    let n: i32 = tc.draw(gs::integers::<i32>().min_value(0).max_value(1_000_000));
    tc.assume(n == 42);
    std::thread::sleep(std::time::Duration::from_millis(10));
}

/// Macro-form suppression with the function-call syntax
/// `HealthCheck::all()`.  Same heavy filtering; same paired-test
/// reasoning: any active health check would fire, but `all()` covers
/// every variant so suppression carries the run through.
#[hegel::test(suppress_health_check = HealthCheck::all())]
fn test_suppress_all(tc: TestCase) {
    let n: i32 = tc.draw(gs::integers::<i32>().min_value(0).max_value(1_000_000));
    tc.assume(n == 42);
    std::thread::sleep(std::time::Duration::from_millis(10));
}





// ── hypothesis/test_health_checks.py ────────────────────────────────────────

mod health_checks {
    //! Individual upstream tests not ported (see SKIPPED.md):
    //!
    //! - `test_returning_non_none_is_forbidden`,
    //!   `test_stateful_returnvalue_healthcheck` — check Hypothesis's `return_value`
    //!   health check on `@given`/`@rule`/`@initialize`/`@invariant`-decorated
    //!   functions. Rust closures have declared return types already; the concept
    //!   is Python-specific and hegel-rust has no corresponding variant.
    //! - `test_the_slow_test_health_check_can_be_disabled`,
    //!   `test_the_slow_test_health_only_runs_if_health_checks_are_on` — use the
    //!   `deadline=None` setting and `skipif_time_unpatched`, a pytest-specific
    //!   time-freezing fixture. hegel-rust has no `deadline` setting.
    //! - `test_differing_executors_fails_health_check` — tests the
    //!   `differing_executors` health check on `@given`-decorated instance methods
    //!   called with different `self` receivers. hegel-rust tests are closures
    //!   passed to `Hegel::new(...).run()` with no class/instance dispatch.
    //! - `test_it_is_an_error_to_suppress_non_iterables`,
    //!   `test_it_is_an_error_to_suppress_non_healthchecks` — Python dynamic
    //!   typing: pass a non-iterable or non-`HealthCheck` to
    //!   `suppress_health_check`. Rust's type system prevents these at compile
    //!   time (`impl IntoIterator<Item = HealthCheck>`).
    //! - `test_nested_given_raises_healthcheck`,
    //!   `test_triply_nested_given_raises_healthcheck`,
    //!   `test_can_suppress_nested_given`,
    //!   `test_cant_suppress_nested_given_on_inner`,
    //!   `test_suppress_triply_nested_given` — all exercise
    //!   `HealthCheck.nested_given`, which detects a `@given`-decorated function
    //!   being called from inside another `@given` function. hegel-rust has no
    //!   `nested_given` variant and no decorator-based test dispatch to nest.

    use hegel::generators as gs;
    use hegel::{HealthCheck, Hegel, Settings, TestCase};



    #[test]
    fn test_default_health_check_can_weaken_specific() {
        Hegel::new(|tc: TestCase| {
            let _: Vec<i64> = tc.draw(gs::vecs(gs::integers::<i64>()).min_size(1));
        })
        .settings(
            Settings::new()
                .test_cases(11)
                .database(None)
                .suppress_health_check(HealthCheck::all()),
        )
        .run();
    }



}
