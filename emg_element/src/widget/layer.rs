use std::{clone::Clone, cmp::PartialEq};

use emg_common::IdStr;
use tracing::{info, info_span, instrument, trace, Span};

use crate::GElement;

// ────────────────────────────────────────────────────────────────────────────────

type LayerChildren<Message, RenderContext> = Vec<GElement<Message, RenderContext>>;

/// A container that distributes its contents vertically.
///
/// A [`Layer`] will try to fill the horizontal space of its container.
//TODO remove all missing_debug_implementations
#[allow(missing_debug_implementations)]
#[derive(Eq)]
pub struct Layer<Message, RenderCtx> {
    id: IdStr,
    //TODO vec?
    children: LayerChildren<Message, RenderCtx>,
}

impl<Message, RenderCtx> std::fmt::Debug for Layer<Message, RenderCtx>
where
    RenderCtx: 'static,
    Message: 'static,
{
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

impl<Message, RenderCtx> Layer<Message, RenderCtx> {
    /// Creates an empty [`Layer`].
    #[must_use]
    pub fn new(id: IdStr) -> Self {
        Self::with_children(id, LayerChildren::<Message, RenderCtx>::new())
    }

    /// Creates a [`Layer`] with the given elements.
    #[must_use]
    pub fn with_children(id: IdStr, children: LayerChildren<Message, RenderCtx>) -> Self {
        Self { id, children }
    }

    #[must_use]
    pub fn set_children(mut self, children: LayerChildren<Message, RenderCtx>) -> Self {
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

    pub fn push(&mut self, child: GElement<Message, RenderCtx>) {
        self.children.push(child);
    }
}

#[cfg(all(feature = "gpu"))]
impl<Message, RenderCtx> crate::Widget<Message, RenderCtx> for Layer<Message, RenderCtx>
where
    RenderCtx: crate::RenderContext + Clone + PartialEq + 'static,
    Message: 'static,
    // Message: PartialEq + 'static + std::clone::Clone,
{
    // #[instrument(skip(ctx), name = "Layer paint")]
    // fn paint(&self, ctx: &mut crate::PaintCtx<RenderContext>) {
    //     let rect = ctx.size().to_rect();
    //     info!("[layer] print... {}", &rect);

    //     //TODO remove this (debug things)
    //     if self.id == "debug_layer" {
    //         ctx.fill(rect, &emg_native::Color::rgb8(70, 0, 0));
    //     } else {
    //         ctx.fill(rect, &emg_native::Color::rgb8(0, 0, 200));
    //     }
    //     // ctx.fill(rect, &emg_native::Color::rgb8(0, 0, 200));

    //     // ctx.save().unwrap();
    //     for child in &self.children {
    //         child.paint(ctx);
    //     }
    // }

    fn paint_sa(
        &self,
        ctx: &emg_state::StateAnchor<emg_native::PaintCtx<RenderCtx>>,
    ) -> emg_state::StateAnchor<emg_native::PaintCtx<RenderCtx>> {
        let id = self.id.clone();
        let span = illicit::expect::<Span>();

        let mut out_ctx = ctx.map(move |incoming_ctx| {
            // let _span = info_span!(parent:&*span,"layer repaint...").entered();
            info!(parent: &*span,"Layer[{}]::paint -> ctx.map -> recalculating ", &id);
            let mut new_ctx = incoming_ctx.clone();
            let rect = new_ctx.size().to_rect();
            if id == "debug_layer" {
                // new_ctx.fill(rect, &emg_native::Color::rgb8(255, 255, 255));
                new_ctx.fill(rect, &emg_native::renderer::Color::BLACK);
            } else if let Some(fill) = new_ctx.get_fill_color() {
                info!(parent: &*span,"fill color: {:?}", &fill);
                new_ctx.fill(rect, &fill);
            }
            if let Some(bw) = new_ctx.get_border_width() {
                if let Some(bc) = new_ctx.get_border_color() {
                    info!(parent: &*span,"border width: {:?} color: {:?}", &bw, &bc);
                    new_ctx.stroke(rect, &bc, bw);
                } else {
                    new_ctx.stroke(
                        rect.inset(-bw / 2. - 0.),
                        &emg_native::renderer::Color::BLACK,
                        bw,
                    );
                }
            }
            new_ctx
        });
        for child in &self.children {
            out_ctx = child.paint_sa(&out_ctx);
        }
        out_ctx
        // self.children
        //     .clone()
        //     .into_iter()
        //     .fold(out_ctx, |acc_ctx, child| child.paint_sa(&acc_ctx))
    }
}
