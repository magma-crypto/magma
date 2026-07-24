#![no_std]
#![feature(gen_blocks)]
extern crate alloc;
use core::{
    array,
    cell::Cell,
    ops::{Add, Mul},
};

use alloc::vec::Vec;

use crate::math::{Field, Poly};
pub mod math;
pub enum Op<F> {
    MatMul { num_pops: usize, elems: Vec<F> },
}
pub enum EwType {}
pub struct ChallengeToken;
pub fn extended_witness_slots<F: Field>(a: &[Op<F>]) -> impl Iterator<Item = EwType> {
    gen move {
        for op in a {
            match op {
                Op::MatMul { num_pops, elems } => {}
            }
        }
    }
}
#[derive(Clone, Copy, Default)]
pub struct Entry<F>(F, F, F);
pub fn prove<F: Field, const D: usize, const B: usize>(
    ops: &[Op<F>],
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
        let mut stack: Vec<Poly<F, D>> = Vec::new();
        for op in ops {
            match op {
                Op::MatMul { num_pops, elems } => {
                    let mut pops = (0..*num_pops)
                        .filter_map(|a| stack.pop())
                        .collect::<Vec<_>>();
                    pops.reverse();
                    for c in elems.chunks_exact(*num_pops) {
                        stack.push(
                            c.iter()
                                .zip(pops.iter())
                                .fold(Default::default(), |a, (b, c)| {
                                    a + (c.clone().scale(b.clone()))
                                }),
                        );
                    }
                }
            }
        }
    }
}

pub fn verify<F: Field, const D: usize, const B: usize>(
    ops: &[Op<F>],
    buf: &[Cell<Entry<F>>; B],
    q_stor: &mut F,
) -> impl Iterator<Item = ChallengeToken> {
    gen move {
        let mut i = 0;
        macro_rules! resp {
            () => {
                yield ChallengeToken;
                let Entry(scale, q, delta) = buf[i].take();
                *q_stor = q_stor.clone() + scale * q.clone();
                i += 1;
                if i == B {
                    i = 0;
                }
                q
            };
        }
        let mut stack: Vec<F> = Vec::new();
        for op in ops {
            match op {
                Op::MatMul { num_pops, elems } => {
                    let mut pops = (0..*num_pops)
                        .filter_map(|a| stack.pop())
                        .collect::<Vec<_>>();
                    pops.reverse();
                    for c in elems.chunks_exact(*num_pops) {
                        stack.push(
                            c.iter()
                                .zip(pops.iter())
                                .fold(Default::default(), |a, (b, c)| a + (b.clone() * c.clone())),
                        );
                    }
                }
            }
        }
    }
}
