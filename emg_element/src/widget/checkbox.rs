/*
 * @Author: Rais
 * @Date: 2021-09-01 09:58:44
 * @LastEditTime: 2022-08-12 16:59:32
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
use emg_refresh::{RefreshFor, RefreshUse, TryRefreshUse};
use tracing::{error, trace, warn};

use std::{any::Any, ops::Deref, rc::Rc};

#[allow(missing_debug_implementations)]
#[derive(Clone, Tid)]
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
where
    Message: PartialEq,
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

impl<'a, Message> RefreshFor<Self> for Checkbox<Message>
where
    Message: 'static + Clone + MessageTid<'a>,
{
    fn refresh_for(&self, who: &mut Self) {
        trace!(
            "Generic: use Checkbox refresh for checkbox self:{}-who:{}",
            &who.label,
            &self.label
        );

        *who = self.clone();
    }
}
// impl<'a, Message, T: RefreshFor<Checkbox<Message>>> AsRefreshFor<Checkbox<Message>> for T {
//     fn as_refresh_for(&self) -> &dyn RefreshFor<Checkbox<Message>> {
//         self
//     }
// }

// @ 被GElement更新自己 ------------------------------------
impl<Message> RefreshFor<Checkbox<Message>> for GElement<Message>
where
    Message: 'static + Clone + for<'a> MessageTid<'a> + std::cmp::PartialEq,
{
    #[allow(clippy::match_same_arms)]
    fn refresh_for(&self, who_checkbox: &mut Checkbox<Message>) {
        match self {
            Self::Layer_(_l) => {
                unimplemented!();
            }
            Self::Builder_(builder) => {
                builder.widget().unwrap().deref().refresh_for(who_checkbox);
            }
            Self::Text_(t) => {
                who_checkbox.label = t.get_content(); //TODO text.get_content directly return IdStr
            }
            Self::Button_(_) => {
                unimplemented!();
            }
            Self::Refresher_(_refresher) => {
                // NOTE this is refresh_for GElement , not Checkbox
                unimplemented!();
            }
            Self::Event_(_) => {
                todo!();
            }
            Self::Generic_(g_self) => {
                error!("use Generic refresh_for Checkbox :{}", g_self.type_name());

                //TODO 反射?
                // todo!("reflection? ",);
            }
            Self::NodeRef_(_) => panic!("GElement::NodeIndex_() should handle before."),
            Self::EmptyNeverUse => panic!("EmptyNeverUse never here"),
            Self::SaNode_(_) => todo!(),

            Self::EvolutionaryFactor(_) => todo!(),
        };
    }
}

// @ 用于更新who -GElement ------------------------------------
impl<Message> RefreshFor<GElement<Message>> for Checkbox<Message>
where
    Message: 'static + Clone + for<'a> MessageTid<'a> + std::cmp::PartialEq,
{
    #[allow(clippy::match_same_arms)]
    fn refresh_for(&self, who: &mut GElement<Message>) {
        match who {
            GElement::Layer_(l) => {
                l.push(self.clone().into());
            }
            GElement::Builder_(builder) => {
                if let Some(box gel) = builder.widget_mut() {
                    self.refresh_for(gel);
                } else {
                    panic!("builder not has widget, in [RefreshFor<GElement<Message>> for Checkbox<Message>] ")
                }
            }
            GElement::Text_(_)
            | GElement::Button_(_)
            | GElement::Refresher_(_)
            | GElement::Event_(_) => {
                unimplemented!();
            }
            GElement::Generic_(g_who) => {
                trace!("use Checkbox refresh_for Generic");
                let dyn_who = g_who.as_mut();

                if let Some(checkbox) = dyn_who.downcast_mut::<Self>() {
                    self.refresh_for(checkbox);
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
// impl<Message> RefreshUse<i32> for Checkbox<Message> {
//     fn refresh_use(&mut self, use_something: &i32) {
//         self.label = format!("checkbox i32: {}", use_something).into()
//     }
// }

impl<'a, Message> RefreshFor<Checkbox<Message>> for i32
where
    Message: 'static + Clone + MessageTid<'a>,
{
    fn refresh_for(&self, who: &mut Checkbox<Message>) {
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
//             "[try_refresh_for] self:{} try downcast to Rc<dyn RefreshFor<{}>>",
//             std::any::type_name::<Self>(),
//             std::any::type_name::<Checkbox<Message>>()
//         );
//         let u = self.clone();
//         let any: &dyn Any = &u;
//         if let Some(u_s_e) = any.downcast_ref::<Rc<dyn RefreshFor<Checkbox<Message>>>>() {
//             who.refresh_for_use(&**u_s_e);
//         } else {
//             warn!("try_refresh failed: use {:?} for who:{:?}", &self, &who);
//         }
//     }
// }

impl<'a, Message> TryRefreshUse for Checkbox<Message>
where
    Message: 'static + Clone + MessageTid<'a>,
{
    fn try_refresh_use(&mut self, any: Box<dyn Any>) {
        warn!(
            "[try_refresh_use]  try downcast to Rc<dyn RefreshFor<{}>>",
            std::any::type_name::<Self>()
        );
        if let Some(x) = any.downcast_ref::<Box<dyn RefreshFor<Self>>>() {
            self.refresh_use(x);
        }
        // if let Some(u_s_e_rf) = any.downcast_ref::<Rc<dyn RefreshFor<Self>>>() {
        //     self.refresh_for_use(&**u_s_e_rf);
        // } else {
        //     warn!("try_refresh failed: use {:?} for who:{:?}", &self, &any);
        // }
    }
}
