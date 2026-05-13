use super::*;

// ── IntegerChoice::simplest ─────────────────────────────────────────────────
//
// Ports of pbtkit/tests/test_core.py::test_integer_choice_simplest.

#[test]
fn integer_choice_simplest_spans_zero() {
    assert_eq!(
        IntegerChoice {
            min_value: -10,
            max_value: 10,
            shrink_towards: 0,
        }
        .simplest(),
        0
    );
}

#[test]
fn integer_choice_simplest_all_positive() {
    assert_eq!(
        IntegerChoice {
            min_value: 5,
            max_value: 100,
            shrink_towards: 0,
        }
        .simplest(),
        5
    );
}

#[test]
fn integer_choice_simplest_all_negative() {
    assert_eq!(
        IntegerChoice {
            min_value: -100,
            max_value: -5,
            shrink_towards: 0,
        }
        .simplest(),
        -5
    );
}

// ── IntegerChoice::unit ─────────────────────────────────────────────────────
//
// Ports of pbtkit/tests/test_core.py::test_integer_choice_unit.

#[test]
fn integer_choice_unit_spans_zero() {
    assert_eq!(
        IntegerChoice {
            min_value: -10,
            max_value: 10,
            shrink_towards: 0,
        }
        .unit(),
        1
    );
}

#[test]
fn integer_choice_unit_all_positive() {
    assert_eq!(
        IntegerChoice {
            min_value: 5,
            max_value: 100,
            shrink_towards: 0,
        }
        .unit(),
        6
    );
}

#[test]
fn integer_choice_unit_all_negative() {
    // simplest is at the top of the range, so unit should fall back to
    // simplest - 1 = -6.
    assert_eq!(
        IntegerChoice {
            min_value: -100,
            max_value: -5,
            shrink_towards: 0,
        }
        .unit(),
        -6
    );
}

#[test]
fn integer_choice_unit_single_value_range() {
    // When the range is a single value, unit falls back to simplest.
    assert_eq!(
        IntegerChoice {
            min_value: 5,
            max_value: 5,
            shrink_towards: 0,
        }
        .unit(),
        5
    );
}

#[test]
#[should_panic(expected = "ChoiceKind::to_index: kind/value mismatch")]
fn choice_kind_to_index_panics_on_kind_value_mismatch() {
    // Asking an Integer kind to index a Boolean value is a programmer error;
    // ChoiceKind::to_index must panic loudly rather than return a bogus index.
    let kind = ChoiceKind::Integer(IntegerChoice {
        min_value: 0,
        max_value: 100,
        shrink_towards: 0,
    });
    let _ = kind.to_index(&ChoiceValue::Boolean(true));
}

// ── ChoiceKind::max_children ──────────────────────────────────────────────
//
// Ports of `compute_max_children` tests from
// `hypothesis-python/tests/conjecture/test_utils.py` plus hegel-specific
// checks for the choice kinds hegel's native engine actually records.

fn bu(n: u64) -> crate::native::bignum::BigUint {
    crate::native::bignum::BigUint::from(n)
}

#[test]
fn integer_bounded_range_gives_exact_count() {
    let kind = ChoiceKind::Integer(IntegerChoice {
        min_value: 0,
        max_value: 200,
        shrink_towards: 0,
    });
    assert_eq!(kind.max_children(), bu(201));
}

#[test]
fn integer_negative_range_gives_exact_count() {
    let kind = ChoiceKind::Integer(IntegerChoice {
        min_value: -10,
        max_value: 10,
        shrink_towards: 0,
    });
    assert_eq!(kind.max_children(), bu(21));
}

#[test]
fn integer_full_i128_range_is_two_pow_128() {
    let kind = ChoiceKind::Integer(IntegerChoice {
        min_value: i128::MIN,
        max_value: i128::MAX,
        shrink_towards: 0,
    });
    // 2^128 = u128::MAX + 1.
    let expected = crate::native::bignum::BigUint::from(u128::MAX) + bu(1);
    assert_eq!(kind.max_children(), expected);
}

#[test]
fn boolean_is_always_two() {
    assert_eq!((ChoiceKind::Boolean(BooleanChoice)).max_children(), bu(2));
}
