use core::{
    array,
    ops::{Add, Mul, Sub},
};

pub trait Field:
    Default + Add<Self, Output = Self> + Mul<Self, Output = Self> + Clone + From<bool>
{
}
impl<F: Default + Add<F, Output = F> + Mul<F, Output = F> + Clone + From<bool>> Field for F {}
#[derive(Clone, Copy)]
pub struct Poly<T, const D: usize>(pub [T; D], pub usize);
impl<T: Default,const D: usize> Default for Poly<T,D>{
    fn default() -> Self {
        Self(array::from_fn(|_|Default::default()), Default::default())
    }
}
impl<T: Default, const D: usize> Poly<T, D> {
    pub fn constant(v: T) -> Self {
        let mut a: [T; D] = array::from_fn(|_| T::default());
        a[0] = v;
        Self(a, 1)
    }
}
impl<T, const D: usize> Poly<T, D> {
    pub fn map<U>(self, mut f: impl FnMut(T) -> U) -> Poly<U, D> {
        Poly(self.0.map(f), self.1)
    }
    pub fn scale<U: Clone>(self, u: U) -> Poly<<T as Mul<U>>::Output, D>
    where
        T: Mul<U>,
    {
        self.map(|v| v * u.clone())
    }
}
impl<U: Clone, T: Add<U> + Clone, const D: usize> Add<Poly<U, D>> for Poly<T, D> {
    type Output = Poly<T::Output, D>;

    fn add(self, rhs: Poly<U, D>) -> Self::Output {
        Poly(
            array::from_fn(|i| self.0[i].clone() + rhs.0[i].clone()),
            self.1.max(rhs.1),
        )
    }
}
impl<U: Clone, T: Sub<U> + Clone, const D: usize> Sub<Poly<U, D>> for Poly<T, D> {
    type Output = Poly<T::Output, D>;

    fn sub(self, rhs: Poly<U, D>) -> Self::Output {
        Poly(
            array::from_fn(|i| self.0[i].clone() - rhs.0[i].clone()),
            self.1.max(rhs.1),
        )
    }
}
impl<
    U: Clone,
    T: Clone + Mul<U, Output = V>,
    V: Default + Clone + Add<V, Output = V>,
    const D: usize,
> Mul<Poly<U, D>> for Poly<T, D>
{
    type Output = Poly<V, D>;

    fn mul(self, rhs: Poly<U, D>) -> Self::Output {
        let mut x: [V; D] = array::from_fn(|_| V::default());
        for (i, a) in self.0[..self.1].iter().cloned().enumerate() {
            for (j, b) in rhs.0[..rhs.1].iter().cloned().enumerate() {
                if let Some(x) = x.get_mut(i + j) {
                    *x = x.clone() + (a.clone() * b);
                } else {
                    panic!("degree exhausted: {i} + {j}")
                }
            }
        }
        Poly(x, self.1 + rhs.1)
    }
}
