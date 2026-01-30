mod common;

use common::utils::find_any;
use hegel::gen::{self, Generate};

#[test]
fn test_sampled_from_returns_element_from_list() {
    hegel::hegel(|| {
        let options = gen::vecs(gen::integers::<i32>()).generate();
        let value = gen::sampled_from(options.clone()).generate();
        assert!(options.contains(&value));
    });
}

#[test]
fn test_sampled_from_strings() {
    hegel::hegel(|| {
        let options = gen::vecs(gen::text()).generate();
        let value = gen::sampled_from(options.clone()).generate();
        assert!(options.contains(&value));
    });
}

#[test]
fn test_sampled_from_slice() {
    hegel::hegel(|| {
        let options = gen::vecs(gen::integers::<i32>()).generate();
        let value = gen::sampled_from_slice(&options).generate();
        assert!(options.contains(&value));
    });
}

#[test]
fn test_optional_can_generate_some() {
    find_any(gen::optional(gen::integers::<i32>()), |v| v.is_some());
}

#[test]
fn test_optional_can_generate_none() {
    find_any(gen::optional(gen::integers::<i32>()), |v| v.is_none());
}

#[test]
fn test_optional_respects_inner_generator_bounds() {
    hegel::hegel(|| {
        let value: Option<i32> =
            gen::optional(gen::integers().with_min(10).with_max(20)).generate();
        if let Some(n) = value {
            assert!(n >= 10 && n <= 20);
        }
    });
}

#[test]
fn test_one_of_returns_value_from_one_generator() {
    hegel::hegel(|| {
        let value: i32 = hegel::one_of!(
            gen::integers().with_min(0).with_max(10),
            gen::integers().with_min(100).with_max(110),
        )
        .generate();
        assert!((0..=10).contains(&value) || (100..=110).contains(&value));
    });
}

#[test]
fn test_one_of_with_different_types_via_map() {
    hegel::hegel(|| {
        let value: String = hegel::one_of!(
            gen::integers::<i32>()
                .with_min(0)
                .with_max(100)
                .map(|n| format!("number: {}", n)),
            gen::text()
                .with_min_size(1)
                .with_max_size(10)
                .map(|s| format!("text: {}", s)),
        )
        .generate();
        assert!(value.starts_with("number: ") || value.starts_with("text: "));
    });
}

#[test]
fn test_one_of_many() {
    hegel::hegel(|| {
        let generators: Vec<_> = (0..10).map(|i| gen::just(i).boxed()).collect();
        let value: i32 = gen::one_of(generators).generate();
        assert!((0..10).contains(&value));
    });
}
