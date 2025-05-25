#[cfg(feature = "alloc")]
pub mod hash_map;
pub use hash_map::HashMap;//这样就可以直接调用了，不必再加axstd::collections::hash_map了