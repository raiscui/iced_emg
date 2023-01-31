/*
 * @Author: Rais
 * @Date: 2021-09-01 09:58:44
 * @LastEditTime: 2023-01-31 21:22:31
 * @LastEditors: Rais
 * @Description:
 */
//! Show toggle controls using checkboxes.
use crate::{g_element::DynGElement, GElement};

use emg_common::{
    any::MessageTid,
    better_any::{Tid, TidAble, TidExt},
    IdStr, LogicLength, TypeCheckObjectSafe, TypeName,
};
use emg_shaping::{Shaping, ShapingUse, TryShapingUse};
use emg_state::StateAnchor;
use tracing::{error, info, trace, warn, Span};

use std::{any::Any, rc::Rc};

#[allow(missing_debug_implementations)]
#[derive(Tid)]
pub struct Checkbox<Message>
// where
//     dyn std::ops::Fn(bool) -> Message + 'static: std::cmp::PartialEq,
{
    is_checked: bool,
    //FIXME use cow for Rc 防止 克隆对象和 原始对象使用同一个 callback
    on_toggle: Rc<dyn Fn(bool) -> Message>,
    label: IdStr,
    id: Option<IdStr>,
    width: LogicLength,
    // #[allow(dead_code)]
    // style: Box<dyn StyleSheet>,
}

impl<Message> Eq for Checkbox<Message> {}

impl<Message> Clone for Checkbox<Message> {
    fn clone(&self) -> Self {
        Self {
            is_checked: self.is_checked.clone(),
            on_toggle: self.on_toggle.clone(),
            label: self.label.clone(),
            id: self.id.clone(),
            width: self.width.clone(),
        }
    }
}

impl<Message> std::fmt::Debug for Checkbox<Message> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Checkbox")
            .field("is_checked", &self.is_checked)
            .field("label", &self.label)
            .field("id", &self.id)
            .field("width", &self.width)
            .finish()
    }
}
impl<Message> PartialEq for Checkbox<Message>
// where
// Message: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.is_checked == other.is_checked
            && std::ptr::eq(
                (std::ptr::addr_of!(*self.on_toggle)).cast::<u8>(),
                (std::ptr::addr_of!(*other.on_toggle)).cast::<u8>(),
            )
            && self.label == other.label
            && self.id == other.id
            && self.width == other.width
    }
}

impl<Message> Checkbox<Message>
// where
//     dyn std::ops::Fn(bool) -> Message + 'static: std::cmp::PartialEq,
{
    /// Creates a new [`Checkbox`].
    ///
    /// It expects:
    ///   * a boolean describing whether the [`Checkbox`] is checked or not
    ///   * the label of the [`Checkbox`]
    ///   * a function that will be called when the [`Checkbox`] is toggled. It
    ///     will receive the new state of the [`Checkbox`] and must produce a
    ///     `Message`.
    pub fn new<F>(is_checked: bool, label: impl Into<IdStr>, f: F) -> Self
    where
        F: 'static + Fn(bool) -> Message,
    {
        Self {
            is_checked,
            on_toggle: Rc::new(f),
            label: label.into(),
            id: None,
            width: LogicLength::default(),
            // style: std::boxed::Box::default(),
        }
    }

    /// Sets the width of the [`Checkbox`].
    #[must_use]
    pub fn width(mut self, width: LogicLength) -> Self {
        self.width = width;
        self
    }

    // /// Sets the style of the [`Checkbox`].
    // #[must_use]
    // pub fn style(mut self, style: impl Into<Box<dyn StyleSheet>>) -> Self {
    //     self.style = style.into();
    //     self
    // }

    /// Sets the id of the [`Checkbox`].
    #[must_use]
    pub fn id(mut self, id: impl Into<IdStr>) -> Self {
        self.id = Some(id.into());
        self
    }

    #[must_use]
    pub fn with_label(mut self, label: IdStr) -> Self {
        self.label = label;
        self
    }
}

#[cfg(all(feature = "gpu"))]
use crate::renderer::*;
#[cfg(all(feature = "gpu"))]
impl<Message> crate::Widget for Checkbox<Message>
where
    Message: 'static,
{
    type SceneCtxType = crate::SceneFrag;
    fn paint_sa(&self, ctx: &StateAnchor<crate::PaintCtx>) -> StateAnchor<Rc<Self::SceneCtxType>> {
        let span = illicit::expect::<Span>();

        ctx.map(move |incoming_ctx| {
            let mut sc = Self::SceneCtxType::new(incoming_ctx.get_translation());
            let mut builder = sc.gen_builder();

            let rect = incoming_ctx.size().to_rect();
            //fill
            if let Some(fill) = incoming_ctx.get_fill_color() {
                info!(parent: &*span,"fill color: {:?}", &fill);
                builder.fill(Fill::NonZero, Affine::IDENTITY, fill, None, &rect);
            }
            // checkbox
            let origin = (rect.height() - 14. * incoming_ctx.dpr()) * 0.5;

            let box_rect = Rect {
                x0: 0.,
                y0: 0.,
                x1: 14. * incoming_ctx.dpr(),
                y1: 14. * incoming_ctx.dpr(),
            }
            .with_origin((origin, origin))
            .to_rounded_rect(3.);

            builder.stroke(
                &Stroke::new(1. * incoming_ctx.dpr() as f32),
                Affine::IDENTITY,
                Color::BLACK,
                None,
                &box_rect,
            );

            // border
            if let Some(bw) = incoming_ctx.get_border_width() {
                if let Some(bc) = incoming_ctx.get_border_color() {
                    info!(parent: &*span,"border width: {:?} color: {:?}", &bw, &bc);

                    builder.stroke(&Stroke::new(bw), Affine::IDENTITY, bc, None, &rect);
                } else {
                    // has border width but no border color
                    builder.stroke(
                        &Stroke::new(bw),
                        Affine::IDENTITY,
                        Color::BLACK,
                        None,
                        &rect.inset(-(bw as f64) / 2. - 0.), //TODO 检查,这是临时设置
                    );
                }
            }

            // ─────────────────────────────────────────────

            builder.finish();
            Rc::new(sc)
        })
    }
}

impl<'a, Message> Shaping<Self> for Checkbox<Message>
where
    Message: 'static + Clone + MessageTid<'a>,
{
    fn shaping(&self, who: &mut Self) {
        trace!(
            "Generic: use Checkbox refresh for checkbox self:{}-who:{}",
            &who.label,
            &self.label
        );

        *who = self.clone();
    }
}
// impl<'a, Message, T: Shaping<Checkbox<Message>>> AsRefreshFor<Checkbox<Message>> for T {
//     fn as_refresh_for(&self) -> &dyn Shaping<Checkbox<Message>> {
//         self
//     }
// }

// @ 被GElement更新自己 ------------------------------------
impl<Message> Shaping<Checkbox<Message>> for GElement<Message>
where
    Message: 'static + Clone + for<'a> MessageTid<'a> + std::cmp::PartialEq,
{
    #[allow(clippy::match_same_arms)]
    fn shaping(&self, who_checkbox: &mut Checkbox<Message>) {
        match self {
            Self::Layer_(_l) => {
                unimplemented!();
            }
            Self::Builder_(builder) => {
                builder.widget().shaping(who_checkbox);
            }
            //TODO enable this
            // Self::Text_(t) => {
            //     who_checkbox.label = t.get_content(); //TODO text.get_content directly return IdStr
            // }
            // Self::Button_(_) => {
            //     unimplemented!();
            // }
            Self::Event_(_) => {
                todo!();
            }
            Self::Generic_(g_self) => {
                error!("use Generic shaping Checkbox :{}", g_self.type_name());

                //TODO 反射?
                // todo!("reflection? ",);
            }
            Self::NodeRef_(_) => panic!("GElement::NodeIndex_() should handle before."),
            Self::EmptyNeverUse => panic!("EmptyNeverUse never here"),
            Self::SaNode_(_) => todo!(),

            Self::EvolutionaryFactor(_) => todo!(),

            GElement::Shaper_(_) => todo!(),
        };
    }
}

// @ 用于更新who -GElement ------------------------------------
impl<Message> Shaping<GElement<Message>> for Checkbox<Message>
where
    Message: 'static + Clone + for<'a> MessageTid<'a> + std::cmp::PartialEq,
{
    #[allow(clippy::match_same_arms)]
    fn shaping(&self, who: &mut GElement<Message>) {
        match who {
            GElement::Layer_(l) => {
                l.push(self.clone().into());
            }
            GElement::Builder_(builder) => {
                self.shaping(builder.widget_mut());
            }
            // GElement::Text_(_)
            // | GElement::Button_(_)
            // |
            GElement::Shaper_(_) | GElement::Event_(_) => {
                unimplemented!();
            }
            GElement::Generic_(g_who) => {
                trace!("use Checkbox shaping Generic");
                let dyn_who = g_who.as_mut();

                if let Some(checkbox) = dyn_who.downcast_mut::<Self>() {
                    self.shaping(checkbox);
                }
            }
            GElement::NodeRef_(_) => panic!("GElement::NodeIndex_() should handle before."),
            GElement::EmptyNeverUse => panic!("EmptyNeverUse never here"),
            GElement::SaNode_(_) => todo!(),
            GElement::EvolutionaryFactor(_) => todo!(),
        };
    }
}

//TODO use macro
impl<Message> TypeCheckObjectSafe for Checkbox<Message> {
    fn type_name(&self) -> TypeName {
        TypeName::new(IdStr::new_inline("Checkbox"))
    }
}

impl<Message> DynGElement<Message> for Checkbox<Message> where
    Message: Clone + 'static + for<'a> MessageTid<'a> + std::cmp::PartialEq
{
}

impl<Message> From<Checkbox<Message>> for GElement<Message>
where
    Message: Clone + for<'a> MessageTid<'a> + std::cmp::PartialEq + 'static,
{
    fn from(checkbox: Checkbox<Message>) -> Self {
        Self::Generic_(Box::new(checkbox))
    }
}
// ────────────────────────────────────────────────────────────────────────────────
// impl<Message> ShapingUse<i32> for Checkbox<Message> {
//     fn shaping_use(&mut self, use_something: &i32) {
//         self.label = format!("checkbox i32: {}", use_something).into()
//     }
// }

impl<'a, Message> Shaping<Checkbox<Message>> for i32
where
    Message: 'static + Clone + MessageTid<'a>,
{
    fn shaping(&self, who: &mut Checkbox<Message>) {
        warn!(
            "[checkbox] use i32 refresh for checkbox self:{:?}-who:{}",
            &who, &self
        );

        who.label = format!("checkbox i32: {}", self).into();
    }
}
// impl<Message, Use: Sized + Clone + std::fmt::Debug + 'static> TryRefreshFor<Checkbox<Message>>
//     for Rc<Use>
// {
//     fn try_refresh_for(&self, who: &mut Checkbox<Message>) {
//         warn!(
//             "[try_refresh_for] self:{} try downcast to Rc<dyn Shaping<{}>>",
//             std::any::type_name::<Self>(),
//             std::any::type_name::<Checkbox<Message>>()
//         );
//         let u = self.clone();
//         let any: &dyn Any = &u;
//         if let Some(u_s_e) = any.downcast_ref::<Rc<dyn Shaping<Checkbox<Message>>>>() {
//             who.shape_of_use(&**u_s_e);
//         } else {
//             warn!("try_refresh failed: use {:?} for who:{:?}", &self, &who);
//         }
//     }
// }

impl<'a, Message> TryShapingUse for Checkbox<Message>
where
    Message: 'static + Clone + MessageTid<'a>,
{
    fn try_shaping_use(&mut self, any: Box<dyn Any>) {
        warn!(
            "[try_shaping_use]  try downcast to Rc<dyn Shaping<{}>>",
            std::any::type_name::<Self>()
        );
        if let Some(x) = any.downcast_ref::<Box<dyn Shaping<Self>>>() {
            self.shaping_use(x);
        }
        // if let Some(u_s_e_rf) = any.downcast_ref::<Rc<dyn Shaping<Self>>>() {
        //     self.shape_of_use(&**u_s_e_rf);
        // } else {
        //     warn!("try_refresh failed: use {:?} for who:{:?}", &self, &any);
        // }
    }
}
