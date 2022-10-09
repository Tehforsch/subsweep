use std::fmt;

use glam::DVec2;
use serde::de::Visitor;
use serde::de::{self};
use serde::Deserialize;
use serde::Deserializer;

use super::super::dimension::Dimension;
use super::super::quantity::Quantity;
use super::get_quantity_if_dimensions_match;
use super::read_unit_str;
use super::QuantityVisitor;

impl<'de, const D: Dimension> Deserialize<'de> for Quantity<DVec2, D> {
    fn deserialize<DE>(deserializer: DE) -> Result<Quantity<DVec2, D>, DE::Error>
    where
        DE: Deserializer<'de>,
    {
        deserializer.deserialize_string(QuantityVisitor::<DVec2, D>::default())
    }
}

impl<'de, const D: Dimension> Visitor<'de> for QuantityVisitor<DVec2, D> {
    type Value = Quantity<DVec2, D>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("two numerical values surrounded by () followed by a series of powers of units, e.g. (1.0 2.0) m s^-2")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let value = value.trim();
        let bracket_end = value
            .find(')')
            .ok_or_else(|| E::custom("No closing bracket in vector string"))?;
        let (vector_part, unit_part) = value.split_at(bracket_end + 1);
        let bracket_begin = vector_part
            .find('(')
            .ok_or_else(|| E::custom("No opening bracket in vector string"))?;
        let vector_part = vector_part[bracket_begin + 1..vector_part.len() - 1].to_string();
        let vector_components = &vector_part.split_whitespace().collect::<Vec<_>>();
        if vector_components.len() != 2 {
            return Err(E::custom("found {} substrings in brackets, expected 2"))?;
        }
        let x_str = vector_components[0];
        let y_str = vector_components[1];
        let x = x_str
            .parse::<f64>()
            .map_err(|e| E::custom(format!("While parsing x component: {}, '{}'", e, x_str)))?;
        let y = y_str
            .parse::<f64>()
            .map_err(|e| E::custom(format!("While parsing x component: {}, '{}'", e, y_str)))?;
        let vector = DVec2::new(x, y);
        let (total_dimension, total_factor) = read_unit_str(unit_part.split_whitespace())?;
        get_quantity_if_dimensions_match::<DVec2, D, E>(
            value,
            total_factor * vector,
            total_dimension,
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::si::Length;
    use crate::si::VecLength;
    use crate::tests::assert_is_close;

    #[test]
    fn deserialize_vector() {
        let q: VecLength = serde_yaml::from_str("(5.0 3.0) km").unwrap();
        assert_is_close(q.x(), Length::kilometers(5.0));
        assert_is_close(q.y(), Length::kilometers(3.0));
    }

    #[test]
    #[should_panic]
    fn deserialize_vector_fails_with_fewer_than_2_components() {
        let _: VecLength = serde_yaml::from_str("(5.0) km").unwrap();
    }

    #[test]
    #[should_panic]
    fn deserialize_vector_fails_with_more_than_2_components() {
        let _: VecLength = serde_yaml::from_str("(5.0 3.0 7.0) km").unwrap();
    }
}
