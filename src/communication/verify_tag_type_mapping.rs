use std::any::type_name;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;

use mpi::traits::Equivalence;
use mpi::Tag;

use super::MpiWorld;
use super::SizedCommunicator;

pub const VERIFICATION_TAG: Tag = 0;

#[derive(PartialEq, Eq, Debug, Equivalence)]
struct TagTypeMapping {
    tag: Tag,
    type_hash: u64,
}

/// Checks whether all ranks agree that the tag is used to
/// communicate type with name type_name
pub(super) fn verify_tag_type_mapping<T>(tag: Tag) {
    let type_name = type_name::<T>();
    let mapping = TagTypeMapping {
        tag,
        type_hash: calculate_hash(&type_name.to_owned()),
    };
    let mut world = MpiWorld::<TagTypeMapping>::new(VERIFICATION_TAG);
    if !world.all_ranks_have_same_value(&mapping) {
        panic!(
            "Different tag <-> type mapping between ranks! On rank {}, tag {} communicates {}.",
            world.rank(),
            tag,
            type_name
        );
    }
}

// Use hashes to check that we are communicating the same type for a
// tag, since they are much easier to communicate using MPI than
// arbitrary length strings
fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}
