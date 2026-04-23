RELEASE_TYPE: minor

This release adds a `DefaultGenerator` impl for `PathBuf` and a public `Hegel::run_with_runner(...)` hook so downstreams can model filesystem paths and provide custom execution backends without forking Hegel.
