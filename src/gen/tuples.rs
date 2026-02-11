use super::{generate_raw, group, labels, Generate};
use crate::cbor_helpers::{cbor_array, cbor_map};
use ciborium::Value;

pub struct Tuple2Generator<G1, G2> {
    gen1: G1,
    gen2: G2,
}

impl<T1, T2, G1, G2> Generate<(T1, T2)> for Tuple2Generator<G1, G2>
where
    G1: Generate<T1>,
    G2: Generate<T2>,
{
    fn generate(&self) -> (T1, T2) {
        if let Some(schema) = self.schema() {
            self.parse_raw(generate_raw(&schema))
        } else {
            group(labels::TUPLE, || {
                let v1 = self.gen1.generate();
                let v2 = self.gen2.generate();
                (v1, v2)
            })
        }
    }

    fn schema(&self) -> Option<Value> {
        let s1 = self.gen1.schema()?;
        let s2 = self.gen2.schema()?;

        Some(cbor_map! {
            "type" => "tuple",
            "elements" => cbor_array![s1, s2]
        })
    }

    fn parse_raw(&self, raw: Value) -> (T1, T2) {
        let arr = match raw {
            Value::Array(arr) => arr,
            _ => panic!("Expected array from tuple schema, got {:?}", raw),
        };
        let mut iter = arr.into_iter();
        let v1 = self
            .gen1
            .parse_raw(iter.next().expect("tuple missing element 0"));
        let v2 = self
            .gen2
            .parse_raw(iter.next().expect("tuple missing element 1"));
        (v1, v2)
    }
}

pub fn tuples<T1, T2, G1: Generate<T1>, G2: Generate<T2>>(
    gen1: G1,
    gen2: G2,
) -> Tuple2Generator<G1, G2> {
    Tuple2Generator { gen1, gen2 }
}

pub struct Tuple3Generator<G1, G2, G3> {
    gen1: G1,
    gen2: G2,
    gen3: G3,
}

impl<T1, T2, T3, G1, G2, G3> Generate<(T1, T2, T3)> for Tuple3Generator<G1, G2, G3>
where
    G1: Generate<T1>,
    G2: Generate<T2>,
    G3: Generate<T3>,
{
    fn generate(&self) -> (T1, T2, T3) {
        if let Some(schema) = self.schema() {
            self.parse_raw(generate_raw(&schema))
        } else {
            group(labels::TUPLE, || {
                let v1 = self.gen1.generate();
                let v2 = self.gen2.generate();
                let v3 = self.gen3.generate();
                (v1, v2, v3)
            })
        }
    }

    fn schema(&self) -> Option<Value> {
        let s1 = self.gen1.schema()?;
        let s2 = self.gen2.schema()?;
        let s3 = self.gen3.schema()?;

        Some(cbor_map! {
            "type" => "tuple",
            "elements" => cbor_array![s1, s2, s3]
        })
    }

    fn parse_raw(&self, raw: Value) -> (T1, T2, T3) {
        let arr = match raw {
            Value::Array(arr) => arr,
            _ => panic!("Expected array from tuple schema, got {:?}", raw),
        };
        let mut iter = arr.into_iter();
        let v1 = self
            .gen1
            .parse_raw(iter.next().expect("tuple missing element 0"));
        let v2 = self
            .gen2
            .parse_raw(iter.next().expect("tuple missing element 1"));
        let v3 = self
            .gen3
            .parse_raw(iter.next().expect("tuple missing element 2"));
        (v1, v2, v3)
    }
}

pub fn tuples3<T1, T2, T3, G1: Generate<T1>, G2: Generate<T2>, G3: Generate<T3>>(
    gen1: G1,
    gen2: G2,
    gen3: G3,
) -> Tuple3Generator<G1, G2, G3> {
    Tuple3Generator { gen1, gen2, gen3 }
}
