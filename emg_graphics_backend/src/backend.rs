/*
 * @Author: Rais
 * @Date: 2022-08-15 21:19:37
 * @LastEditTime: 2023-01-04 10:19:41
 * @LastEditors: Rais
 * @Description:
 */
//! Write a graphics backend.

/// The graphics backend of a [`Renderer`].
///
/// [`Renderer`]: crate::Renderer
pub trait Backend {
    type SceneCtx;

    fn new_scene_ctx() -> Self::SceneCtx;

    fn on_loop_destroyed(&mut self);
}

// /// A graphics backend that supports text rendering.
// pub trait Text {
//     /// The icon font of the backend.
//     const ICON_FONT: Font;

//     /// The `char` representing a ✔ icon in the [`ICON_FONT`].
//     ///
//     /// [`ICON_FONT`]: Self::ICON_FONT
//     const CHECKMARK_ICON: char;

//     /// The `char` representing a ▼ icon in the built-in [`ICON_FONT`].
//     ///
//     /// [`ICON_FONT`]: Self::ICON_FONT
//     const ARROW_DOWN_ICON: char;

//     /// Returns the default size of text.
//     fn default_size(&self) -> u16;

//     /// Measures the text contents with the given size and font,
//     /// returning the size of a laid out paragraph that fits in the provided
//     /// bounds.
//     fn measure(&self, contents: &str, size: f32, font: Font, bounds: Size) -> (f32, f32);

//     /// Tests whether the provided point is within the boundaries of [`Text`]
//     /// laid out with the given parameters, returning information about
//     /// the nearest character.
//     ///
//     /// If nearest_only is true, the hit test does not consider whether the
//     /// the point is interior to any glyph bounds, returning only the character
//     /// with the nearest centeroid.
//     fn hit_test(
//         &self,
//         contents: &str,
//         size: f32,
//         font: Font,
//         bounds: Size,
//         point: Point,
//         nearest_only: bool,
//     ) -> Option<text::Hit>;
// }

// /// A graphics backend that supports image rendering.
// pub trait Image {
//     /// Returns the dimensions of the provided image.
//     fn dimensions(&self, handle: &image::Handle) -> (u32, u32);
// }

// /// A graphics backend that supports SVG rendering.
// pub trait Svg {
//     /// Returns the viewport dimensions of the provided SVG.
//     fn viewport_dimensions(&self, handle: &svg::Handle) -> (u32, u32);
// }
