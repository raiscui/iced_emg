/*
 * @Author: Rais
 * @Date: 2023-04-13 13:08:16
 * @LastEditTime: 2023-04-13 18:38:30
 * @LastEditors: Rais
 * @Description:
 */
#[cfg(feature = "video-player")]
mod video;
#[cfg(feature = "video-player")]
pub use video::Video;
