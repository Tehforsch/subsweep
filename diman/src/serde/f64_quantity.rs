use std::fmt;

use serde::de::Visitor;
use serde::de::{self};
use serde::Deserialize;
use serde::Deserializer;

use super::super::dimension::Dimension;
use super::super::quantity::Quantity;
use super::get_quantity_if_dimensions_match;
use super::read_unit_str;
use super::QuantityVisitor;
use crate::dimension::NONE;

impl<'de, const D: Dimension> Deserialize<'de> for Quantity<f64, D> {
    fn deserialize<DE>(deserializer: DE) -> Result<Quantity<f64, D>, DE::Error>
    where
        DE: Deserializer<'de>,
    {
        deserializer.deserialize_string(QuantityVisitor::<f64, D>::default())
    }
}

impl<'de, const D: Dimension> Visitor<'de> for QuantityVisitor<f64, D> {
    type Value = Quantity<f64, D>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a numerical value followed by a series of powers of units")
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if D == NONE {
            Ok(Quantity::<f64, D>(value as f64))
        } else {
            Err(E::custom(format!(
                "dimensionless numerical value given for non-dimensionless quantity: {}",
                value
            )))
        }
    }
    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if D == NONE {
            Ok(Quantity::<f64, D>(value as f64))
        } else {
            Err(E::custom(format!(
                "dimensionless numerical value given for non-dimensionless quantity: {}",
                value
            )))
        }
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if D == NONE {
            Ok(Quantity::<f64, D>(value as f64))
        } else {
            Err(E::custom(format!(
                "dimensionless numerical value given for non-dimensionless quantity: {}",
                value
            )))
        }
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
        let numerical_value = numerical_value_str.parse::<f64>().map_err(|_| {
            E::custom(format!(
                "unable to parse numerical value {}",
                &numerical_value_str
            ))
        })?;
        let (total_dimension, total_factor) = read_unit_str(split)?;
        get_quantity_if_dimensions_match::<f64, D, E>(
            value,
            numerical_value * total_factor,
            total_dimension,
        )
    }
}
