/*
 * @Author: Rais
 * @Date: 2021-03-08 16:50:04
 * @LastEditTime: 2021-03-11 14:39:55
 * @LastEditors: Rais
 * @Description:
 */

use crate::{
    runtime::{Element, Text},
    EventCallbackType, Layer, NodeBuilderWidget, RefreshFor,
};
use std::{convert::TryFrom, rc::Rc};
use strum_macros::Display;

pub use GElement::*;

#[derive(Clone, Display)]
pub enum GElement<'a, Message> {
    NodeBuilderWidget_(NodeBuilderWidget<'a, Message>),
    Element_(Element<'a, Message>),
    Layer_(Layer<'a, Message>),
    Text_(Text),
    Refresher_(Rc<dyn RefreshFor<GElement<'a, Message>> + 'a>),
    EventCallBack_(EventCallbackType),
    _Empty,
}

impl<'a, Message> Default for GElement<'a, Message> {
    fn default() -> Self {
        Self::_Empty
    }
}

impl<'a, Message> From<NodeBuilderWidget<'a, Message>> for GElement<'a, Message> {
    fn from(v: NodeBuilderWidget<'a, Message>) -> Self {
        Self::NodeBuilderWidget_(v)
    }
}

impl<'a, Message> From<EventCallbackType> for GElement<'a, Message> {
    fn from(v: EventCallbackType) -> Self {
        Self::EventCallBack_(v)
    }
}

// fn replace_with<X, F: Fn(X) -> X>(x: &mut X, convert: F)
// where
//     X: Default,
// {
//     let old = std::mem::take(x);
//     *x = convert(old);
// }
fn replace_with_result<X, F: Fn(X) -> Result<X, ()>>(x: &mut X, convert: F) -> Result<&mut X, ()>
where
    X: Default,
{
    let old = std::mem::take(x);
    convert(old).map(|new| {
        *x = new;
        x
    })
}

impl<'a, Message: std::clone::Clone + 'static> GElement<'a, Message> {
    /// Returns `true` if the `g_element` is [`EventCallBack_`].
    #[must_use]
    pub fn is_event_call_back_(&self) -> bool {
        matches!(self, Self::EventCallBack_(..))
    }

    fn convert_inside_to_node_builder_widget_(self) -> Result<Self, ()> {
        use match_any::match_any;
        match_any! (self,
            NodeBuilderWidget_(_)=>Ok(self),
            Layer_( x)=> {
                Ok(NodeBuilderWidget_(NodeBuilderWidget::new(Rc::new(x))))
            },
            Element_(_) |Text_(_)|Refresher_(_)|EventCallBack_(_)=>Err(()),
            _Empty=>Err(())
        )
    }

    /// # Errors
    ///
    /// Will return `Err` if `GElement` does impl `NodeBuilder` trait,it can't convert to `NodeBuilder`.
    // TODO use Error type
    #[allow(clippy::result_unit_err)]
    pub fn try_convert_inside_to_node_builder_widget_(&mut self) -> Result<&mut Self, ()> {
        replace_with_result(self, Self::convert_inside_to_node_builder_widget_)
    }
}

impl<'a, Message: std::fmt::Debug> std::fmt::Debug for GElement<'a, Message> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Layer_(l) => f.debug_tuple("GElement::GContainer").field(l).finish(),
            Text_(t) => f.debug_tuple("GElement::Text").field(t).finish(),
            Refresher_(_) => f
                .debug_tuple("GElement::GUpdater(Rc<dyn RtUpdateFor<GElement<'a, Message>>>)")
                .finish(),
            Element_(_) => f
                .debug_tuple("GElement::Element_(Element<'a, Message>)")
                .finish(),
            EventCallBack_(_) => f
                .debug_tuple("GElement::EventCallBack_")
                .field(&"Box<dyn EventCallbackClone>")
                .finish(),
            NodeBuilderWidget_(nb_widget) => f
                .debug_tuple("GElement::NodeBuilderWidget_(NodeBuilderWidget<'a, Message>)")
                .field(nb_widget)
                .finish(),
            _Empty => {
                write!(f, "GElement::Empty")
            }
        }
    }
}

impl<'a, Message> TryFrom<GElement<'a, Message>> for Element<'a, Message>
where
    Message: 'static + Clone,
{
    type Error = ();

    // #[allow(clippy::useless_conversion)]
    fn try_from(ge: GElement<'a, Message>) -> Result<Self, Self::Error> {
        use match_any::match_any;

        match_any! (ge,
            Element_(x)=>Ok(x),
            Layer_(x)|Text_(x)|NodeBuilderWidget_(x) => Ok(x.into()),
            Refresher_(_)|EventCallBack_(_)=>Err(()),
            _Empty=>Err(())
        )
    }
}
