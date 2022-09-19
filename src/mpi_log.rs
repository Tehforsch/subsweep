use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use mpi::traits::CommunicatorCollectives;

use crate::communication::MPI_UNIVERSE;

pub static RANK: AtomicUsize = AtomicUsize::new(0);
pub static SIZE: AtomicUsize = AtomicUsize::new(0);

pub fn initialize(rank: i32, size: usize) {
    RANK.swap(rank as usize, Ordering::SeqCst);
    SIZE.swap(size, Ordering::SeqCst);
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
                if $crate::mpi_log::RANK.load(std::sync::atomic::Ordering::SeqCst) == 0 {
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

#[macro_export]
macro_rules! mpidbg {
    () => {
        eprintln!("[{}:{}] rank={}",  $crate::file!(), $crate::line!(),
                  $crate::mpi_log::RANK.load(std::sync::atomic::Ordering::SeqCst)
        )
    };
    ($val:expr $(,)?) => {
        // Use of `match` here is intentional because it affects the lifetimes
        // of temporaries - https://stackoverflow.com/a/48732525/1063961
        match $val {
            tmp => {
                eprintln!("[{}:{}] rank={} {} = {:#?}",
                          file!(), line!(),
                          $crate::mpi_log::RANK.load(std::sync::atomic::Ordering::SeqCst),
                          stringify!($val), &tmp);
                tmp
            }
        }
    };
    ($($val:expr),+ $(,)?) => {
        ($(mpidbg!($val)),+,)
    };
}

pub fn start_barrier() {
    let rank = crate::mpi_log::RANK.load(std::sync::atomic::Ordering::SeqCst);
    let world = MPI_UNIVERSE.world();
    for _ in 0..rank {
        world.barrier();
    }
}

pub fn end_barrier() {
    let rank = crate::mpi_log::RANK.load(std::sync::atomic::Ordering::SeqCst);
    let size = crate::mpi_log::SIZE.load(std::sync::atomic::Ordering::SeqCst);
    let world = MPI_UNIVERSE.world();
    for _ in rank..size {
        world.barrier();
    }
}

#[macro_export]
macro_rules! barrierdbg {
    () => {
        $crate::mpi_log::start_barrier();
        eprintln!("[{}:{}] rank={}",  $crate::file!(), $crate::line!(),
                  $crate::mpi_log::RANK.load(std::sync::atomic::Ordering::SeqCst)
        )
        $crate::mpi_log::end_barrier();
    };
    ($val:expr $(,)?) => {
        // Use of `match` here is intentional because it affects the lifetimes
        // of temporaries - https://stackoverflow.com/a/48732525/1063961
        $crate::mpi_log::start_barrier();
        match $val {
            tmp => {
                eprintln!("[{}:{}] rank={} {} = {:#?}",
                          file!(), line!(),
                          $crate::mpi_log::RANK.load(std::sync::atomic::Ordering::SeqCst),
                          stringify!($val), &tmp);
                $crate::mpi_log::end_barrier();
                tmp
            }
        }
    };
    ($($val:expr),+ $(,)?) => {
        $crate::mpi_log::start_barrier();
        ($(mpidbg!($val)),+,)
        $crate::mpi_log::end_barrier();
    };
}
