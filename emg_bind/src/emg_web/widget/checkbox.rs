/*
 * @Author: Rais
 * @Date: 2021-09-01 09:58:44
 * @LastEditTime: 2022-06-07 19:09:03
 * @LastEditors: Rais
 * @Description:
 */
//! Show toggle controls using checkboxes.
use crate::{
    emg_runtime::{Bus, Element, Widget},
    iced_runtime::{css, Length},
    DynGElement, GElement, GenerateElement, MessageTid, NodeBuilder,
};

#[allow(unused_imports)]
use better_any::{impl_tid, tid, type_id, Tid, TidAble, TidExt};

use dyn_partial_eq::DynPartialEq;
use emg_core::{IdStr, TypeCheckObjectSafe, TypeName};
use emg_refresh::RefreshFor;
pub use iced_style::checkbox::{Style, StyleSheet};
use seed_styles::GlobalStyleSV;
use tracing::trace;

use crate::emg_runtime::dodrio::bumpalo;
use std::{ops::Deref, rc::Rc};

/// A box that can be checked.
///
/// # Example
///
/// ```
/// # use emg_bind::Checkbox;
///
/// pub enum Message {
///     CheckboxToggled(bool),
/// }
///
/// let is_checked = true;
///
/// Checkbox::new(is_checked, "Toggle me!", Message::CheckboxToggled);
/// ```
///
/// ![Checkbox drawn by Coffee's renderer](https://github.com/hecrj/coffee/blob/bda9818f823dfcb8a7ad0ff4940b4d4b387b5208/images/ui/checkbox.png?raw=true)
#[allow(missing_debug_implementations)]
#[derive(Clone, Tid, DynPartialEq)]
#[eq_opt(where_add = "Message: PartialEq + 'static,")]
pub struct Checkbox<Message>
// where
//     dyn std::ops::Fn(bool) -> Message + 'static: std::cmp::PartialEq,
{
    is_checked: bool,
    //FIXME use cow for Rc 防止 克隆对象和 原始对象使用同一个 callback
    on_toggle: Rc<dyn Fn(bool) -> Message>,
    label: IdStr,
    id: Option<IdStr>,
    width: Length,
    // #[allow(dead_code)]
    // style: Box<dyn StyleSheet>,
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
            width: Length::Shrink,
            // style: std::boxed::Box::default(),
        }
    }

    /// Sets the width of the [`Checkbox`].
    #[must_use]
    pub const fn width(mut self, width: Length) -> Self {
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
}

impl<Message> NodeBuilder<Message> for Checkbox<Message>
where
    Message: 'static + Clone + std::cmp::PartialEq,
{
    fn generate_element_builder<'b>(
        &self,
        bump: &'b bumpalo::Bump,
        bus: &Bus<Message>,
        _style_sheet: &GlobalStyleSV,
    ) -> dodrio::builder::ElementBuilder<
        'b,
        bumpalo::collections::Vec<'b, dodrio::Listener<'b>>,
        bumpalo::collections::Vec<'b, dodrio::Attribute<'b>>,
        bumpalo::collections::Vec<'b, dodrio::Node<'b>>,
    > {
        use dodrio::builder::{input, label, text};
        use dodrio::bumpalo::collections::String;

        let checkbox_label = String::from_str_in(&self.label, bump).into_bump_str();

        let event_bus = bus.clone();
        let on_toggle = self.on_toggle.clone();
        let is_checked = self.is_checked;

        // let row_class = style_sheet.insert(bump, css::Rule::Row);

        // let spacing_class = style_sheet.insert(bump, css::Rule::Spacing(5));

        let (label, input) = self.id.as_ref().map_or_else(
            || (label(bump), input(bump)),
            |id| {
                let id = String::from_str_in(id, bump).into_bump_str();

                (label(bump).attr("for", id), input(bump).attr("id", id))
            },
        );

        label
            // .attr(
            //     "class",
            //     bumpalo::format!(in bump, "{} {}", row_class, spacing_class)
            //         .into_bump_str(),
            // )
            .attr(
                "style",
                bumpalo::format!(in bump, "width: {}; align-items: center", css::length(self.width))
                    .into_bump_str(),
            )
            .children(bumpalo::vec![in bump;
                // TODO: Checkbox styling
                 input
                    .attr("type", "checkbox")
                    .bool_attr("checked", self.is_checked)
                    .on("click", move |_root, vdom, _event| {
                        let msg = on_toggle(!is_checked);
                        event_bus.publish(msg);

                        vdom.schedule_render();
                    })
                    .finish(),
                text(checkbox_label),
            ])
    }
}

impl<Message> Widget<Message> for Checkbox<Message>
where
    Message: 'static + Clone + std::cmp::PartialEq,
{
    fn node<'b>(
        &self,
        bump: &'b bumpalo::Bump,
        bus: &Bus<Message>,
        style_sheet: &GlobalStyleSV,
    ) -> dodrio::Node<'b> {
        self.generate_element_builder(bump, bus, style_sheet)
            .finish()
    }
}

impl<Message> From<Checkbox<Message>> for Element<Message>
where
    Message: 'static + Clone + std::cmp::PartialEq,
{
    fn from(checkbox: Checkbox<Message>) -> Self {
        Self::new(checkbox)
    }
}

impl<Message> GenerateElement<Message> for Checkbox<Message>
where
    Message: 'static + Clone + std::cmp::PartialEq,
{
    fn generate_element(&self) -> Element<Message> {
        //TODO remove ref? not clone?
        Element::new(self.clone())
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

// @ 被GElement更新 ------------------------------------
impl<Message> RefreshFor<Checkbox<Message>> for GElement<Message>
where
    Message: 'static + Clone + for<'a> MessageTid<'a> + std::cmp::PartialEq,
{
    fn refresh_for(&self, who_checkbox: &mut Checkbox<Message>) {
        match self {
            GElement::Layer_(_l) => {
                unimplemented!();
            }
            GElement::Builder_(gel, _) => {
                gel.deref().refresh_for(who_checkbox);
            }
            GElement::Text_(t) => {
                who_checkbox.label = t.get_content(); //TODO text.get_content directly return IdStr
            }
            GElement::Button_(_) => {
                unimplemented!();
            }
            GElement::Refresher_(_refresher) => {
                // NOTE this is refresh_for GElement , not Checkbox
                unimplemented!();
            }
            GElement::Event_(_) => {
                todo!();
            }
            GElement::Generic_(_g_self) => {
                trace!("use Generic refresh_for Checkbox");
                //TODO 反射?
                todo!("reflection?");
            }
            GElement::NodeRef_(_) => panic!("GElement::NodeIndex_() should handle before."),
            GElement::EmptyNeverUse => panic!("EmptyNeverUse never here"),
        };
    }
}

// @ 更新who -GElement ------------------------------------
impl<Message> RefreshFor<GElement<Message>> for Checkbox<Message>
where
    Message: 'static + Clone + for<'a> MessageTid<'a> + std::cmp::PartialEq,
{
    fn refresh_for(&self, who: &mut GElement<Message>) {
        match who {
            GElement::Layer_(l) => {
                // unimplemented!();
                l.ref_push(self.clone());
            }
            GElement::Builder_(gel, _) => {
                self.refresh_for(gel.as_mut());
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
    Message: Clone + for<'a> MessageTid<'a> + std::cmp::PartialEq,
{
    fn from(checkbox: Checkbox<Message>) -> Self {
        Self::Generic_(Box::new(checkbox))
    }
}
