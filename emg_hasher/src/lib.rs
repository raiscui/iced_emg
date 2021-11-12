// pub use xxhash_rust::xxh3::Xxh3 as CustomHasher;
// pub use twox_hash::xxh3::Hash64 as CustomHasher;
// pub use twox_hash::XxHash32 as CustomHasher;
// pub use std::collections::hash_map::DefaultHasher as CustomHasher;

pub use rustc_hash::FxHasher as CustomHasher;

// pub use fnv::FnvHasher as CustomHasher;

// pub use ahash::AHasher as CustomHasher;
// #[cfg(test)]
// mod tests {
//     #[test]
//     fn it_works() {
//         let result = 2 + 2;
//         assert_eq!(result, 4);
//     }
// }
