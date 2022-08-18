killall tenet
cargo build && mpirun -np 4 kitty gdb -ex start --args ~/.cargo-target/debug/tenet $@
killall tenet
