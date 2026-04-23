RELEASE_TYPE: minor

This release adds a `DefaultGenerator` impl for `PathBuf`, so `#[derive(DefaultGenerator)]` works for structs with path fields and exercises common filesystem edge cases by default.
