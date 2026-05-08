RELEASE_TYPE: patch

This patch adds `tc.target(score, label)` for targeted property-based testing. Call it inside a test body to feed an observation back to the engine, which uses the score to guide generation toward higher-scoring inputs.

```rust
use hegel::generators as gs;

#[hegel::test]
fn my_test(tc: hegel::TestCase) {
    let n: u64 = tc.draw(gs::integers::<u64>().max_value(1000));
    let m: u64 = tc.draw(gs::integers::<u64>().max_value(1000));
    tc.target((n + m) as f64, "");
    assert!(n + m < 2000);
}
```

The label distinguishes multiple simultaneous targeting goals; pass `""` for a single unlabeled score.
