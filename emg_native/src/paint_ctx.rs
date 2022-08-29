/*
 * @Author: Rais
 * @Date: 2022-08-18 15:57:30
 * @LastEditTime: 2022-08-29 12:54:29
 * @LastEditors: Rais
 * @Description:
 */

use std::ops::{Deref, DerefMut};

use emg_common::na::Translation3;
use emg_state::StateAnchor;
use tracing::error;

use crate::Size;

//TODO move to global
pub const DPR: f64 = 2.0;

#[derive(Clone, Default, PartialEq)]
pub struct PaintCtx<RenderContext> {
    // pub(crate) state: &'a mut ContextState<'b>,
    widget_state: WidgetState,
    /// The render context for actually painting.
    pub render_ctx: RenderContext,
    // /// The z-order paint operations.
    // pub(crate) z_ops: Vec<ZOrderPaintOp>,
    // /// The currently visible region.
    // pub(crate) region: Region,
    // /// The approximate depth in the tree at the time of painting.
    // pub(crate) depth: u32,
}

impl<RenderContext> PaintCtx<RenderContext>
where
    RenderContext: crate::RenderContext,
{
    pub fn new(widget_state: WidgetState, render_ctx: RenderContext) -> Self {
        Self {
            widget_state,
            render_ctx,
        }
    }

    pub fn size(&self) -> Size {
        self.widget_state.size() * DPR
    }

    pub fn set_widget_state(&mut self, widget_state: WidgetState) {
        self.widget_state = widget_state;
    }

    pub fn with_save(&mut self, f: impl FnOnce(&mut PaintCtx<RenderContext>)) {
        self.render_ctx
            .save()
            .expect("Failed to save RenderContext");
        // if let Err(e) = self.render_ctx.save() {
        //     error!("Failed to save RenderContext: '{}'", e);
        //     return;
        // }

        f(self);

        self.render_ctx
            .restore()
            .expect("Failed to restore RenderContext");
        // if let Err(e) = self.render_ctx.restore() {
        //     error!("Failed to restore RenderContext: '{}'", e);
        // }
    }
}

impl<RenderContext> Deref for PaintCtx<RenderContext> {
    type Target = RenderContext;

    fn deref(&self) -> &Self::Target {
        &self.render_ctx
    }
}

impl<RenderContext> DerefMut for PaintCtx<RenderContext> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.render_ctx
    }
}

#[derive(Copy, Clone, Default, PartialEq, Debug)]
pub struct WidgetState {
    // pub(crate) id: WidgetId,
    /// The size of the child; this is the value returned by the child's layout
    /// method.
    size: Size,
    pub translation: Translation3<f64>,
    // /// The origin of the child in the parent's coordinate space; together with
    // /// `size` these constitute the child's layout rect.
    // origin: Point,
    // /// A flag used to track and debug missing calls to set_origin.
    // is_expecting_set_origin_call: bool,
    // /// The insets applied to the layout rect to generate the paint rect.
    // /// In general, these will be zero; the exception is for things like
    // /// drop shadows or overflowing text.
    // pub(crate) paint_insets: Insets,

    // /// The offset of the baseline relative to the bottom of the widget.
    // ///
    // /// In general, this will be zero; the bottom of the widget will be considered
    // /// the baseline. Widgets that contain text or controls that expect to be
    // /// laid out alongside text can set this as appropriate.
    // pub(crate) baseline_offset: f64,

    // // The region that needs to be repainted, relative to the widget's bounds.
    // pub(crate) invalid: Region,

    // // The part of this widget that is visible on the screen is offset by this
    // // much. This will be non-zero for widgets that are children of `Scroll`, or
    // // similar, and it is used for propagating invalid regions.
    // pub(crate) viewport_offset: Vec2,

    // // TODO: consider using bitflags for the booleans.
    // pub(crate) is_hot: bool,

    // pub(crate) is_active: bool,

    // pub(crate) needs_layout: bool,

    // /// Any descendant is active.
    // has_active: bool,

    // /// In the focused path, starting from window and ending at the focused widget.
    // /// Descendants of the focused widget are not in the focused path.
    // pub(crate) has_focus: bool,

    // /// Any descendant has requested an animation frame.
    // pub(crate) request_anim: bool,

    // /// Any descendant has requested update.
    // pub(crate) request_update: bool,

    // pub(crate) focus_chain: Vec<WidgetId>,
    // pub(crate) request_focus: Option<FocusChange>,
    // pub(crate) children: Bloom<WidgetId>,
    // pub(crate) children_changed: bool,
    // /// Associate timers with widgets that requested them.
    // pub(crate) timers: HashMap<TimerToken, WidgetId>,
    // /// The cursor that was set using one of the context methods.
    // pub(crate) cursor_change: CursorChange,
    // /// The result of merging up children cursors. This gets cleared when merging state up (unlike
    // /// cursor_change, which is persistent).
    // pub(crate) cursor: Option<Cursor>,
}
impl WidgetState {
    // pub(crate) fn new(id: WidgetId, size: Option<Size>) -> WidgetState {
    pub fn new(size: (f64, f64), trans: Translation3<f64>) -> WidgetState {
        WidgetState {
            // id,
            // origin: Point::ORIGIN,
            size: Size::new(size.0, size.1),
            translation: trans,
            // is_expecting_set_origin_call: true,
            // paint_insets: Insets::ZERO,
            // invalid: Region::EMPTY,
            // viewport_offset: Vec2::ZERO,
            // baseline_offset: 0.0,
            // is_hot: false,
            // needs_layout: false,
            // is_active: false,
            // has_active: false,
            // has_focus: false,
            // request_anim: false,
            // request_update: false,
            // request_focus: None,
            // focus_chain: Vec::new(),
            // children: Bloom::new(),
            // children_changed: false,
            // timers: HashMap::new(),
            // cursor_change: CursorChange::Default,
            // cursor: None,
        }
    }

    // pub(crate) fn add_timer(&mut self, timer_token: TimerToken) {
    //     self.timers.insert(timer_token, self.id);
    // }

    // /// Update to incorporate state changes from a child.
    // ///
    // /// This will also clear some requests in the child state.
    // ///
    // /// This method is idempotent and can be called multiple times.
    // fn merge_up(&mut self, child_state: &mut WidgetState) {
    //     let clip = self
    //         .layout_rect()
    //         .with_origin(Point::ORIGIN)
    //         .inset(self.paint_insets);
    //     let offset = child_state.layout_rect().origin().to_vec2() - child_state.viewport_offset;
    //     for &r in child_state.invalid.rects() {
    //         let r = (r + offset).intersect(clip);
    //         if r.area() != 0.0 {
    //             self.invalid.add_rect(r);
    //         }
    //     }
    //     // Clearing the invalid rects here is less fragile than doing it while painting. The
    //     // problem is that widgets (for example, Either) might choose not to paint certain
    //     // invisible children, and we shouldn't allow these invisible children to accumulate
    //     // invalid rects.
    //     child_state.invalid.clear();

    //     self.needs_layout |= child_state.needs_layout;
    //     self.request_anim |= child_state.request_anim;
    //     self.has_active |= child_state.has_active;
    //     self.has_focus |= child_state.has_focus;
    //     self.children_changed |= child_state.children_changed;
    //     self.request_update |= child_state.request_update;
    //     self.request_focus = child_state.request_focus.take().or(self.request_focus);
    //     self.timers.extend_drain(&mut child_state.timers);

    //     // We reset `child_state.cursor` no matter what, so that on the every pass through the tree,
    //     // things will be recalculated just from `cursor_change`.
    //     let child_cursor = child_state.take_cursor();
    //     if let CursorChange::Override(cursor) = &self.cursor_change {
    //         self.cursor = Some(cursor.clone());
    //     } else if child_state.has_active || child_state.is_hot {
    //         self.cursor = child_cursor;
    //     }

    //     if self.cursor.is_none() {
    //         if let CursorChange::Set(cursor) = &self.cursor_change {
    //             self.cursor = Some(cursor.clone());
    //         }
    //     }
    // }

    // /// Because of how cursor merge logic works, we need to handle the leaf case;
    // /// in that case there will be nothing in the `cursor` field (as merge_up
    // /// is never called) and so we need to also check the `cursor_change` field.
    // fn take_cursor(&mut self) -> Option<Cursor> {
    //     self.cursor.take().or_else(|| self.cursor_change.cursor())
    // }

    #[inline]
    pub(crate) fn size(&self) -> Size {
        self.size
    }

    // /// The paint region for this widget.
    // ///
    // /// For more information, see [`WidgetPod::paint_rect`].
    // ///
    // /// [`WidgetPod::paint_rect`]: struct.WidgetPod.html#method.paint_rect
    // pub(crate) fn paint_rect(&self) -> Rect {
    //     self.layout_rect() + self.paint_insets
    // }

    // pub(crate) fn layout_rect(&self) -> Rect {
    //     Rect::from_origin_size(self.origin, self.size)
    // }
}
