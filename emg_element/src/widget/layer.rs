use std::{clone::Clone, cmp::PartialEq, rc::Rc};

use emg_common::IdStr;
use emg_state::{Anchor, StateAnchor, StateMultiAnchor};
use tracing::{info, Span};

use crate::GElement;

// ────────────────────────────────────────────────────────────────────────────────

type LayerChildren<Message> = Vec<GElement<Message>>;

/// A container that distributes its contents vertically.
///
/// A [`Layer`] will try to fill the horizontal space of its container.
//TODO remove all missing_debug_implementations
#[allow(missing_debug_implementations)]
#[derive(Eq)]
pub struct Layer<Message> {
    id: IdStr,
    //TODO vec?
    children: LayerChildren<Message>,
}

impl<Message> std::fmt::Debug for Layer<Message>
where
    Message: 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Layer")
            .field("id", &self.id)
            .field("children", &self.children)
            .finish()
    }
}

impl<Message> Clone for Layer<Message> {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            children: self.children.clone(),
        }
    }
}
impl<Message> PartialEq for Layer<Message>
// where
//     Message: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.children == other.children
    }
}

impl<Message> Default for Layer<Message> {
    fn default() -> Self {
        Self::new(IdStr::new_inline(""))
    }
}

impl<Message> Layer<Message> {
    /// Creates an empty [`Layer`].
    #[must_use]
    pub fn new(id: IdStr) -> Self {
        Self::with_children(id, LayerChildren::<Message>::new())
    }

    /// Creates a [`Layer`] with the given elements.
    #[must_use]
    pub fn with_children(id: IdStr, children: LayerChildren<Message>) -> Self {
        Self { id, children }
    }

    #[must_use]
    pub fn set_children(mut self, children: LayerChildren<Message>) -> Self {
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

    pub fn push(&mut self, child: GElement<Message>) {
        self.children.push(child);
    }
}

#[cfg(all(feature = "gpu"))]
use crate::renderer::*;
#[cfg(all(feature = "gpu"))]
impl<Message> crate::Widget for Layer<Message>
where
    Message: 'static,
{
    type SceneCtxType = crate::SceneFrag;
    fn paint_sa(&self, ctx: &StateAnchor<crate::PaintCtx>) -> StateAnchor<Rc<Self::SceneCtxType>> {
        let id = self.id.clone();
        let span = illicit::expect::<Span>();

        let children_sc_list_sa: StateAnchor<Vec<Rc<crate::SceneFrag>>> = self
            .children
            .iter()
            .map(|child| child.paint_sa(&ctx).into_anchor())
            .collect::<Anchor<Vec<_>>>()
            .into();

        (ctx, &children_sc_list_sa).map(move |incoming_ctx, children_sc_list| {
            let mut sc = Self::SceneCtxType::new(incoming_ctx.get_translation());
            let mut builder = sc.gen_builder();

            let rect = incoming_ctx.size().to_rect();
            if id == "debug_layer" {
                builder.fill(Fill::NonZero, Affine::IDENTITY, Color::BLACK, None, &rect);
            } else if let Some(fill) = incoming_ctx.get_fill_color() {
                info!(parent: &*span,"fill color: {:?}", &fill);
                builder.fill(Fill::NonZero, Affine::IDENTITY, fill, None, &rect);
            }
            if let Some(bw) = incoming_ctx.get_border_width() {
                if let Some(bc) = incoming_ctx.get_border_color() {
                    info!(parent: &*span,"border width: {:?} color: {:?}", &bw, &bc);

                    builder.stroke(&Stroke::new(bw as f32), Affine::IDENTITY, bc, None, &rect);
                } else {
                    builder.stroke(
                        &Stroke::new(bw as f32),
                        Affine::IDENTITY,
                        Color::BLACK,
                        None,
                        &rect.inset(-bw / 2. - 0.), //TODO 检查,这是临时设置
                    );
                }
            }

            children_sc_list
                .iter()
                .for_each(|sc| builder.append(sc, sc.get_transform()));

            // ─────────────────────────────────────────────

            builder.finish();
            Rc::new(sc)
        })
    }
}
