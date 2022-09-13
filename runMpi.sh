killall tenet
numRanks=$1
shift
cargo build && mpirun --mca opal_warn_on_missing_libcuda 0 -n $numRanks ~/.cargo-target/debug/tenet $@
killall tenet
