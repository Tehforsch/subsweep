pub trait Named {
    fn name() -> &'static str;
}

pub use derive_named::Named;

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
}
