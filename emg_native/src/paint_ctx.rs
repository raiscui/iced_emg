/*
 * @Author: Rais
 * @Date: 2022-08-18 15:57:30
 * @LastEditTime: 2023-01-11 17:29:13
 * @LastEditors: Rais
 * @Description:
 */

mod impl_refresh;
use std::{cell::Cell, rc::Rc};

use crate::renderer::{Affine, Color, Size};
use emg_common::{na::Translation3, LayoutOverride, Vector};
use emg_shaping::ShapingWhoNoWarper;
use emg_state::{state_lit::StateVarLit, StateAnchor};
use seed_styles::{CssBorderColor, CssBorderWidth, CssFill};
use tracing::{debug, info};

//TODO use app state viewport dpr
//TODO use  window.scale_factor()
pub const DPR: f64 = 2.0;

/// used for check restore right
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

#[derive(Debug, Clone, PartialEq, Default)]
pub struct PaintCtx {
    // pub(crate) state: &'a mut ContextState<'b>,
    widget_state: WidgetState,
    /// The render context for actually painting.
    // pub scene_ctx: SceneCtx,
    widget_state_stack: Vector<WidgetState>,
    // /// The z-order paint operations.
    // pub(crate) z_ops: Vec<ZOrderPaintOp>,
    // /// The currently visible region.
    // pub(crate) region: Region,
    // /// The approximate depth in the tree at the time of painting.
    // pub(crate) depth: u32,
}

impl PaintCtx {
    pub fn size(&self) -> Size {
        //TODO move DPR to const T
        self.widget_state.size() * DPR
    }
    pub fn get_translation(&self) -> Option<Affine> {
        let t = self.widget_state.translation;
        if t.x == 0. && t.y == 0. {
            None
        } else {
            Some(crate::renderer::Affine::translate((t.x * DPR, t.y * DPR)))
        }
    }
    pub fn get_fill_color(&self) -> Option<Color> {
        self.widget_state.fill.as_ref().map(|fill| match *fill {
            CssFill::Rgba(r, g, b, a) => {
                // debug!("CssFill::Rgba( {:?}, {:?}, {:?}, {:?})", r, g, b, a);
                Color::rgba(r, g, b, a)
            }
            CssFill::Hsl(h, s, l) => {
                // debug!("CssFill::hsl(  {:?}, {:?}, {:?})", h, s, l);
                Color::hlc(h, l, s / 100. * 127.)
            }
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
        // self.scene_ctx.save().expect("Failed to save RenderContext");
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
        // self.scene_ctx
        //     .restore()
        //     .expect("Failed to restore RenderContext");

        let widget_state = self
            .widget_state_stack
            .pop_back()
            .expect("widget_state_stack pop error");
        self.widget_state = widget_state;
    }
}

impl ShapingWhoNoWarper for WidgetState {}
#[derive(Clone, PartialEq, Debug)]
pub struct WidgetState {
    pub children_layout_override: StateAnchor<Option<LayoutOverride>>,
    size: Size,
    pub translation: Translation3<f64>,
    pub world: StateAnchor<Translation3<f64>>,
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

impl Default for WidgetState {
    fn default() -> Self {
        Self {
            children_layout_override: StateAnchor::constant(None),
            size: Default::default(),
            translation: Default::default(),
            world: StateAnchor::constant(Translation3::default()),
            fill: Default::default(),
            border_width: Default::default(),
            border_color: Default::default(),
        }
    }
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
    pub fn new(
        size: (f64, f64),
        trans: Translation3<f64>,
        world: StateAnchor<Translation3<f64>>,
        children_layout_override: StateAnchor<Option<LayoutOverride>>,
    ) -> WidgetState {
        WidgetState {
            // id,
            // origin: Point::ORIGIN,
            size: Size::new(size.0, size.1),
            translation: trans,
            world,
            children_layout_override,
            fill: None,
            border_width: None,
            border_color: None,
        }
    }
    pub fn merge(&mut self, new_current: &Self) {
        self.size = new_current.size;
        self.translation = new_current.translation;
        self.world = new_current.world.clone();
        self.children_layout_override = new_current.children_layout_override.clone();
        // match &other.fill {
        //     Some(fill) => match fill {
        //         CssFill::Inherit => (),
        //         other_fill => self.fill = Some(other_fill.clone()),
        //     },
        //     None => self.fill = None,
        // };

        //NOTE because the css Inherit
        //混合的时候，如果是Inherit，就不要覆盖了
        //如果是其他值,self 的值就覆盖为 new_current 的值
        css_merge!(self, new_current, CssFill, fill);
        css_merge!(self, new_current, CssBorderWidth, border_width);
        css_merge!(self, new_current, CssBorderColor, border_color);
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
