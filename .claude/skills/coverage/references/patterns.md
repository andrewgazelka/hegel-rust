# Testability Patterns

## Parameterize over environment

Extract logic from functions that read environment/global state into parameterized functions that take those values as arguments.

```rust
// Hard to test — reads env vars directly
fn cache_dir() -> PathBuf {
    if let Ok(xdg) = std::env::var("XDG_CACHE_HOME") {
        return PathBuf::from(xdg).join("myapp");
    }
    // ...
}

// Testable — takes values as parameters
fn cache_dir_from(xdg: Option<String>, home: Option<PathBuf>) -> PathBuf {
    if let Some(xdg) = xdg {
        return PathBuf::from(xdg).join("myapp");
    }
    // ...
}

// Thin wrapper calls the testable version
fn cache_dir() -> PathBuf {
    cache_dir_from(std::env::var("XDG_CACHE_HOME").ok(), std::env::home_dir())
}
```

## Platform-specific match arms

Take arch/os as parameters so all branches are testable from any platform:

```rust
// Can only test the current platform's branch
fn platform_archive_name() -> Result<String, String> {
    archive_name_for(std::env::consts::ARCH, std::env::consts::OS)
}

// All branches testable
fn archive_name_for(arch: &str, os: &str) -> Result<String, String> {
    match (arch, os) {
        ("aarch64", "macos") => ...,
        ("x86_64", "linux") => ...,
        _ => Err(...),
    }
}
```

## Command fallback chains

Take the command list as a parameter so tests can exercise the fallback without manipulating PATH:

```rust
fn compute_sha256(path: &Path) -> String {
    compute_sha256_with(path, &["sha256sum", "shasum -a 256"])
}

fn compute_sha256_with(path: &Path, commands: &[&str]) -> String {
    // try each command in order...
}
```

Test with: `compute_sha256_with(&file, &["nonexistent_tool", "sha256sum"])`

## Error paths in shell-outs

- Restructure so the error message is on the same line as the call (for line-level coverage).
- Convert defensive error returns to panics when the function has only one caller that would panic anyway.
