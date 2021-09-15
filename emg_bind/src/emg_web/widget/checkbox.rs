/*
 * @Author: Rais
 * @Date: 2021-09-01 09:58:44
 * @LastEditTime: 2021-09-15 15:37:18
 * @LastEditors: Rais
 * @Description:
 */
//! Show toggle controls using checkboxes.
use crate::{
    emg_runtime::{Bus, Element, Widget},
    iced_runtime::{css, Length},
    DynGElement, GElement, GenerateElement, MessageTid, NodeBuilder,
};

use better_any::{impl_tid, tid, type_id, Tid, TidAble, TidExt};

use emg_core::{TypeCheckObjectSafe, TypeName};
use emg_refresh::RefreshFor;
pub use iced_style::checkbox::{Style, StyleSheet};
use seed_styles::GlobalStyleSV;
use tracing::{error, trace, warn};

use crate::emg_runtime::dodrio::bumpalo;
use std::{
    ops::{Deref, DerefMut},
    rc::Rc,
};

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
#[derive(Clone, Tid)]
pub struct Checkbox<Message> {
    is_checked: bool,
    //FIXME use cow for Rc 防止 克隆对象和 原始对象使用同一个 callback
    on_toggle: Rc<dyn Fn(bool) -> Message>,
    label: String,
    id: Option<String>,
    width: Length,
    #[allow(dead_code)]
    style: Box<dyn StyleSheet>,
}

impl<Message> Checkbox<Message> {
    /// Creates a new [`Checkbox`].
    ///
    /// It expects:
    ///   * a boolean describing whether the [`Checkbox`] is checked or not
    ///   * the label of the [`Checkbox`]
    ///   * a function that will be called when the [`Checkbox`] is toggled. It
    ///     will receive the new state of the [`Checkbox`] and must produce a
    ///     `Message`.
    pub fn new<F>(is_checked: bool, label: impl Into<String>, f: F) -> Self
    where
        F: 'static + Fn(bool) -> Message,
    {
        Self {
            is_checked,
            on_toggle: Rc::new(f),
            label: label.into(),
            id: None,
            width: Length::Shrink,
            style: std::boxed::Box::default(),
        }
    }

    /// Sets the width of the [`Checkbox`].
    #[must_use]
    pub const fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the style of the [`Checkbox`].
    pub fn style(mut self, style: impl Into<Box<dyn StyleSheet>>) -> Self {
        self.style = style.into();
        self
    }

    /// Sets the id of the [`Checkbox`].
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }
}

impl<Message> NodeBuilder<Message> for Checkbox<Message>
where
    Message: 'static + Clone,
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
    Message: 'static + Clone,
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

impl<'a, Message> From<Checkbox<Message>> for Element<'a, Message>
where
    Message: 'static + Clone,
{
    fn from(checkbox: Checkbox<Message>) -> Element<'a, Message> {
        Element::new(checkbox)
    }
}

impl<'a, Message> GenerateElement<'a, Message> for Checkbox<Message>
where
    Message: 'static + Clone,
{
    fn generate_element(&self) -> Element<'a, Message> {
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
impl<'a, Message> RefreshFor<Checkbox<Message>> for GElement<'a, Message>
where
    Message: 'static + Clone + MessageTid<'a>,
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
                who_checkbox.label = t.get_content();
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
        };
    }
}

// @ 更新who -GElement ------------------------------------
impl<'a, Message> RefreshFor<GElement<'a, Message>> for Checkbox<Message>
where
    Message: 'static + Clone + MessageTid<'a>,
{
    fn refresh_for(&self, who: &mut GElement<'a, Message>) {
        match who {
            GElement::Layer_(l) => {
                // unimplemented!();
                l.ref_push(self.clone());
            }
            GElement::Builder_(gel, _) => {
                let mut_gel = gel.deref_mut();
                self.refresh_for(mut_gel);
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
        };
    }
}
impl<Message> TypeCheckObjectSafe for Checkbox<Message> {
    fn type_name(&self) -> TypeName {
        TypeName::new("Checkbox")
    }
}

impl<'a, Message> DynGElement<'a, Message> for Checkbox<Message> where
    Message: Clone + 'static + MessageTid<'a>
{
}

impl<'a, Message> From<Checkbox<Message>> for GElement<'a, Message>
where
    Message: Clone + MessageTid<'a>,
{
    fn from(checkbox: Checkbox<Message>) -> Self {
        GElement::Generic_(Box::new(checkbox))
    }
}
