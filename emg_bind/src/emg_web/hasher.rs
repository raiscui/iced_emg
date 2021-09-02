/*
 * @Author: Rais
 * @Date: 2021-09-01 09:10:24
 * @LastEditTime: 2021-09-02 12:52:02
 * @LastEditors: Rais
 * @Description:
 */
use std::fmt;

use rustc_hash::FxHasher as CustomHasher;
// use std::collections::hash_map::DefaultHasher;
/// The hasher used to compare subscriptions.

pub struct Hasher(CustomHasher);

impl Default for Hasher {
    fn default() -> Self {
        Self(CustomHasher::default())
    }
}

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
