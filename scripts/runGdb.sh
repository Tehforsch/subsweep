killall raxiom
numRanks=$1
shift
cargo build && mpirun -np $numRanks kitty gdb -ex start --args ~/.cargo-target/debug/raxiom $@
killall raxiom
