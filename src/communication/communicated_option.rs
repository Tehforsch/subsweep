use std::mem::MaybeUninit;

use mpi::datatype::UserDatatype;
use mpi::internal::memoffset::offset_of;
use mpi::traits::Equivalence;
use mpi::Address;

#[derive(Debug)]
pub struct CommunicatedOption<T> {
    valid: bool,
    data: MaybeUninit<T>,
}

impl<T> Clone for CommunicatedOption<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        let data = unsafe {
            match self.valid {
                true => MaybeUninit::new(self.data.assume_init_ref().clone()),
                false => MaybeUninit::uninit(),
            }
        };
        Self {
            valid: self.valid,
            data,
        }
    }
}

impl<T> From<Option<T>> for CommunicatedOption<T> {
    fn from(data: Option<T>) -> Self {
        match data {
            Some(data) => Self {
                valid: true,
                data: MaybeUninit::<T>::new(data),
            },
            None => Self {
                valid: false,
                data: MaybeUninit::<T>::uninit(),
            },
        }
    }
}

impl<T> From<CommunicatedOption<T>> for Option<T> {
    fn from(other: CommunicatedOption<T>) -> Option<T> {
        if other.valid {
            unsafe { Some(other.data.assume_init()) }
        } else {
            None
        }
    }
}

unsafe impl<T> Equivalence for CommunicatedOption<T>
where
    T: Equivalence,
{
    type Out = UserDatatype;

    fn equivalent_datatype() -> Self::Out {
        UserDatatype::structured(
            &[1, 1],
            &[
                offset_of!(CommunicatedOption<T>, valid) as Address,
                offset_of!(CommunicatedOption<T>, data) as Address,
            ],
            &[
                UserDatatype::contiguous(1, &bool::equivalent_datatype()),
                UserDatatype::contiguous(1, &T::equivalent_datatype()),
            ],
        )
    }
}

#[cfg(test)]
mod tests {
    use mpi::traits::Communicator;
    use mpi::traits::Equivalence;

    use crate::communication::communicated_option::CommunicatedOption;
    use crate::communication::MPI_UNIVERSE;

    #[derive(Clone, Equivalence, PartialEq, Eq, Debug)]
    struct ComplexStruct {
        i: [i32; 3],
        b: bool,
    }

    #[test]
    fn communicated_option() {
        pack_unpack_option(Some(15i32));
        pack_unpack_option::<i32>(None);
        pack_unpack_option(Some(ComplexStruct {
            i: [4, 5, 6],
            b: true,
        }));
        pack_unpack_option::<ComplexStruct>(None);
    }

    fn pack_unpack_option<T>(option: Option<T>)
    where
        T: Equivalence + Clone + PartialEq + std::fmt::Debug,
    {
        let world = MPI_UNIVERSE.with(|universe| universe.world());

        let option_converted: CommunicatedOption<T> = option.clone().into();
        let mut q2: CommunicatedOption<T> = None.into();
        let bytes = world.pack(&option_converted);
        unsafe {
            world.unpack_into(&bytes, &mut q2, 0);
        }
        assert_eq!(option, q2.into());
    }
}
