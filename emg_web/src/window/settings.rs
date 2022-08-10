/*
 * @Author: Rais
 * @Date: 2022-08-09 20:43:29
 * @LastEditTime: 2022-08-09 20:44:56
 * @LastEditors: Rais
 * @Description:
 */

/// The window settings of an application.
#[derive(Debug, Clone)]
pub struct Settings {}

impl Default for Settings {
    fn default() -> Settings {
        Settings {}
    }
}

// impl From<Settings> for iced_winit::settings::Window {
//     fn from(settings: Settings) -> Self {
//         Self {
//             size: settings.size,
//             position: iced_winit::Position::from(settings.position),
//             min_size: settings.min_size,
//             max_size: settings.max_size,
//             resizable: settings.resizable,
//             decorations: settings.decorations,
//             transparent: settings.transparent,
//             always_on_top: settings.always_on_top,
//             icon: settings.icon.map(Icon::into),
//             platform_specific: Default::default(),
//         }
//     }
// }
