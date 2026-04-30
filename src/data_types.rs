
use dashmap::DashMap;
use std::hash::BuildHasherDefault;
use rustc_hash::FxHasher;

pub type FxDashMap<K, V> = DashMap<K, V, BuildHasherDefault<FxHasher>>;


// end of file
