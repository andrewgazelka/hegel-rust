//! Hegel is a property-based testing library for Rust. Hegel is based on [Hypothesis](https://github.com/hypothesisworks/hypothesis), using the [Hegel](https://hegel.dev/) protocol.
//!
//! # Getting Started
//!
//! ## Prerequisites
//!
//! You will need [`uv`](https://github.com/astral-sh/uv) installed and on your PATH.
//!
//! ## Install Hegel
//!
//! Add `hegeltest` to your `Cargo.toml` as a dev dependency:
//!
//! ```toml
//! [dev-dependencies]
//! hegeltest = "0.1.0"
//! ```
//!
//! ## Write your first test
//!
//! Create a new test in the project's `tests/` directory:
//!
//! ```no_run
//! use hegel::TestCase;
//! use hegel::generators::integers;
//!
//! #[hegel::test]
//! fn test_integer_self_equality(tc: TestCase) {
//!     let n = tc.draw(integers::<i32>());
//!     assert_eq!(n, n); // integers should always be equal to themselves
//! }
//! ```
//!
//! Now run the test using `cargo test --test <filename>`. You should see that this test passes.
//!
//! The `#[hegel::test]` attribute runs your test many times (100, by default). The test
//! function takes a [`TestCase`] parameter, which provides a [`draw`](TestCase::draw) method
//! for drawing different values. This test draws a random integer and checks that it is equal
//! to itself.
//!
//! Next, try a test that fails:
//!
//! ```no_run
//! # use hegel::TestCase;
//! # use hegel::generators::integers;
//! #[hegel::test]
//! fn test_integers_always_below_50(tc: TestCase) {
//!     let n = tc.draw(integers::<i32>());
//!     assert!(n < 50); // this will fail!
//! }
//! ```
//!
//! This test asserts that any integer is less than 50, which is obviously incorrect. Hegel
//! will find a test case that makes this assertion fail, and then shrink it to find the
//! smallest counterexample — in this case, `n = 50`.
//!
//! To fix this test, you can constrain the integers you generate with the `min_value` and
//! `max_value` methods:
//!
//! ```no_run
//! # use hegel::TestCase;
//! # use hegel::generators::integers;
//! #[hegel::test]
//! fn test_bounded_integers_always_below_50(tc: TestCase) {
//!     let n = tc.draw(integers::<i32>()
//!         .min_value(0)
//!         .max_value(49));
//!     assert!(n < 50);
//! }
//! ```
//!
//! ## Use generators
//!
//! Hegel provides a rich library of generators that you can use out of the box. There are
//! primitive generators, such as [`integers`](generators::integers),
//! [`floats`](generators::floats), and [`text`](generators::text), and combinators that allow
//! you to make generators out of other generators, such as [`vecs`](generators::vecs) and
//! [`tuples`](generators::tuples).
//!
//! For example, you can use [`vecs`](generators::vecs) to generate a vector of integers:
//!
//! ```no_run
//! # use hegel::TestCase;
//! # use hegel::generators::integers;
//! use hegel::generators::vecs;
//!
//! #[hegel::test]
//! fn test_append_increases_length(tc: TestCase) {
//!     let mut vector = tc.draw(vecs(integers::<i32>()));
//!     let initial_length = vector.len();
//!     vector.push(tc.draw(integers::<i32>()));
//!     assert!(vector.len() > initial_length);
//! }
//! ```
//!
//! You can also define custom generators. For example, say you have a `Person` struct:
//!
//! ```ignore
//! #[derive(Debug)]
//! struct Person {
//!     age: i32,
//!     name: String,
//! }
//! ```
//!
//! You can use the [`composite`] macro to create a generator for this struct:
//!
//! ```ignore
//! use hegel::generators::text;
//!
//! #[hegel::composite]
//! fn generate_person(tc: TestCase) -> Person {
//!     let age = tc.draw(integers::<i32>());
//!     let name = tc.draw(text());
//!     Person { age, name }
//! }
//! ```
//!
//! Note that you can feed the results of a [`draw`](TestCase::draw) to subsequent calls.
//! For example, say that you extend `Person` to include a `driving_license` field:
//!
//! ```ignore
//! use hegel::generators::booleans;
//!
//! #[hegel::composite]
//! fn generate_person(tc: TestCase) -> Person {
//!     let age = tc.draw(integers::<i32>());
//!     let name = tc.draw(text());
//!     let driving_license = if age >= 18 {
//!         tc.draw(booleans())
//!     } else {
//!         false
//!     };
//!     Person { age, name, driving_license }
//! }
//! ```
//!
//! ## Debug your failing test cases
//!
//! Use the [`note`](TestCase::note) method to attach debug information:
//!
//! ```no_run
//! # use hegel::TestCase;
//! # use hegel::generators::integers;
//! #[hegel::test]
//! fn test_with_notes(tc: TestCase) {
//!     let x = tc.draw(integers::<i32>());
//!     let y = tc.draw(integers::<i32>());
//!     tc.note(&format!("x + y = {}, y + x = {}", x + y, y + x));
//!     assert_eq!(x + y, y + x);
//! }
//! ```
//!
//! Notes only appear when Hegel replays the minimal failing example.
//!
//! ## Configuration
//!
//! Hegel tests can be configured:
//!
//! ```no_run
//! # use hegel::TestCase;
//! # use hegel::generators::integers;
//! use hegel::Verbosity;
//!
//! #[hegel::test(test_cases = 500, verbosity = Verbosity::Verbose)]
//! fn test_with_options(tc: TestCase) {
//!     let n = tc.draw(integers::<i32>());
//!     assert!(n + 0 == n);
//! }
//! ```
//!
//! # Generators
//!
//! ## Primitives
//!
//! ```no_run
//! use hegel::generators::{unit, booleans, just};
//!
//! #[hegel::test]
//! fn my_test(tc: hegel::TestCase) {
//!     let _: () = tc.draw(unit());
//!     let b: bool = tc.draw(booleans());
//!     let n: i32 = tc.draw(just(42));
//! }
//! ```
//!
//! ## Numbers
//!
//! ```no_run
//! use hegel::generators::{integers, floats};
//!
//! #[hegel::test]
//! fn my_test(tc: hegel::TestCase) {
//!     // integers
//!     let i: i32 = tc.draw(integers::<i32>());
//!     let bounded: i32 = tc.draw(integers().min_value(0).max_value(100));
//!
//!     // floats
//!     let f: f64 = tc.draw(floats::<f64>());
//!     let bounded: f64 = tc.draw(floats()
//!         .min_value(0.0)
//!         .max_value(1.0)
//!         .exclude_min()
//!         .exclude_max());
//! }
//! ```
//!
//! ## Strings
//!
//! ```no_run
//! use hegel::generators::{text, from_regex, emails, urls, ip_addresses};
//!
//! #[hegel::test]
//! fn my_test(tc: hegel::TestCase) {
//!     let s: String = tc.draw(text());
//!     let bounded: String = tc.draw(text().min_size(1).max_size(100));
//!
//!     // strings matching regex
//!     let pattern: String = tc.draw(from_regex(r"[a-z]{3}-[0-9]{3}"));
//!
//!     let email: String = tc.draw(emails());
//!     let url: String = tc.draw(urls());
//!     let ip: String = tc.draw(ip_addresses().v4());
//! }
//! ```
//!
//! ## Collections
//!
//! ```no_run
//! use hegel::generators::{vecs, hashsets, hashmaps, integers, text};
//! use std::collections::{HashSet, HashMap};
//!
//! #[hegel::test]
//! fn my_test(tc: hegel::TestCase) {
//!     let vec: Vec<i32> = tc.draw(vecs(integers()).min_size(1));
//!     let set: HashSet<i32> = tc.draw(hashsets(integers()));
//!     let map: HashMap<String, i32> = tc.draw(hashmaps(text(), integers()));
//! }
//! ```
//!
//! ## Combinators
//!
//! ```no_run
//! use hegel::generators::{sampled_from, integers, optional};
//!
//! #[hegel::test]
//! fn my_test(tc: hegel::TestCase) {
//!     // sample from a collection of values
//!     let color: &str = tc.draw(sampled_from(vec!["red", "green", "blue"]));
//!
//!     // sample a value from one of multiple generators
//!     let n: i32 = tc.draw(hegel::one_of!(
//!         integers::<i32>().min_value(0).max_value(10),
//!         integers::<i32>().min_value(100).max_value(110),
//!     ));
//!
//!     let opt: Option<i32> = tc.draw(optional(integers()));
//! }
//! ```
//!
//! ## Transformations
//!
//! ```no_run
//! use hegel::generators::{integers, text, Generator};
//!
//! #[hegel::test]
//! fn my_test(tc: hegel::TestCase) {
//!     // inline transformation of generated values
//!     let squared: i32 = tc.draw(integers::<i32>()
//!         .min_value(1)
//!         .max_value(10)
//!         .map(|x| x * x));
//!
//!     // filter out invalid values
//!     let even: i32 = tc.draw(integers::<i32>()
//!         .filter(|x| x % 2 == 0));
//! }
//! ```
//!
//! # Deriving generators
//!
//! `#[derive(DefaultGenerator)]` automatically derives a generator for structs. It can be combined with [`generators::default`]:
//!
//! ```no_run
//! use hegel::DefaultGenerator;
//! use hegel::generators::{default, integers};
//!
//! #[derive(DefaultGenerator, Debug)]
//! struct Person {
//!     name: String,
//!     age: u32,
//! }
//!
//! #[hegel::test]
//! fn my_test(tc: hegel::TestCase) {
//!     let person: Person = tc.draw(default::<Person>());
//!
//!     // customize fields
//!     let person: Person = tc.draw(default::<Person>()
//!         .age(integers().min_value(0).max_value(120)));
//! }
//! ```
//!
//! For external types, use [`derive_generator!`]:
//!
//! ```ignore
//! use hegel::derive_generator;
//! use hegel::generators::default;
//!
//! derive_generator!(Point { x: f64, y: f64 });
//!
//! let point: Point = tc.draw(default::<Point>());
//! ```
//!
//! # Assumptions
//!
//! Use [`TestCase::assume`] to reject invalid test inputs:
//!
//! ```no_run
//! use hegel::generators::integers;
//!
//! #[hegel::test]
//! fn my_test(tc: hegel::TestCase) {
//!     let age: u32 = tc.draw(integers());
//!     tc.assume(age >= 18);
//!     // Test logic for adults only...
//! }
//! ```
//!
//! # Feature flags
//!
//! - **`rand`**: Enables [`generators::randoms()`] for generating structs that implement [`rand::RngCore`].

#![forbid(future_incompatible)]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub(crate) mod antithesis;
pub(crate) mod cbor_utils;
pub(crate) mod control;
pub mod generators;
pub(crate) mod protocol;
pub(crate) mod runner;
pub mod stateful;
mod test_case;

pub use control::currently_in_test_context;
pub use generators::Generator;
pub use test_case::TestCase;

// re-export for macro use
#[doc(hidden)]
pub use ciborium;
#[doc(hidden)]
pub use paste;
#[doc(hidden)]
pub use test_case::{__IsTestCase, __assert_is_test_case, generate_from_schema, generate_raw};

// re-export public api
#[doc(hidden)]
pub use antithesis::TestLocation;
pub use hegel_macros::DefaultGenerator;
pub use hegel_macros::composite;
pub use hegel_macros::state_machine;
pub use hegel_macros::test;
pub use runner::{HealthCheck, Hegel, Settings, Verbosity, hegel};
