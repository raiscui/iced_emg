/*
 * @Author: Rais
 * @Date: 2021-03-08 16:50:04
 * @LastEditTime: 2022-01-18 16:23:51
 * @LastEditors: Rais
 * @Description:
 */
use crate::{
    emg_runtime::{Button, Element, EventNode, Layer, Text},
    NodeBuilder, NodeBuilderWidget,
};
pub use better_any;
use better_any::{Tid, TidAble};
use emg_core::{IdStr, TypeCheckObjectSafe};
use emg_refresh::{RefreshFor, RefreshUse};
// extern crate derive_more;
use derive_more::From;
use dyn_clonable::clonable;
use std::{cell::RefCell, convert::TryFrom, rc::Rc};
use strum_macros::Display;
use tracing::debug;
pub trait GenerateElement<Message> {
    fn generate_element(&self) -> Element<Message>;
}

#[allow(clippy::module_name_repetitions)]
#[clonable]
pub trait DynGElement<Message>:
    // AsRefreshFor<GElement< Message>>
    for<'a> Tid<'a>
     +RefreshFor<GElement< Message>>
     +RefreshUse<GElement<Message>>
    + GenerateElement<Message>
    + NodeBuilder<Message>
    + TypeCheckObjectSafe
    + Clone
    where Message: 'static
{
}
pub trait MessageTid<'a>: TidAble<'a> {}
// pub trait AsRefreshFor<T> {
//     fn as_refresh_for(&self) -> &dyn RefreshFor<T>;
// }
// impl< Message, T: RefreshFor<GElement< Message>>> AsRefreshFor<GElement< Message>> for T {
//     fn as_refresh_for(&self) -> &dyn RefreshFor<GElement< Message>> {
//         self
//     }
// }

// pub trait AsNodeBuilder<T> {
//     fn as_node_builder(&self) -> &dyn NodeBuilder<T>;
// }
// // impl<Message, T: NodeBuilder<Message>> AsNodeBuilder<Message> for T {
// //     fn as_node_builder(&self) -> &dyn NodeBuilder<Message> {
// //         self
// //     }
// // }
// impl< Message> AsNodeBuilder<Message> for Box<dyn DynGElement< Message>> {
//     fn as_node_builder(&self) -> &dyn NodeBuilder<Message> {
//         self.as_ref()
//     }
// }

#[derive(Clone, Display, From)]
pub enum GElement<Message>
where
    Message: 'static,
{
    //TODO cow
    Builder_(Rc<RefCell<Self>>, NodeBuilderWidget<Message>),
    Layer_(Layer<Message>),
    Text_(Text),
    Button_(Button<Message>),
    Refresher_(Rc<dyn RefreshFor<Self>>),
    Event_(EventNode<Message>),
    //internal
    Generic_(Box<dyn DynGElement<Message>>), //范型
    #[from(ignore)]
    NodeRef_(IdStr),     // IntoE(Rc<dyn Into<Element< Message>>>),
    EmptyNeverUse,
}

pub fn node_ref<Message>(str: impl Into<IdStr>) -> GElement<Message> {
    GElement::NodeRef_(str.into())
}

// fn replace_with<X, F: Fn(X) -> X>(x: &mut X, convert: F)
// where
//     X: Default,
// {
//     let old = std::mem::take(x);
//     *x = convert(old);
// }
// fn replace_with_result<X, F: Fn(X) -> Result<X, ()>>(x: &mut X, convert: F) -> Result<&mut X, ()>
// where
//     X: Default,
// {
//     let old = std::mem::take(x);
//     convert(old).map(|new| {
//         *x = new;
//         x
//     })
// }

impl<Message: std::clone::Clone + 'static> GElement<Message> {
    /// Returns `true` if the `g_element` is [`EventCallBack_`].
    #[must_use]
    pub fn is_event_(&self) -> bool {
        matches!(self, Self::Event_(..))
    }

    /// Returns `true` if the g element is [`NodeIndex_`].
    ///
    /// [`NodeIndex_`]: GElement::NodeIndex_
    pub fn is_node_ref_(&self) -> bool {
        matches!(self, Self::NodeRef_(..))
    }

    pub fn as_node_ref_(&self) -> Option<&IdStr> {
        if let Self::NodeRef_(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

impl<Message: std::fmt::Debug + std::clone::Clone> std::fmt::Debug for GElement<Message> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use GElement::{Builder_, Button_, Event_, Generic_, Layer_, Refresher_, Text_};
        let nbw = "NodeBuilderWidget< Message>".to_string();

        match self {
            Layer_(l) => f.debug_tuple("GElement::GContainer").field(l).finish(),
            Text_(t) => f.debug_tuple("GElement::Text").field(t).finish(),
            Refresher_(_) => f
                .debug_tuple("GElement::GUpdater(Rc<dyn RtUpdateFor<GElement< Message>>>)")
                .finish(),
            Builder_(ge, _) => f
                .debug_tuple("GElement::Builder_")
                .field(&ge)
                .field(&nbw)
                .finish(),
            Event_(e) => f.debug_tuple("GElement::EventCallBack_").field(&e).finish(),
            Button_(_) => {
                write!(f, "GElement::Button_")
            }
            Generic_(_) => write!(f, "GElement::Generic_"),
            GElement::NodeRef_(nid) => {
                write!(f, "GElement::NodeIndex({})", nid)
            }
            GElement::EmptyNeverUse => write!(f, "GElement::EmptyNeverUse"),
        }
    }
}

impl<Message> TryFrom<GElement<Message>> for Element<Message>
where
    Message: 'static + Clone,
{
    type Error = ();

    ///  Refresher_(_)|Event_(_) can't to Element
    fn try_from(ge: GElement<Message>) -> Result<Self, Self::Error> {
        use match_any::match_any;
        use GElement::{
            Builder_, Button_, EmptyNeverUse, Event_, Generic_, Layer_, NodeRef_, Refresher_, Text_,
        };

        // if let GElement::Builder_(gel, builder) = ge {
        //     let x = gel.borrow_mut().as_mut();
        //     let ff = Rc::new(f);
        // }

        match_any!(ge,
            Builder_(ref gel, mut builder) => {

                builder.set_widget(gel);
                Ok(builder.into())
            },
            Layer_(x) | Text_(x) | Button_(x) => Ok(x.into()),
            Refresher_(_) | Event_(_) => Err(()),
            Generic_(x) => {
                debug!("Generic_:: from Generic_ to element");
                Ok(x.generate_element())},
            NodeRef_(_)=> panic!("TryFrom<GElement to Element: \n     GElement::NodeIndex_() should handle before."),
            EmptyNeverUse=> panic!("EmptyNeverUse never here")



        )
    }
}
