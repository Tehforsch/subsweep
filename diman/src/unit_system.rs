#[macro_export]
macro_rules! unit_system {
    ($dimension: ident, $quantity: ident, $($const: ident, $quantity_name:ident, $($dimension_name: ident: $dimension_value: literal),*, {$($unit:ident, $factor:literal, $($unit_symbol:literal)?),*}),+) => {
        use paste::paste;
        pub const UNIT_NAMES: &[($dimension, &str, f64)] = &[
        $(
            $(
                $(
                    ($const, $unit_symbol, $factor),
                )*
            )*
        )*
        ];
        $(
            pub const $const: $dimension = $dimension {
                $(
                    $dimension_name: $dimension_value,
                )*
                .. NONE };
            pub type $quantity_name = $quantity<f64, $const>;
            paste!{
                pub type [<Vec $quantity_name>] = $quantity<glam::DVec2, $const>;
                pub type [<Vec2 $quantity_name>] = $quantity<glam::DVec2, $const>;
                pub type [<Vec3 $quantity_name>] = $quantity<glam::DVec3, $const>;
            }
            impl $quantity_name {
                $(
                    pub const fn $unit(v: f64) -> $quantity_name {
                        $quantity::<f64, $const>(v * $factor)
                    }
                )*
            }
            paste! {
            impl [<Vec $quantity_name>] {
                $(
                    pub fn $unit(x: f64, y: f64) -> $quantity::<glam::DVec2, $const> {
                        $quantity::<glam::DVec2, $const>(glam::DVec2::new(x, y) * $factor)
                    }
                )*
            }
            }
        )*
    }
}
