use bevy_utils::StableHashMap;
use bevy_utils::StableHashSet;

pub type HashMap<K, V> = StableHashMap<K, V>;
pub type HashSet<K> = StableHashSet<K>;
pub type BiMap<K, V> = bimap::BiMap<K, V>;
