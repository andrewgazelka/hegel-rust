use crate::common::utils::minimal;
use hegel::generators;

#[test]
fn test_minimize_string_to_empty() {
    assert_eq!(minimal(generators::text(), |_| true), "");
}

#[test]
fn test_minimize_longer_string() {
    // Note: use chars().count() not len(), since len() counts bytes in Rust.
    let result = minimal(generators::text(), |x: &String| x.chars().count() >= 10);
    assert_eq!(result.chars().count(), 10);
    // Hypothesis shrinks to "0" * 10, but Hegel's minimal character may differ.
    // All characters should be the same (the minimal character).
    let chars: Vec<char> = result.chars().collect();
    assert!(
        chars.iter().all(|&c| c == chars[0]),
        "Expected all same character, got {:?}",
        result
    );
}

#[test]
fn test_minimize_longer_list_of_strings() {
    assert_eq!(
        minimal(generators::vecs(generators::text()), |x: &Vec<String>| {
            x.len() >= 10
        }),
        vec![""; 10]
    );
}
