#![no_std]

use core::{array, cell::Cell, mem::take, sync::atomic::AtomicUsize, task::Poll};

use atomic_waker::AtomicWaker;
use magma_proof::{Transport, math::{Field, Poly}};
#[derive(Clone, Copy, Default)]
pub struct Entry<F>(F, F);
pub struct Prover<'a, F: Field, const B: usize, const N: usize>(
    &'a [(
        Cell<Option<Entry<F>>>,
        Cell<Poly<F, N>>,
        AtomicWaker,
        AtomicWaker,
    ); B],
    AtomicUsize,
);
pub struct Verifier<'a, F: Field, const B: usize, const N: usize>(
    &'a [(Cell<Option<Entry<F>>>, Cell<F>, AtomicWaker, AtomicWaker); B],
    AtomicUsize,
);
impl<'a, F: Field, const B: usize, const N: usize> Transport<F> for Verifier<'a, F, B, N> {
    type Wrap = (F, usize);

    async fn add(&self, wa: &Self::Wrap, wb: &Self::Wrap) -> Self::Wrap {
        (wa.0.clone() + wb.0.clone(), wa.1.max(wb.1))
    }

    async fn mul(&self, wa: &Self::Wrap, wb: &Self::Wrap) -> Self::Wrap {
        if wa.1 + wb.1 < N {
            return (wa.0.clone() * wb.0.clone(), wa.1 + wb.1);
        }
        let mut wa = wa.clone();
        let mut wb = wb.clone();
        for i in 0..=1 {
            let p = if i == 0usize { &mut wa } else { &mut wb };
            let i = self.1.fetch_add(1, core::sync::atomic::Ordering::SeqCst) % B;
            let Entry(q, delta) = core::future::poll_fn(|cx| match &self.0[i] {
                (a, _, b, _) => match a.take() {
                    Some(a) => Poll::Ready(a),
                    None => {
                        b.register(cx.waker());
                        Poll::Pending
                    }
                },
            })
            .await;
            p.0 = q;
            p.1 = 2;
            if wa.1 + wb.1 < N {
                return (wa.0.clone() * wb.0.clone(), wa.1 + wb.1);
            }
        }
        unreachable!()
    }

    async fn scale(&self, wa: &Self::Wrap, b: F) -> Self::Wrap {
        (wa.0.clone() * b, wa.1)
    }

    async fn wrap(&self, a: F) -> Self::Wrap {
        (a, 1)
    }
}
impl<'a, F: Field, const B: usize, const N: usize> Transport<F> for Prover<'a, F, B, N> {
    type Wrap = Poly<F, N>;

    async fn add(&self, wa: &Self::Wrap, wb: &Self::Wrap) -> Self::Wrap {
        wa.clone() + wb.clone()
    }

    async fn mul(&self, wa: &Self::Wrap, wb: &Self::Wrap) -> Self::Wrap {
        if wa.1 + wb.1 < N {
            return wa.clone() * wb.clone();
        }
        let mut wa = wa.clone();
        let mut wb = wb.clone();
        for i in 0..=1 {
            let p = if i == 0usize { &mut wa } else { &mut wb };
            let i = self.1.fetch_add(1, core::sync::atomic::Ordering::SeqCst) % B;
            let Entry(u, v) = core::future::poll_fn(|cx| match &self.0[i] {
                (a, _, b, _) => match a.take() {
                    Some(a) => Poll::Ready(a),
                    None => {
                        b.register(cx.waker());
                        Poll::Pending
                    }
                },
            })
            .await;
            let val = p.0[p.1 - 1].clone();
            p.0[p.1 - 1] = val.clone() + u;
            self.0[i].1.replace(take(&mut *p));
            self.0[i].3.wake();
            p.0[0] = v;
            p.0[1] = val;
            p.1 = 2;
            if wa.1 + wb.1 < N {
                return wa.clone() * wb.clone();
            }
        }
        unreachable!()
    }

    async fn scale(&self, wa: &Self::Wrap, b: F) -> Self::Wrap {
        wa.clone().scale(b)
    }

    async fn wrap(&self, a: F) -> Self::Wrap {
        let mut arr: [F; N] = array::from_fn(|_| F::default());
        arr[0] = a;
        let p = Poly(arr, 1);
        p
    }
}
