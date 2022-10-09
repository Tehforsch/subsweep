#[macro_export]
macro_rules! impl_vector_methods {
    ($quantity: ident, $dimension: ident, $dimensionless_const: ident, $vector_type: ident, $float_type: ident) => {
        impl<const D: $dimension> $quantity<$vector_type, D> {
            pub fn new(x: $quantity<$float_type, D>, y: $quantity<$float_type, D>) -> Self {
                Self($vector_type::new(x.unwrap_value(), y.unwrap_value()))
            }

            pub fn new_x(x: $quantity<$float_type, D>) -> Self {
                Self($vector_type::new(x.unwrap_value(), 0.0))
            }

            pub fn new_y(y: $quantity<$float_type, D>) -> Self {
                Self($vector_type::new(0.0, y.unwrap_value()))
            }

            pub fn from_vector_and_scale(
                vec: $vector_type,
                scale: $quantity<$float_type, D>,
            ) -> Self {
                Self::new(vec.x * scale, vec.y * scale)
            }

            pub fn abs(&self) -> Self {
                Self(self.0.abs())
            }

            pub fn zero() -> Self {
                Self($vector_type::new(0.0, 0.0))
            }

            pub fn x(&self) -> $quantity<$float_type, D> {
                $quantity(self.0.x)
            }

            pub fn y(&self) -> $quantity<$float_type, D> {
                $quantity(self.0.y)
            }

            pub fn set_x(&mut self, new_x: $quantity<$float_type, D>) {
                self.0.x = new_x.unwrap_value();
            }

            pub fn set_y(&mut self, new_y: $quantity<$float_type, D>) {
                self.0.y = new_y.unwrap_value();
            }

            pub fn length(&self) -> $quantity<$float_type, D> {
                $quantity::<$float_type, D>(self.0.length())
            }

            pub fn distance(&self, other: &Self) -> $quantity<$float_type, D> {
                $quantity::<$float_type, D>(self.0.distance(other.0))
            }

            pub fn distance_squared(
                &self,
                other: &Self,
            ) -> $quantity<$float_type, { D.dimension_powi(2) }>
            where
                $quantity<$float_type, { D.dimension_powi(2) }>:,
            {
                $quantity::<$float_type, { D.dimension_powi(2) }>(self.0.distance_squared(other.0))
            }

            pub fn normalize(&self) -> $quantity<$vector_type, NONE> {
                $quantity::<$vector_type, NONE>(self.0.normalize())
            }
        }
    };
}
