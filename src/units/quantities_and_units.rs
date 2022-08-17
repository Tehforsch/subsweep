use super::dimension::Dimension;
use super::quantity::Quantity;

pub(super) const NONE: Dimension = Dimension {
    length: 0,
    time: 0,
    mass: 0,
};

impl<const D: Dimension> Quantity<f32, D> {
    pub fn abs(&self) -> Self {
        Self(self.0.abs())
    }
}

macro_rules! unit_functions {
    ($storage_type:ty, $($const: ident, $quantity:ident, $($dimension_name: ident: $dimension: literal),*, {$($unit:ident, $factor:literal),+}),+) => {
        use super::Dimension;
        use super::Quantity;
        use super::NONE;
        $(

            const $const: Dimension = Dimension {
                $(
                    $dimension_name: $dimension,
                )*
                .. NONE };
            pub type $quantity = Quantity<$storage_type, $const>;
            $(
            pub fn $unit(v: $storage_type) -> $quantity {
                Quantity::<$storage_type, $const>(v * $factor)
            }
            )*
        )*
    }
}

#[rustfmt::skip]
macro_rules! implement_storage_type {
    ($type:ty) => {
        unit_functions!($type,
                    DIMENSIONLESS, Dimensionless, length: 0,
                    {
                        dimensionless, 1.0
                    },
                    LENGTH, Length, length: 1,
                    {
                        meter, 1.0,
                        kilometer, 1000.0
                    },
                    TIME, Time, time: 1,
                    {
                        second, 1.0
                    },
                    VELOCITY, Velocity, length: 1, time: -1,
                    {
                        meters_per_second, 1.0
                    },
                    MASS, Mass, mass: 1,
                    {
                        kilograms, 1.0
                    },
                    FORCE, Force, mass: 1, length: 1, time: -2,
                    {
                        newton, 1.0
                    }
                    );
    }
}

pub mod vec2 {
    use glam::Vec2;
    implement_storage_type!(Vec2);
}

pub mod f32 {
    implement_storage_type!(f32);
}
