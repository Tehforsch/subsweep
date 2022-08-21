use std::fmt;

use serde::de::Visitor;
use serde::de::{self};
use serde::Deserialize;
use serde::Deserializer;

use super::dimension::Dimension;
use super::f32::UNIT_NAMES;
use super::quantity::Quantity;
use super::NONE;

impl<'de, const D: Dimension> Deserialize<'de> for Quantity<f32, D> {
    fn deserialize<DE>(deserializer: DE) -> Result<Quantity<f32, D>, DE::Error>
    where
        DE: Deserializer<'de>,
    {
        deserializer.deserialize_string(QuantityVisitor)
    }
}

struct QuantityVisitor<const D: Dimension>;

impl<'de, const D: Dimension> Visitor<'de> for QuantityVisitor<D> {
    type Value = Quantity<f32, D>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a numerical value followed by a series of powers of units")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let value = value.trim();
        let mut split = value.split_whitespace();
        let numerical_value_str = split
            .next()
            .ok_or_else(|| E::custom("unable to parse empty string"))?;
        let numerical_value = numerical_value_str.parse::<f32>().map_err(|_| {
            E::custom(format!(
                "unable to parse numerical value {}",
                &numerical_value_str
            ))
        })?;
        let mut total_dimension = NONE;
        let mut total_factor = 1.0;
        for unit in split {
            let (dimension, factor) = read_unit_str(unit)?;
            total_dimension = total_dimension.dimension_mul(dimension.clone());
            total_factor *= factor;
        }
        if total_dimension == D {
            Ok(Quantity::<f32, D>(numerical_value * total_factor))
        } else {
            Err(E::custom(format!(
                "mismatch in dimensions: needed: {:?} given: {:?}",
                D, total_dimension
            )))
        }
    }
}

fn read_unit_str<E>(unit_str: &str) -> Result<(Dimension, f32), E>
where
    E: de::Error,
{
    let (unit, exponent) = if unit_str.contains("^") {
        let split: Vec<_> = unit_str.split("^").collect();
        if split.len() != 2 {
            return Err(E::custom(format!("invalid unit string: {}", unit_str)));
        }
        (
            split[0],
            split[1]
                .parse::<i32>()
                .map_err(|_| E::custom(format!("unable to parse unit exponent: {}", split[1])))?,
        )
    } else {
        (unit_str, 1)
    };
    let (dimension, _, factor) = UNIT_NAMES
        .iter()
        .filter(|(_, known_unit_name, _)| &unit == known_unit_name)
        .next()
        .ok_or_else(|| E::custom(format!("unknown unit: {}", &unit)))?;
    Ok((dimension.clone().dimension_powi(exponent), *factor))
}

#[cfg(test)]
mod tests {
    use crate::units::f32::Dimensionless;
    use crate::units::f32::Force;
    use crate::units::f32::Length;
    use crate::units::tests::assert_is_close;

    #[test]
    fn deserialize_basic_units() {
        let q: Length = serde_yaml::from_str(&"1.0 m").unwrap();
        assert_is_close(q, Length::meter(1.0));
        let q: Length = serde_yaml::from_str(&"2.0 m").unwrap();
        assert_is_close(q, Length::meter(2.0));
        let q: Length = serde_yaml::from_str(&"2.0e8 m").unwrap();
        assert_is_close(q, Length::meter(2.0e8));
        let q: Length = serde_yaml::from_str(&"5.0 km").unwrap();
        assert_is_close(q, Length::meter(5000.0));
    }

    #[test]
    fn deserialize_dimensionless_quantities() {
        let q: Dimensionless = serde_yaml::from_str(&"5.0").unwrap();
        assert_is_close(q, Dimensionless::dimensionless(5.0));
    }

    #[test]
    #[should_panic]
    fn do_not_deserialize_dimensionless_quantities_with_unit_str() {
        let q: Dimensionless = serde_yaml::from_str(&"5.0 m").unwrap();
        assert_is_close(q, Dimensionless::dimensionless(5.0));
    }

    #[test]
    #[should_panic]
    fn do_not_allow_unit_mismatch() {
        let _q: Dimensionless = serde_yaml::from_str(&"5.0 km m").unwrap();
    }

    #[test]
    fn deserialize_unit_exponents() {
        let q: Dimensionless = serde_yaml::from_str(&"5.0 km m^-1").unwrap();
        assert_is_close(q, Dimensionless::dimensionless(5000.0));
        let q: Force = serde_yaml::from_str(&"5.0 kg m^1 s^-2").unwrap();
        assert_is_close(q, Force::newton(5.0));
        let q: Force = serde_yaml::from_str(&"5.0e0 kg km^1 s^-2").unwrap();
        assert_is_close(q, Force::newton(5000.0));
    }
}
