---
name: coverage
description: "How to approach code coverage in this project. Use when coverage CI fails, when writing tests for new code, when deciding whether to add // nocov, or when you need to make untestable code testable. Also use proactively when writing new code to ensure it will be coverable."
---

# Code Coverage

This project requires 100% line coverage for new code.

## The ratchet is not a budget

The nocov count in `.github/coverage-ratchet.json` tracks excluded lines and can only decrease. Just because previous work reduced the count does not mean you have implicit permission to add new uncovered lines. Think of the ratchet as immediately ratcheting down after any reduction — the slack is gone.

You may not add `// nocov` annotations without explicit human permission. If you think code is genuinely untestable, your first move should be to refactor it for testability, not to annotate it. See `references/patterns.md` for testability refactoring examples.

## Writing good tests

Tests should catch real bugs, not mirror the implementation.

- **Validate against external reality**: if your code maps to external identifiers (URLs, file names, API paths), hardcode the real external data in the test and validate against it. Don't just assert that each match arm produces a specific string — that's duplicating the code.
- **Test behavior, not structure**: a test that would still pass after introducing a bug is not testing anything.
- **Avoid network in unit tests**: use `file://` URLs with curl, create local tar.gz fixtures, use `tempfile::tempdir()` for isolation. Keep exactly one integration test that verifies the real network path works end-to-end.
- **Shell out to system tools** (sha256sum, tar) rather than reimplementing them. Makes the code simpler and the tests more realistic.

## Diagnosing coverage failures

When CI coverage fails:

1. Read the failure output — it lists each uncovered file:line and content.
2. Categorize each uncovered line:
   - **Thin wrapper**: a 1-2 line function that just delegates to a parameterized version. Is it called by any test (including integration tests via TempRustProject)? If subprocess coverage isn't picking it up, you may need a unit test that calls the wrapper directly.
   - **Platform-specific code**: refactor to take platform as a parameter (see `references/patterns.md`).
   - **Error handling**: can you trigger the error in a test? (Bad file path, bad URL, invalid input.) If the error path is inside a panic/assert format string, the coverage checker excludes those continuation lines automatically.
   - **Dead code**: if it's truly unreachable, delete it.
3. Run `just check-coverage` locally if `cargo-llvm-cov` is available to iterate faster than CI.

For details on how the coverage script works and what it auto-excludes, see `references/internals.md`.
