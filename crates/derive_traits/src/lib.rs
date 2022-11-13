//! This library exists mainly because I found it impossible/annoyingly hard
//! to create procedural macros that derive traits from the main crate.
//! This works fine for the main crate itself, but fails when
//! building examples. The problem is path/name resolution of the derived trait.
//! I found some info on this here: https://github.com/rust-lang/rust/issues/54363
use serde::{Serialize, Deserialize};

pub trait Named {
    fn name() -> &'static str;
}

#[cfg(test)]
mod tests {
    use crate::named::Named;

    #[test]
    fn name_derive() {
        #[derive(Named)]
        #[name = "A"]
        struct A {
            _x: i32,
        }

        assert_eq!(A::name(), "A");
    }

    #[test]
    fn name_derive_generic() {
        #[derive(Named)]
        #[name = "B"]
        struct X<T> {
            _t: T,
        }

        assert_eq!(X::<i32>::name(), "B");
    }

    #[test]
    fn name_derive_more_attributes() {
        #[derive(Named)]
        #[repr(transparent)]
        #[name = "A"]
        struct A {
            _x: i32,
        }

        assert_eq!(A::name(), "A");
    }

    #[test]
    fn name_derive_implicitly() {
        #[derive(Named)]
        struct Foo {
            _x: i32,
        }

        assert_eq!(Foo::name(), "Foo");
    }
}

pub trait RaxiomParameters: Serialize + for<'de> Deserialize<'de> + bevy::prelude::Resource {
    fn section_name() -> Option<&'static str>;

    fn unwrap_section_name() -> &'static str {
        Self::section_name()
            .unwrap_or_else(|| panic!("Called unwrap_section_name on unnamed parameter struct."))
    }
}
