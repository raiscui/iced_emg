/*
 * @Author: Rais
 * @Date: 2022-08-18 15:57:30
 * @LastEditTime: 2022-09-06 10:27:08
 * @LastEditors: Rais
 * @Description:
 */

mod impl_refresh;
use std::{
    cell::Cell,
    ops::{Deref, DerefMut},
    rc::Rc,
};

use crate::renderer::{Color, Size};
use emg_common::{na::Translation3, Vector};
use seed_styles::{CssBorderColor, CssBorderWidth, CssFill};
use tracing::info;

//TODO move to global
pub const DPR: f64 = 2.0;

#[derive(Clone)]
pub struct CtxIndex(Rc<Cell<usize>>);

impl Default for CtxIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl CtxIndex {
    pub fn new() -> Self {
        Self(Rc::new(Cell::new(0)))
    }

    pub fn set(&self, val: usize) {
        self.0.set(val)
    }

    pub fn get(&self) -> usize {
        self.0.get()
    }
}

#[derive(Clone, Default, PartialEq)]
pub struct PaintCtx<RenderContext> {
    // pub(crate) state: &'a mut ContextState<'b>,
    widget_state: WidgetState,
    /// The render context for actually painting.
    pub render_ctx: RenderContext,

    widget_state_stack: Vector<WidgetState>,
    // /// The z-order paint operations.
    // pub(crate) z_ops: Vec<ZOrderPaintOp>,
    // /// The currently visible region.
    // pub(crate) region: Region,
    // /// The approximate depth in the tree at the time of painting.
    // pub(crate) depth: u32,
}

impl<RenderCtx> PaintCtx<RenderCtx>
where
    RenderCtx: crate::renderer::RenderContext,
{
    pub fn new(widget_state: WidgetState, render_ctx: RenderCtx) -> Self {
        Self {
            widget_state,
            render_ctx,
            widget_state_stack: Default::default(),
        }
    }

    pub fn size(&self) -> Size {
        //TODO move DPR to const T
        self.widget_state.size() * DPR
    }
    pub fn get_fill_color(&self) -> Option<Color> {
        self.widget_state.fill.as_ref().map(|fill| match *fill {
            CssFill::Rgba(r, g, b, a) => Color::rgba(r, g, b, a),
            CssFill::Hsl(_, _, _) => todo!(),
            CssFill::Hsla(_, _, _, _) => todo!(),
            CssFill::Hex(_) => todo!(),
            CssFill::StringValue(_) => todo!(),
            CssFill::Inherit => todo!("get stack latest"),
        })
    }
    // #[instrument(skip(self), ret)]
    pub fn get_border_width(&self) -> Option<f64> {
        self.widget_state.border_width.as_ref().map(|bw| match bw {
            CssBorderWidth::Medium => todo!(),
            CssBorderWidth::Thin => todo!(),
            CssBorderWidth::Thick => todo!(),
            CssBorderWidth::Length(l) => l
                .try_get_number()
                .expect("[Unit] currently only px /empty can get"),
            CssBorderWidth::Initial => todo!(),
            CssBorderWidth::Inherit => todo!(),
            CssBorderWidth::StringValue(_) => todo!(),
        })
    }
    // #[instrument(skip(self), ret)]
    pub fn get_border_color(&self) -> Option<Color> {
        self.widget_state.border_color.as_ref().map(|bc| match *bc {
            CssBorderColor::Rgba(r, g, b, a) => Color::rgba(r, g, b, a),
            CssBorderColor::Hsl(_, _, _) => todo!(),
            CssBorderColor::Hsla(_, _, _, _) => todo!(),
            CssBorderColor::Hex(_) => todo!(),
            CssBorderColor::StringValue(_) => todo!(),
            CssBorderColor::Inherit => todo!(),
        })
    }

    pub fn merge_widget_state(&mut self, widget_state: &WidgetState) {
        //TODO make overwrite
        self.widget_state.merge(widget_state);
    }

    pub fn save(&mut self) {
        self.widget_state_stack.push_back(self.widget_state.clone());
        self.render_ctx
            .save()
            .expect("Failed to save RenderContext");
    }

    pub fn save_assert(&mut self, index: &CtxIndex) {
        // let s_len = self.widget_state_stack.len();
        index.set(self.widget_state_stack.len());

        // let index_len = index.get();
        // info!("[save_assert], s_len: {} index_len: {} ", s_len, index_len);

        self.save();
    }
    pub fn restore_assert(&mut self, index: &CtxIndex) {
        self.restore();
        let s_len = self.widget_state_stack.len();
        let index_len = index.get();
        // info!(
        //     "[restore_assert], s_len: {} index_len: {}",
        //     s_len, index_len
        // );
        assert!(s_len == index_len);
    }
    pub fn restore(&mut self) {
        self.render_ctx
            .restore()
            .expect("Failed to restore RenderContext");

        let widget_state = self
            .widget_state_stack
            .pop_back()
            .expect("widget_state_stack pop error");
        self.widget_state = widget_state;
    }

    pub fn with_save(&mut self, f: impl FnOnce(&mut PaintCtx<RenderCtx>)) {
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

#[derive(Clone, PartialEq, Debug, Default)]
pub struct WidgetState {
    // pub(crate) id: WidgetId,
    /// The size of the child; this is the value returned by the child's layout
    /// method.
    size: Size,
    pub translation: Translation3<f64>,
    // pub background_color: CssBackgroundColor,
    pub fill: Option<CssFill>,
    pub border_width: Option<CssBorderWidth>,
    pub border_color: Option<CssBorderColor>,
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
macro_rules! css_merge {
    ($self:ident,$other:ident,$css:ident,$v:ident) => {
        match &$other.$v {
            Some(val) => match val {
                $css::Inherit => (),
                other_val => $self.$v = Some(other_val.clone()),
            },
            None => $self.$v = None,
        };
    };
}

impl WidgetState {
    // pub(crate) fn new(id: WidgetId, size: Option<Size>) -> WidgetState {
    pub fn new(size: (f64, f64), trans: Translation3<f64>) -> WidgetState {
        WidgetState {
            // id,
            // origin: Point::ORIGIN,
            size: Size::new(size.0, size.1),
            translation: trans,
            ..Self::default()
        }
    }
    pub fn merge(&mut self, other: &Self) {
        self.size = other.size;
        self.translation = other.translation;
        // match &other.fill {
        //     Some(fill) => match fill {
        //         CssFill::Inherit => (),
        //         other_fill => self.fill = Some(other_fill.clone()),
        //     },
        //     None => self.fill = None,
        // };
        css_merge!(self, other, CssFill, fill);
        css_merge!(self, other, CssBorderWidth, border_width);
        css_merge!(self, other, CssBorderColor, border_color);
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
    pub fn size(&self) -> Size {
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
