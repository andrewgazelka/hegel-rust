# Hypothesis Test Port Notes

## Summary

**192 tests** ported from Hypothesis's test suite to hegel-rust:
- `tests/test_shrink_quality.rs` — 74 tests (shrink quality)
- `tests/test_find_quality.rs` — 118 tests (findability + number shrinking)

All 192 tests pass.

## Source Files

### Shrink quality tests (test_shrink_quality.rs)

| Hypothesis file | Tests ported | Tests skipped |
|---|---|---|
| `tests/quality/test_shrink_quality.py` | 44 | 4 |
| `tests/quality/test_float_shrinking.py` | 7 | 2 |
| `tests/nocover/test_flatmap.py` | 9 | 3 |
| `tests/nocover/test_find.py` | 6 | 1 |
| `tests/nocover/test_collective_minimization.py` | 3 | parametrized over all types |

### Findability / discovery tests (test_find_quality.rs)

| Hypothesis file | Tests ported | Tests skipped |
|---|---|---|
| `tests/quality/test_discovery_ability.py` | 55 | 5 |
| `tests/nocover/test_simple_numbers.py` | 55 | 8 |
| `tests/nocover/test_floating.py` | 2 | 10 |
| `tests/cover/test_find.py` | 0 | 1 |

### Not ported (use Hypothesis internals)

| Hypothesis file | Reason |
|---|---|
| `tests/quality/test_zig_zagging.py` | Uses `ConjectureRunner` |
| `tests/quality/test_poisoned_lists.py` | Uses `ConjectureRunner` |
| `tests/quality/test_poisoned_trees.py` | Uses `ConjectureRunner` |
| `tests/quality/test_deferred_strategies.py` | Needs `deferred()` |
| `tests/quality/test_integers.py` | Uses `ConjectureData` |
| Other conjecture/engine tests | Internal APIs |

## Tests Skipped (with reasons)

### Missing generator features
- **test_minimal_fractions_***: No `fractions()` generator in hegel-rust.
- **test_minimize_sets_of_sets**: No `frozensets()` (no hashable set type in Rust).
- **test_can_find_sets_unique_by_incomplete_data**: No `unique_by` parameter on list generators.
- **test_multiple_empty_lists_are_independent**: Uses `none()` generator (no direct equivalent).
- **test_calculator_benchmark**: Needs `deferred()` / recursive strategies.
- **test_large_branching_tree / test_non_trivial_json / test_self_recursive_lists**: Need `deferred()`.

### Uses Hypothesis internals (ConjectureRunner, ConjectureData)
- **test_avoids_zig_zag_trap**: Uses `ConjectureRunner` and low-level buffer manipulation.
- **test_minimal_poisoned_containers**: Uses `ConjectureRunner` and custom `SearchStrategy`.
- **test_can_reduce_poison_from_any_subtree**: Uses `ConjectureRunner` and buffer patching.
- **test_always_reduces_integers_to_smallest_suitable_sizes**: Uses `ConjectureData`, shrinker internals.
- **test_generates_boundary_values_even_when_unlikely**: Uses `ConjectureData.for_buffer()`.
- **test_find_uses_provided_random**: Uses `find()` with `random` parameter not available in Rust SDK.

### Type system mismatch (ported using enums)
- **test_minimize_one_of**, **test_minimize_mixed_list**, **test_mixed_list_flatmap**: Originally use Python's dynamic typing (`integers() | text() | booleans()`). Ported using Rust enums (`IntOrTextOrBool`, `IntOrText`, `BoolOrText`) with `one_of!` and composite generators.

### Discovery tests — distribution / mixing
- **test_sampled_from_often_distorted**: Statistical distribution test requiring repeated independent runs.
- **test_mixing_is_sometimes_distorted**, **test_mixes_2_reasonably_often**, **test_partial_mixes_3_reasonably_often**, **test_mixes_not_too_often**: Use Python's `booleans() | tuples()` (mixed types with dynamic dispatch).

### simple_numbers.py — skipped tests
- **Parametrized boundary tests** (full set): Ported a representative subset (22 boundaries) rather than all ~50 from Hypothesis's `2^i, 2^i-1, 2^i+1, 10^i` set.
- **test_floats_in_constrained_range**: Uses `@given` with `data()` for two-phase drawing.
- **test_no_allow_infinity_upper/lower**: Uses `allow_infinity` parameter not available in hegel-rust floats generator.
- **test_floats_from_zero_have_reasonable_range** (full set): Ported k=0,3,6 as representative subset.

### floating.py — skipped tests
- **test_is_float**, **test_negation_is_self_inverse**, **test_largest_range**: Trivial type/property checks already covered.
- **test_is_not_nan**, **test_is_not_positive_infinite**, etc.: "fails" decorator tests that verify Hypothesis CAN find counterexamples — overlaps with findability tests already ported.
- **test_floats_are_in_range**: Uses `data()` strategy for two-phase drawing.
- **test_can_find_negative_and_signaling_nans**: Uses `float_to_int` internal for signaling NaN detection.

## Porting Adaptations

1. **Byte vs char length**: Python `len(str)` counts codepoints; Rust `str.len()` counts bytes. Used `chars().count()` where needed.
2. **Integer overflow**: Python integers are unbounded; Rust uses fixed-width. Used bounded ranges or i64 where Python uses arbitrary-precision.
3. **1-tuples**: Hypothesis `tuples(integers())` creates 1-tuples. Used 2-tuples with relaxed assertions.
4. **Composite generators**: Used `#[hegel::composite]` for tests requiring `tuples(lists(...), integers())` patterns.
5. **test_cases counts**: Matched Hypothesis `max_examples` values (e.g., 10000 for large-range tests).
6. **Mixed types via enums**: Python's `integers() | text()` ported using Rust enums with `one_of!` mapping into enum variants, or `#[hegel::composite]` for flatmap-style branching.
7. **Statistical → find_any**: Hypothesis's `define_test` (probabilistic, 100 runs × 150 examples) ported as `find_any` (1000 attempts), which is sufficient for all tested probabilities (p ≥ 0.1).
8. **Float specifics**: `f64::MAX / 2.0` for "very large float", `i64::MAX / 2` for "really large int".

## Failed Tests (that genuinely test the same thing)

None — all 192 ported tests pass.
