RELEASE_TYPE: patch

This patch makes Hegel surface every distinct failing test case from a
run, rather than collapsing multi-bug runs down to whichever failure
fired last. Previously the runner would print only one diagnostic and
re-raise with one message, even when Hypothesis had actually found
several unrelated bugs in the same run. That could even hide unrelated
panics — e.g. an overflow in a test predicate would get silently
overwritten by a later assertion failure.

When the run produces multiple distinct failures, Hegel now prints:

```
Hegel found N failing test cases:
<diagnostic for failure 1>
<diagnostic for failure 2>
...
```

followed by a `Property-based test failed with N distinct failures.`
panic. Single-failure runs are unchanged: the diagnostic and
`Property test failed: <msg>` panic look exactly as before.

This patch also adds a `report_multiple_failures` setting (default
`true`) which controls the behaviour:

```rust
use hegel::Settings;

let settings = Settings::new().report_multiple_failures(false);
```

When set to `false`, Hegel asks the server to collapse multi-bug runs
to a single failing example — useful when one root-cause bug shows up
as several superficially-distinct failures and the extra reports are
noise. The setting maps to Hypothesis's `report_multiple_bugs`. It
takes effect when paired with hegel-core 0.9.0 or later (older
versions silently ignore the field and use the Hypothesis default).
