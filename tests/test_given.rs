mod common;

use common::project::TempRustProject;
use hegel::generators;

#[hegel::given(generators::integers::<i32>())]
#[test]
fn test_given_positional_single(x: i32) {
    let _ = x;
}

#[hegel::given(generators::integers::<i32>(), generators::booleans())]
#[test]
fn test_given_positional_multi(a: i32, b: bool) {
    let _ = (a, b);
}

#[hegel::given(_)]
#[test]
fn test_given_infer_all(a: i32, b: bool) {
    let _ = (a, b);
}

#[hegel::given(_, _)]
#[test]
fn test_given_infer_all_multiple(a: i32, b: bool) {
    let _ = (a, b);
}

#[hegel::given(_, generators::booleans())]
#[test]
fn test_given_infer_mixed(a: i32, b: bool) {
    let _ = (a, b);
}

#[hegel::given(generators::booleans())]
#[hegel::settings(test_cases = 10)]
#[test]
fn test_given_with_settings(x: bool) {
    let _ = x;
}

#[hegel::settings(test_cases = 10)]
#[hegel::given(generators::booleans())]
#[test]
fn test_settings_before_given(x: bool) {
    let _ = x;
}

#[hegel::given(generators::booleans())]
#[test]
fn test_given_with_extra_draw(x: bool) {
    let y: String = hegel::draw(&generators::text());
    let _ = (x, y);
}

#[test]
fn test_given_param_output_format() {
    let code = r#"
use hegel::generators;

#[hegel::given(generators::booleans())]
fn main(x: bool) {
    panic!("fail");
}
"#;
    let output = TempRustProject::new(code).run();
    assert!(!output.status.success());

    assert!(
        output.stderr.contains("main(x: false)"),
        "Expected named param output, got: {}",
        output.stderr
    );
}

#[test]
fn test_settings_without_given_fails() {
    let code = r#"
use hegel::generators;

#[hegel::settings(test_cases = 10)]
fn main() {}
"#;
    let output = TempRustProject::new(code).run();
    assert!(
        !output.status.success(),
        "Expected compile error for #[settings] without #[given]"
    );
    assert!(
        output.stderr.contains("#[settings] requires #[given]"),
        "Expected specific error message, got: {}",
        output.stderr
    );
}

#[test]
fn test_given_wrong_arg_count_fails() {
    let code = r#"
use hegel::generators;

#[hegel::given(generators::booleans(), generators::booleans())]
fn main(x: bool) {
    let _ = x;
}
"#;
    let output = TempRustProject::new(code).run();
    assert!(!output.status.success());
}

#[test]
fn test_given_rejects_tuple_destructuring() {
    let code = r#"
use hegel::generators;

#[hegel::given(generators::booleans())]
fn main((a, b): (bool, bool)) {
    let _ = (a, b);
}
"#;
    let output = TempRustProject::new(code).run();
    assert!(!output.status.success());
    assert!(
        output
            .stderr
            .contains("does not support tuple destructuring"),
        "Expected tuple destructuring error, got: {}",
        output.stderr
    );
}

#[test]
fn test_given_rejects_struct_destructuring() {
    let code = r#"
use hegel::generators;

struct Foo { x: bool }

#[hegel::given(generators::booleans())]
fn main(Foo { x }: Foo) {
    let _ = x;
}
"#;
    let output = TempRustProject::new(code).run();
    assert!(!output.status.success());
    assert!(
        output
            .stderr
            .contains("does not support struct destructuring"),
        "Expected struct destructuring error, got: {}",
        output.stderr
    );
}

#[test]
fn test_given_rejects_wildcard_param() {
    let code = r#"
use hegel::generators;

#[hegel::given(generators::booleans())]
fn main(_: bool) {
    // wildcard param — no name to bind
}
"#;
    let output = TempRustProject::new(code).run();
    assert!(!output.status.success());
    assert!(
        output
            .stderr
            .contains("does not support wildcard (_) parameters"),
        "Expected wildcard parameter error, got: {}",
        output.stderr
    );
}

#[test]
fn test_given_rejects_slice_destructuring() {
    let code = r#"
use hegel::generators;

#[hegel::given(generators::booleans())]
fn main([a, b]: [bool; 2]) {
    let _ = (a, b);
}
"#;
    let output = TempRustProject::new(code).run();
    assert!(!output.status.success());
    assert!(
        output
            .stderr
            .contains("does not support slice destructuring"),
        "Expected slice destructuring error, got: {}",
        output.stderr
    );
}

#[test]
fn test_given_rejects_ref_pattern() {
    let code = r#"
use hegel::generators;

#[hegel::given(generators::booleans())]
fn main(&x: &bool) {
    let _ = x;
}
"#;
    let output = TempRustProject::new(code).run();
    assert!(!output.status.success());
    assert!(
        output
            .stderr
            .contains("does not support reference patterns"),
        "Expected named parameter error, got: {}",
        output.stderr
    );
}
