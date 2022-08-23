/*
 * @Author: Rais
 * @Date: 2022-08-15 21:19:37
 * @LastEditTime: 2022-08-21 00:45:03
 * @LastEditors: Rais
 * @Description:
 */
//! Write a graphics backend.

/// The graphics backend of a [`Renderer`].
///
/// [`Renderer`]: crate::Renderer
pub trait Backend {
    type ImplRenderContext: emg_native::RenderContext;
    // /// Trims the measurements cache.
    // ///
    // /// This method is currently necessary to properly trim the text cache in
    // /// `iced_wgpu` and `iced_glow` because of limitations in the text rendering
    // /// pipeline. It will be removed in the future.
    // fn trim_measurements(&mut self) {}
    fn new_render_ctx(&self) -> Self::ImplRenderContext;

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
