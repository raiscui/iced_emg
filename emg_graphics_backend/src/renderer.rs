/*
 * @Author: Rais
 * @Date: 2022-08-15 21:17:17
 * @LastEditTime: 2022-09-02 19:15:51
 * @LastEditors: Rais
 * @Description:
 */
//! Create a renderer from a [`Backend`].

use std::marker::PhantomData;

use emg_native::{PaintCtx, WidgetState};

use crate::Backend;

/// A backend-agnostic renderer that supports all the built-in widgets.
#[derive(Debug)]
pub struct Renderer<B: Backend> {
    backend: B,
    // primitives: Vec<Primitive>,
    // theme: PhantomData<Theme>,
}

impl<B: Backend> Renderer<B> {
    /// Creates a new [`Renderer`] from the given [`Backend`].
    pub fn new(backend: B) -> Self {
        Self {
            backend,
            // primitives: Vec::new(),
            // theme: PhantomData,
        }
    }

    /// Returns the [`Backend`] of the [`Renderer`].
    pub fn backend(&self) -> &B {
        &self.backend
    }
    pub fn backend_mut(&mut self) -> &mut B {
        &mut self.backend
    }

    // /// Enqueues the given [`Primitive`] in the [`Renderer`] for drawing.
    // pub fn draw_primitive(&mut self, primitive: Primitive) {
    // self.primitives.push(primitive);
    // }

    // /// Runs the given closure with the [`Backend`] and the recorded primitives
    // /// of the [`Renderer`].
    // pub fn with_primitives(&mut self, f: impl FnOnce(&mut B, &[Primitive])) {
    // f(&mut self.backend, &self.primitives);
    // }
}

impl<B: Backend> emg_native::renderer::Renderer for Renderer<B> {
    type ImplRenderContext = <B as Backend>::ImplRenderContext;

    fn new_paint_ctx(&self) -> PaintCtx<Self::ImplRenderContext> {
        let new_render_ctx = self.backend.new_render_ctx();
        PaintCtx::new(WidgetState::default(), new_render_ctx)
    }
    fn on_loop_destroyed(&mut self) {
        self.backend.on_loop_destroyed();
    }
}

// impl<B, T> text::Renderer for Renderer<B, T>
// where
//     B: Backend + backend::Text,
// {
//     type Font = Font;

//     const ICON_FONT: Font = B::ICON_FONT;
//     const CHECKMARK_ICON: char = B::CHECKMARK_ICON;
//     const ARROW_DOWN_ICON: char = B::ARROW_DOWN_ICON;

//     fn default_size(&self) -> u16 {
//         self.backend().default_size()
//     }

//     fn measure(&self, content: &str, size: u16, font: Font, bounds: Size) -> (f32, f32) {
//         self.backend()
//             .measure(content, f32::from(size), font, bounds)
//     }

//     fn hit_test(
//         &self,
//         content: &str,
//         size: f32,
//         font: Font,
//         bounds: Size,
//         point: Point,
//         nearest_only: bool,
//     ) -> Option<text::Hit> {
//         self.backend()
//             .hit_test(content, size, font, bounds, point, nearest_only)
//     }

//     fn fill_text(&mut self, text: Text<'_, Self::Font>) {
//         self.primitives.push(Primitive::Text {
//             content: text.content.to_string(),
//             bounds: text.bounds,
//             size: text.size,
//             color: text.color,
//             font: text.font,
//             horizontal_alignment: text.horizontal_alignment,
//             vertical_alignment: text.vertical_alignment,
//         });
//     }
// }

// impl<B, T> image::Renderer for Renderer<B, T>
// where
//     B: Backend + backend::Image,
// {
//     type Handle = image::Handle;

//     fn dimensions(&self, handle: &image::Handle) -> (u32, u32) {
//         self.backend().dimensions(handle)
//     }

//     fn draw(&mut self, handle: image::Handle, bounds: Rectangle) {
//         self.draw_primitive(Primitive::Image { handle, bounds })
//     }
// }

// impl<B, T> svg::Renderer for Renderer<B, T>
// where
//     B: Backend + backend::Svg,
// {
//     fn dimensions(&self, handle: &svg::Handle) -> (u32, u32) {
//         self.backend().viewport_dimensions(handle)
//     }

//     fn draw(&mut self, handle: svg::Handle, bounds: Rectangle) {
//         self.draw_primitive(Primitive::Svg { handle, bounds })
//     }
// }
