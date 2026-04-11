use hegel::generators::{self as gs, BoxedGenerator, Generator};
use hegel::{Hegel, Settings};
use hegel_conformance::{get_test_cases, make_non_basic, write};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Deserialize)]
struct Params {
    min_size: usize,
    max_size: Option<usize>,
    min_value: Option<i32>,
    max_value: Option<i32>,
    #[serde(default)]
    mode: String,
    #[serde(default)]
    unique: bool,
}

#[derive(Serialize)]
struct Metrics {
    size: usize,
    min_element: Option<i32>,
    max_element: Option<i32>,
    all_unique: bool,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: test_lists '<json_params>'");
        std::process::exit(1);
    }

    let params: Params = serde_json::from_str(&args[1]).unwrap_or_else(|e| {
        eprintln!("Failed to parse params: {}", e);
        std::process::exit(1);
    });

    Hegel::new(move |tc| {
        let mut g = gs::integers::<i32>();
        if let Some(min) = params.min_value {
            g = g.min_value(min);
        }
        if let Some(max) = params.max_value {
            g = g.max_value(max);
        }

        let elem_gen: BoxedGenerator<'static, i32> = if params.mode == "non_basic" {
            make_non_basic(g)
        } else {
            g.boxed()
        };

        let mut vec_gen = gs::vecs(elem_gen)
            .min_size(params.min_size)
            .unique(params.unique);
        if let Some(max) = params.max_size {
            vec_gen = vec_gen.max_size(max);
        }

        let list = tc.draw(vec_gen);

        let size = list.len();
        let (min_element, max_element) = if list.is_empty() {
            (None, None)
        } else {
            (list.iter().min().copied(), list.iter().max().copied())
        };
        let unique_count = {
            let mut seen = std::collections::HashSet::new();
            list.iter().filter(|x| seen.insert(*x)).count()
        };

        write(&Metrics {
            size,
            min_element,
            max_element,
            all_unique: unique_count == size,
        });
    })
    .settings(Settings::new().test_cases(get_test_cases()))
    .run();
}
