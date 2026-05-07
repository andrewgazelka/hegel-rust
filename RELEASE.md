RELEASE_TYPE: patch

This patch adds generators for the [`serde_json`](https://docs.rs/serde_json) crate behind the new `serde_json` feature flag.

`hegel::extras::serde_json` provides generators for:

* `numbers()` — generates `serde_json::Number`.
* `values()` — generates `serde_json::Value`.
* `raw_values()` - generates `Box<serde_json::value::RawValue>`. Requires the `serde_json_raw_value` feature flag.

And default generators for:

* `serde_json::Number`
* `serde_json::Value`
* `serde_json::Map<String, Value>`

For example:

```rust
use hegel::extras::serde_json as json_gs;

#[hegel::test]
fn my_test(tc: hegel::TestCase) {
    let v = tc.draw(json_gs::values());
    let s = serde_json::to_string(&v).unwrap();
    let _: serde_json::Value = serde_json::from_str(&s).unwrap();
}
```
