/*
 * @Author: Rais
 * @Date: 2021-09-01 09:14:26
 * @LastEditTime: 2022-08-11 16:37:14
 * @LastEditors: Rais
 * @Description:
 */
use seed_styles::GlobalStyleSV;
use std::rc::Rc;

use crate::emg_runtime::{Bus, Widget};

use crate::emg_runtime::dodrio::bumpalo;

/// A generic [`Widget`].
///
/// It is useful to build composable user interfaces that do not leak
/// implementation details in their __view logic__.
///
/// If you have a [built-in widget], you should be able to use `Into<Element>`
/// to turn it into an [`Element`].
///
/// [built-in widget]: mod@crate::widget
#[allow(missing_debug_implementations)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(PartialEq, Eq)]
pub struct Element<Message> {
    pub(crate) widget: Box<dyn Widget<Message>>,
}

impl<Message> Clone for Element<Message> {
    fn clone(&self) -> Self {
        Self {
            widget: self.widget.clone(),
        }
    }
}

impl<Message> std::fmt::Debug for Element<Message> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[Element]")
    }
}

impl<Message> Element<Message> {
    /// Create a new [`Element`] containing the given [`Widget`].
    pub fn new(widget: impl Widget<Message> + 'static) -> Self {
        Self {
            widget: Box::new(widget),
        }
    }

    /// Applies a transformation to the produced message of the [`Element`].
    ///
    /// This method is useful when you want to decouple different parts of your
    /// UI and make them __composable__.
    pub fn map<F, B>(self, f: F) -> Element<B>
    where
        Message: 'static + Clone,
        B: 'static + Clone,
        F: 'static + Fn(Message) -> B,
    {
        Element {
            widget: Box::new(Map::new(self.widget, f)),
        }
    }

    /// Marks the [`Element`] as _to-be-explained_.
    #[must_use]
    pub const fn explain(self, _color: Color) -> Self {
        self
    }

    /// Produces a VDOM node for the [`Element`].
    pub fn node<'b>(
        &self,
        bump: &'b bumpalo::Bump,
        bus: &Bus<Message>,
        style_sheet: &GlobalStyleSV,
    ) -> dodrio::Node<'b> {
        self.widget.node(bump, bus, style_sheet)
    }
}

#[derive(Clone)]
struct Map<A, B> {
    widget: Box<dyn Widget<A>>,
    mapper: Rc<dyn Fn(A) -> B>,
}
impl<A, B> PartialEq for Map<A, B> {
    fn eq(&self, other: &Self) -> bool {
        self.widget == other.widget
            && std::ptr::eq(
                (std::ptr::addr_of!(*self.mapper)).cast::<u8>(),
                (std::ptr::addr_of!(*other.mapper)).cast::<u8>(),
            )
    }
}
// impl<A, B> DynPartialEq for Map<A, B>
// where
//     A: 'static,
//     B: 'static,
// {
//     fn as_any(&self) -> &dyn core::any::Any {
//         self
//     }
//     fn box_eq(&self, other: &dyn core::any::Any) -> bool {
//         other.downcast_ref::<Self>().map_or(false, |a| self == a)
//     }
// }

impl<A, B> Map<A, B> {
    pub fn new<F>(widget: Box<dyn Widget<A>>, mapper: F) -> Self
    where
        F: 'static + Fn(A) -> B,
    {
        Self {
            widget,
            mapper: Rc::new(mapper),
        }
    }
}

impl<A, B> Widget<B> for Map<A, B>
where
    A: 'static + Clone,
    B: 'static + Clone,
{
    fn node<'b>(
        &self,
        bump: &'b bumpalo::Bump,
        bus: &Bus<B>,
        style_sheet: &GlobalStyleSV,
    ) -> dodrio::Node<'b> {
        self.widget
            .node(bump, &bus.map(self.mapper.clone()), style_sheet)
    }
}
