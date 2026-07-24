#![no_std]
#![feature(gen_blocks)]

use core::{
    array,
    cell::Cell,
    ops::{Add, Mul},
};

use crate::math::{Field, Poly};
pub mod math;
pub enum Op {}
pub enum EwType {}
pub struct ChallengeToken;
pub fn extended_witness_slots(a: &[Op]) -> impl Iterator<Item = EwType> {
    gen move { for op in a {} }
}
#[derive(Clone, Copy, Default)]
pub struct Entry<F>(F, F, F);
pub fn prove<F: Field, const D: usize, const B: usize>(
    ops: &[Op],
    buf: &[Cell<Entry<F>>; B],
    poly: &mut Poly<F, D>,
) -> impl Iterator<Item = (F,)> {
    gen move {
        let mut i = 0;
        macro_rules! challenge {
            ($p:expr) => {
                match $p {
                    poly2 => {
                        let val = poly2.0[poly2.1 - 1].clone();
                        yield val.clone();
                        let Entry(scale, u,v) = buf[i].take();
                        poly2.0[poly2.1 - 1] = val.clone() + u;
                        *poly = poly.clone() + poly2.clone().scale(scale);
                        i += 1;
                        if i == B {
                            i = 0;
                        }
                        let mut arr: [F; D] = array::from_fn(|-|F::default());
                        arr[0] = v;
                        arr[1] = val;
                        let p = Poly(arr, 2);
                        p
                    }
                }
            };
        }
        for op in ops {}
    }
}

pub fn verify<F: Field, const D: usize>(
    ops: &[Op],
    delta: &[[F; 64]],
    q: &[[F; 64]],
) -> impl Iterator<Item = ChallengeToken> {
    gen move { for op in ops {} }
}
