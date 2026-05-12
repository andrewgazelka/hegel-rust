mod common;

use common::project::TempRustProject;

/// The derive macro's generated code must compile without the user importing
/// the Generator trait. Previously, `new()` called `.boxed()` (a Generator
/// trait method) without importing the trait, so it only compiled when users
/// happened to `use hegel::DefaultGenerator` (which brings both the derive
/// macro AND the trait into scope).
#[test]
fn test_derive_compiles_without_generator_trait_import() {
    TempRustProject::new()
        .main_file(
            r#"
#[derive(Debug, hegel::DefaultGenerator)]
struct Person {
    name: String,
    age: i32,
}

fn main() {}
"#,
        )
        .cargo_run(&[]);
}

#[test]
fn test_enum_derive_compiles_with_non_snake_case_denied() {
    TempRustProject::new()
        .main_file(
            r#"
#![deny(deprecated, non_snake_case)]

use hegel::generators::DefaultGenerator as _;

#[derive(Debug, hegel::DefaultGenerator)]
enum Operation {
    Record { chunk: u8 },
    Condemn(u8),
    Purge,
}

#[derive(Debug, hegel::DefaultGenerator)]
enum KeywordVariant {
    Type(u8),
    Gen(u8),
    Crate(u8),
}

fn main() {
    let _ = Operation::default_generator()
        .record(Operation::default_generator().default_record());
    let _ = Operation::default_generator()
        .Record(Operation::default_generator().default_Record());
    let _ = KeywordVariant::default_generator()
        .r#type(KeywordVariant::default_generator().default_type());
    let _ = KeywordVariant::default_generator()
        .r#gen(KeywordVariant::default_generator().default_gen());
    let _ = KeywordVariant::default_generator()
        .crate_(KeywordVariant::default_generator().default_crate());
}
"#,
        )
        .cargo_run(&[]);
}
