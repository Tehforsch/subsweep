use bevy::utils::StableHashMap;
use bevy::utils::StableHashSet;

pub type HashMap<K, V> = StableHashMap<K, V>;
pub type HashSet<K> = StableHashSet<K>;
pub type BiMap<K, V> = bimap::BiMap<K, V>;
