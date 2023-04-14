/*
 * @Author: Rais
 * @Date: 2023-04-13 15:53:35
 * @LastEditTime: 2023-04-14 16:36:58
 * @LastEditors: Rais
 * @Description:
 */
/*
 * @Author: Rais
 * @Date: 2023-04-13 13:31:37
 * @LastEditTime: 2023-04-13 13:31:38
 * @LastEditors: Rais
 * @Description:
 */
mod ui_features;
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(feature = "video-player")]
pub use ui_features::video_player::VideoPlayer;
// ─────────────────────────────────────────────────────────────────────────────

pub use ui_features::video_player_trait::VideoPlayerT;
