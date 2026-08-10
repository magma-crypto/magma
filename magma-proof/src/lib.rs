#![no_std]

use core::{
    array,
    cell::Cell,
    marker::PhantomData,
    mem::take,
    ops::{Add, Mul},
    sync::atomic::AtomicUsize,
    task::{Poll, Waker},
};

use crate::math::{Field, Poly};
pub mod math;
pub trait Transport<F: Field> {
    type Wrap;
    async fn add(&self, wa: &Self::Wrap, wb: &Self::Wrap) -> Self::Wrap;
    async fn mul(&self, wa: &Self::Wrap, wb: &Self::Wrap) -> Self::Wrap;
    async fn scale(&self, wa: &Self::Wrap, b: F) -> Self::Wrap;
    async fn wrap(&self, a: F) -> Self::Wrap;
}
pub struct Executor;
impl<F: Field> Transport<F> for Executor {
    type Wrap = F;

    async fn add(&self, wa: &Self::Wrap, wb: &Self::Wrap) -> Self::Wrap {
        wa.clone() + wb.clone()
    }

    async fn mul(&self, wa: &Self::Wrap, wb: &Self::Wrap) -> Self::Wrap {
        wa.clone() * wb.clone()
    }

    async fn scale(&self, wa: &Self::Wrap, b: F) -> Self::Wrap {
        wa.clone() * b
    }

    async fn wrap(&self, a: F) -> Self::Wrap {
        a
    }
}
