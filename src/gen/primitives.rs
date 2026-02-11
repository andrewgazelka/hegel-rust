use super::{generate_raw, Generate};
use crate::cbor_helpers::{cbor_map, cbor_serialize};
use ciborium::Value;

pub fn unit() -> JustGenerator<()> {
    just(())
}

pub struct JustGenerator<T> {
    value: T,
    schema: Option<Value>,
}

impl<T: Clone + Send + Sync + serde::Serialize + serde::de::DeserializeOwned> Generate<T>
    for JustGenerator<T>
{
    fn generate(&self) -> T {
        self.value.clone()
    }

    fn schema(&self) -> Option<Value> {
        self.schema.clone()
    }

    fn parse_raw(&self, raw: Value) -> T {
        super::deserialize_value(raw)
    }
}

pub fn just<T: Clone + Send + Sync + serde::Serialize + serde::de::DeserializeOwned>(
    value: T,
) -> JustGenerator<T> {
    let schema = Some(cbor_map! {"const" => cbor_serialize(&value)});
    JustGenerator { value, schema }
}

pub struct JustAnyGenerator<T> {
    value: T,
}

impl<T: Clone + Send + Sync> Generate<T> for JustAnyGenerator<T> {
    fn generate(&self) -> T {
        self.value.clone()
    }
}
pub fn just_any<T: Clone + Send + Sync>(value: T) -> JustAnyGenerator<T> {
    JustAnyGenerator { value }
}

pub struct BoolGenerator;

impl Generate<bool> for BoolGenerator {
    fn generate(&self) -> bool {
        self.parse_raw(generate_raw(&self.schema().unwrap()))
    }

    fn schema(&self) -> Option<Value> {
        Some(cbor_map! {"type" => "boolean"})
    }

    fn parse_raw(&self, raw: Value) -> bool {
        super::deserialize_value(raw)
    }
}

pub fn booleans() -> BoolGenerator {
    BoolGenerator
}
