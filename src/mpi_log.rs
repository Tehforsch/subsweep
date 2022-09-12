use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

pub static RANK: AtomicUsize = AtomicUsize::new(0);

pub fn initialize(rank: i32) {
    RANK.swap(rank as usize, Ordering::SeqCst);
}

#[macro_export]
macro_rules! maindbg {
    () => {
        if RANK == 0 {
        eprintln!("[{}:{}]", $crate::file!(), $crate::line!())
        }
    };
    ($val:expr $(,)?) => {
        // Use of `match` here is intentional because it affects the lifetimes
        // of temporaries - https://stackoverflow.com/a/48732525/1063961
        match $val {
            tmp => {
                if crate::mpi_log::RANK.load(std::sync::atomic::Ordering::SeqCst) == 0 {
                    eprintln!("[{}:{}] {} = {:#?}",
                        file!(), line!(), stringify!($val), &tmp);
                }
                tmp
            }
        }
    };
    ($($val:expr),+ $(,)?) => {
        ($(maindbg!($val)),+,)
    };
}
