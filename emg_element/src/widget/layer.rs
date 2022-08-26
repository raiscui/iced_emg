use std::{clone::Clone, cmp::PartialEq};

use emg_common::IdStr;
use emg_native::Rect;
use tracing::{info, instrument, trace};

use crate::GElement;

// ────────────────────────────────────────────────────────────────────────────────

type LayerChildren<Message, RenderContext> = Vec<GElement<Message, RenderContext>>;

/// A container that distributes its contents vertically.
///
/// A [`Layer`] will try to fill the horizontal space of its container.
//TODO remove all missing_debug_implementations
#[allow(missing_debug_implementations)]
#[derive(Eq)]
pub struct Layer<Message, RenderContext> {
    id: IdStr,
    //TODO vec?
    children: LayerChildren<Message, RenderContext>,
}

impl<Message, RenderContext> std::fmt::Debug for Layer<Message, RenderContext> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Layer")
            .field("id", &self.id)
            .field("children", &self.children)
            .finish()
    }
}

impl<Message, RenderContext> Clone for Layer<Message, RenderContext> {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            children: self.children.clone(),
        }
    }
}
impl<Message, RenderContext> PartialEq for Layer<Message, RenderContext>
// where
//     Message: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.children == other.children
    }
}

impl<Message, RenderContext> Default for Layer<Message, RenderContext> {
    fn default() -> Self {
        Self::new(IdStr::new_inline(""))
    }
}

impl<Message, RenderContext> Layer<Message, RenderContext> {
    /// Creates an empty [`Layer`].
    #[must_use]
    pub fn new(id: IdStr) -> Self {
        Self::with_children(id, LayerChildren::<Message, RenderContext>::new())
    }

    /// Creates a [`Layer`] with the given elements.
    #[must_use]
    pub fn with_children(id: IdStr, children: LayerChildren<Message, RenderContext>) -> Self {
        Self { id, children }
    }

    #[must_use]
    pub fn set_children(mut self, children: LayerChildren<Message, RenderContext>) -> Self {
        self.children = children;
        self
    }

    // /// Sets the width of the [`Layer`].
    // #[must_use]
    // pub const fn width(mut self, width: Length) -> Self {
    //     self.width = width;
    //     self
    // }

    // /// Sets the height of the [`Layer`].
    // #[must_use]
    // pub const fn height(mut self, height: Length) -> Self {
    //     self.height = height;
    //     self
    // }

    pub fn push(&mut self, child: GElement<Message, RenderContext>) {
        self.children.push(child);
    }
}

#[cfg(all(feature = "gpu"))]
impl<Message, RenderContext> crate::Widget<Message, RenderContext> for Layer<Message, RenderContext>
where
    RenderContext: emg_native::RenderContext + 'static,
    Message: 'static,
    // Message: PartialEq + 'static + std::clone::Clone,
{
    #[instrument(skip(ctx), name = "Layer paint")]
    fn paint(&self, ctx: &mut crate::PaintCtx<RenderContext>) {
        let rect = ctx.size().to_rect();
        info!("[layer] print... {}", &rect);

        //TODO remove this (debug things)
        if self.id == "debug_layer" {
            ctx.fill(rect, &emg_native::Color::rgb8(70, 0, 0));
        } else {
            ctx.fill(rect, &emg_native::Color::rgb8(0, 0, 200));
        }
        // ctx.fill(rect, &emg_native::Color::rgb8(0, 0, 200));

        // ctx.save().unwrap();
        for child in &self.children {
            child.paint(ctx);
        }
    }
}
