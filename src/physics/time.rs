use crate::output::Attribute;

#[derive(Clone)]
pub struct Time(pub crate::units::Time);

impl Attribute for Time {
    type Output = crate::units::Time;

    fn to_value(&self) -> Self::Output {
        self.0
    }
}
