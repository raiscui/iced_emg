/*
 * @Author: Rais
 * @Date: 2022-08-22 22:42:54
 * @LastEditTime: 2023-04-14 16:40:11
 * @LastEditors: Rais
 * @Description:
 */
#![allow(clippy::module_name_repetitions)]

pub use emg_element::gtree_macro_prelude;
pub use emg_element::prelude::*;

#[cfg(feature = "video-player")]
pub use emg_element::component::Video;
