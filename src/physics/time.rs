use bevy::prelude::Deref;
use bevy::prelude::DerefMut;

use crate::io::output::Attribute;

#[derive(Clone, Deref, DerefMut)]
pub struct Time(pub crate::units::Time);

impl Attribute for Time {
    type Output = crate::units::Time;

    fn to_value(&self) -> Self::Output {
        self.0
    }

    fn name() -> &'static str {
        "time"
    }
}
