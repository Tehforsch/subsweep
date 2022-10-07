use bevy::prelude::Deref;
use bevy::prelude::DerefMut;

use crate::io::output::ToAttribute;
use crate::named::Named;

#[derive(Clone, Deref, DerefMut, Named)]
#[name = "time"]
pub struct Time(pub crate::units::Time);

impl ToAttribute for Time {
    type Output = crate::units::Time;

    fn to_value(&self) -> Self::Output {
        self.0
    }
}
