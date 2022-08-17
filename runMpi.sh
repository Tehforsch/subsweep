killall tenet
cargo build && mpirun -n 4 ~/.cargo-target/debug/tenet $@
killall tenet
