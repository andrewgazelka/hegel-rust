//! Arbitrary-precision integer re-exports for the native engine.
//!
//! pbtkit leans on Python's bignum `int` in its shortlex index arithmetic
//! (`to_index` / `from_index` / `max_index` in `core.py`). The full
//! `IntegerChoice` index space (`u128`-wide) exceeds fixed-width arithmetic
//! during accumulation, so the port routes those calls through a bignum type.
//!
//! Routing all big-integer arithmetic through this module keeps the
//! backend choice localised: swapping `num-bigint` for e.g. `malachite`
//! later would only touch this file.

pub use num_bigint::BigUint;
pub use num_traits::Zero;
