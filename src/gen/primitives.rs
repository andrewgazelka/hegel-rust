//! Primitive generators for unit, boolean, and constant values.

use super::{generate_from_schema, Generate};
use serde_json::{json, Value};

/// Generate unit values.
pub fn unit() -> JustGenerator<()> {
    just(())
}

// ============================================================================
// Just Generators
// ============================================================================

/// Generator that always produces the same value (with schema).
pub struct JustGenerator<T> {
    value: T,
}

impl<T: Clone + Send + Sync + serde::Serialize> Generate<T> for JustGenerator<T> {
    fn generate(&self) -> T {
        self.value.clone()
    }

    fn schema(&self) -> Option<Value> {
        Some(json!({"const": self.value}))
    }
}

/// Generate a constant value with schema support.
///
/// Provides a `{"const": value}` schema for better shrinking.
/// For non-serializable types, use `just_any()`.
pub fn just<T: Clone + Send + Sync + serde::Serialize>(value: T) -> JustGenerator<T> {
    JustGenerator { value }
}

/// Generator that always produces the same value (no schema).
pub struct JustAnyGenerator<T> {
    value: T,
}

impl<T: Clone + Send + Sync> Generate<T> for JustAnyGenerator<T> {
    fn generate(&self) -> T {
        self.value.clone()
    }

    fn schema(&self) -> Option<Value> {
        None
    }
}

/// Generate a constant value without schema support.
///
/// Use for types that don't implement `Serialize`.
/// For serializable types, prefer `just()`.
pub fn just_any<T: Clone + Send + Sync>(value: T) -> JustAnyGenerator<T> {
    JustAnyGenerator { value }
}

// ============================================================================
// Boolean Generator
// ============================================================================

/// Generator for boolean values.
pub struct BoolGenerator;

impl Generate<bool> for BoolGenerator {
    fn generate(&self) -> bool {
        generate_from_schema(&json!({"type": "boolean"}))
    }

    fn schema(&self) -> Option<Value> {
        Some(json!({"type": "boolean"}))
    }
}

/// Generate boolean values.
pub fn booleans() -> BoolGenerator {
    BoolGenerator
}
