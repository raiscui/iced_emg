/*
 * @Author: Rais
 * @Date: 2021-09-01 09:10:24
 * @LastEditTime: 2021-09-28 17:29:42
 * @LastEditors: Rais
 * @Description:
 */

 //TODO remove this mod
use emg_hasher::CustomHasher;
use std::fmt;
// use std::collections::hash_map::DefaultHasher;
/// The hasher used to compare subscriptions.

#[derive(Default)]
pub struct Hasher(CustomHasher);

// impl Default for Hasher {
//     fn default() -> Self {
//         Self(CustomHasher::default())
//     }
// }

impl core::hash::Hasher for Hasher {
    fn write(&mut self, bytes: &[u8]) {
        self.0.write(bytes);
    }

    fn finish(&self) -> u64 {
        self.0.finish()
    }
}

impl fmt::Debug for Hasher {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Hasher(CustomHasher)")
            .finish_non_exhaustive()
    }
}
