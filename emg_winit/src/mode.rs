/*
 * @Author: Rais
 * @Date: 2022-08-12 13:36:56
 * @LastEditTime: 2022-08-12 13:36:56
 * @LastEditors: Rais
 * @Description:
 */
/// The mode of a window-based application.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// The application appears in its own window.
    Windowed,

    /// The application takes the whole screen of its current monitor.
    Fullscreen,

    /// The application is hidden
    Hidden,
}
