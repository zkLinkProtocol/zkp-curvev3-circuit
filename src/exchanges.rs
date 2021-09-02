//!
//! The swap consequences calculation.
//!

use crate::{Address, N, Balance, PRECISION_MUL, ZERO};
use crate::invariant::calculate;
use primitive_types::U256;

/// The token being withdrawn balance get_y the swap.
pub fn get_y(
    mut balances: [Balance; N],
    amplifier: u64,
    d: U256,
    token_x_idx: usize,
    token_y_idx: Option<usize>,
    after_x: Option<Balance>,
    is_get_y_d: bool
) -> Balance {
    assert!((is_get_y_d && token_y_idx.is_none()) || (!is_get_y_d && token_y_idx.is_some()));
    assert_ne!(token_x_idx, token_y_idx, "Cannot exchange between the same coins");
    assert!(0 <= token_x_idx && token_x_idx < N, "There is no x token Id in the pool");
    assert!(0 <= token_y_idx.unwrap() && token_y_idx.unwrap() < N, "There is no y token Id in the pool");
    
    //  
    balances.iter_mut().for_each(|balance| *balance *= PRECISION_MUL);

    let an: U256 = (amplifier * N as u64).into();

    // let x_magnitude_diff = tokens[token_x_idx].magnitude_diff() * PRECISION_MUL;
    // let y_magnitude_diff = tokens[token_y_idx].magnitude_diff() * PRECISION_MUL;

    let mut c = d;
    let mut s: Balance = ZERO;
    // let after_x_p = after_x * x_magnitude_diff;

    for i in 0..N {
        let after_x_p = if is_get_y_d{
            if i != token_x_idx {
                balances[i]
            } else { continue; }
        } else {
            if i == token_x_idx {
                after_x
            } else if i != token_y_idx.unwrap() {
                balances[i]
            } else { continue; }
        };
        s += after_x_p;
        c = c * d / (after_x_p * N)
    }

    c = c * d / (an * N);

    let b: Balance = s + d / an;
    let mut y = d;
    for _ in 0..255{
        let y_next = (y * y + c) / (2 * y + b - d);

        if (y > y_next && y - y_next > U256::one())
            || (y <= y_next && y_next - y > U256::one()) {
            return y_next;
        }
        y = y_next;
    }

    y
}

#[test]
fn ok_equal_precision() {
    let new_y = get_y(
        [1_000 as Balance, 1_000 as Balance],
        100 as u64,
        0,
        Some(1),
        1_050 as Balance,
        false,
    );

    assert_eq!(new_y, 950, "The balance get_y withdrawal does not match the reference");
}

#[test]
fn ok_equal_precision_amplified() {
    let new_y = get_y(
        [1_E6 as Balance, 1_E6 as Balance],
        100 as u64,
        0,
        Some(1),
        1_900_000 as Balance,
        false,
    );

    assert_eq!(new_y, 130_370, "The balance get_y withdrawal does not match the reference");
}

#[test]
fn ok_different_precision_lesser_bigger() {
    let new_y = get_y(
        [1_E6 as Balance, 1_E18 as Balance],
        100 as u64,
        0,
        Some(1),
        1.050_E6 as Balance,
        false,
    );

    assert_eq!(new_y, 950_024_800_946_586_013, "The balance get_y withdrawal does not match the reference");
}

#[test]
fn ok_different_precision_lesser_bigger_amplified() {
    let new_y = get_y(
        [1_E6 as Balance, 1_E18 as Balance],
        100 as u64,
        0,
        Some(1),
        1.950_E6 as Balance,
        false,
    );

    assert_eq!(new_y, 94_351_900_636_131_207, "The balance get_y withdrawal does not match the reference");
}

#[test]
fn ok_different_precision_bigger_lesser() {
    let new_y = get_y(
        [1_E18 as Balance, 1_E6 as Balance],
        100 as u64,
        0,
        Some(1),
        1.050_E18 as Balance,
        false,
    );

    assert_eq!(new_y, 950_024, "The balance get_y withdrawal does not match the reference");
}

#[test]
fn ok_different_precision_bigger_lesser_amplified() {
    let new_y = get_y(
        [1_E18 as Balance, 1_E6 as Balance],
        100 as u64,
        0,
        Some(1),
        1.950_E18 as Balance,
        false,
    );

    assert_eq!(new_y, 94_351, "The balance get_y withdrawal does not match the reference");
}

#[test]
#[should_panic]
fn error_same_tokens() {
    get_y(
        [1E3 as Balance, 1E3 as Balance],
        100 as u64,
        Some(1),
        Some(1),
        100 as Balance,
        false,
    );
}