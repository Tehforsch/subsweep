killall tenet
cargo build && mpirun --mca opal_warn_on_missing_libcuda 0 -n 4 ~/.cargo-target/debug/tenet $@
killall tenet
