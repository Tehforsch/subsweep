use rand::distributions::uniform::SampleBorrow;
use rand::distributions::uniform::SampleUniform;
use rand::distributions::uniform::UniformFloat;
use rand::distributions::uniform::UniformSampler;
use rand::prelude::*;

use super::Dimension;
use super::Quantity;

#[derive(Clone, Copy, Debug)]
pub struct UniformQuantity<S, const D: Dimension>(UniformFloat<S>);

impl<const D: Dimension> UniformSampler for UniformQuantity<f64, D> {
    type X = Quantity<f64, D>;
    fn new<B1, B2>(low: B1, high: B2) -> Self
    where
        B1: SampleBorrow<Self::X> + Sized,
        B2: SampleBorrow<Self::X> + Sized,
    {
        UniformQuantity::<f64, D>(UniformFloat::<f64>::new(low.borrow().0, high.borrow().0))
    }
    fn new_inclusive<B1, B2>(low: B1, high: B2) -> Self
    where
        B1: SampleBorrow<Self::X> + Sized,
        B2: SampleBorrow<Self::X> + Sized,
    {
        UniformQuantity::<f64, D>(UniformFloat::<f64>::new_inclusive(
            low.borrow().0,
            high.borrow().0,
        ))
    }
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Self::X {
        Quantity::<f64, D>(self.0.sample(rng))
    }
}

impl<const D: Dimension> SampleUniform for Quantity<f64, D> {
    type Sampler = UniformQuantity<f64, D>;
}

#[cfg(test)]
mod tests {
    use rand::Rng;

    use crate::units::Length;

    #[test]
    fn test_random_quantity_generation() {
        let mut rng = rand::thread_rng();
        for _ in 0..100 {
            let x = rng.gen_range(Length::meters(0.0)..Length::meters(1.0));
            assert!(Length::meters(0.0) <= x);
            assert!(x < Length::meters(1.0));
        }
    }
}
