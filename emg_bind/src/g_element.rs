/*
 * @Author: Rais
 * @Date: 2021-03-08 16:50:04
 * @LastEditTime: 2021-09-09 17:15:32
 * @LastEditors: Rais
 * @Description:
 */
use crate::{
    emg_runtime::{Button, Element, EventNode, Layer, Text},
    NodeBuilder, NodeBuilderWidget,
};
pub use better_any;
use better_any::{Tid, TidAble};
use emg::NodeIndex;
use emg_core::TypeCheckObjectSafe;
use emg_refresh::{RefreshFor, RefreshUse};
// extern crate derive_more;
use derive_more::From;
use dyn_clonable::clonable;
use std::{convert::TryFrom, rc::Rc};
use strum_macros::Display;
pub trait GenerateElement<'a, Message> {
    fn generate_element(&self) -> Element<'a, Message>;
}

#[allow(clippy::module_name_repetitions)]
#[clonable]
pub trait DynGElement<'a, Message>:
    // AsRefreshFor<GElement<'a, Message>>
    Tid<'a>
     +RefreshFor<GElement<'a, Message>>
     +RefreshUse<GElement<'a,Message>>
    + GenerateElement<'a, Message>
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
// impl<'a, Message, T: RefreshFor<GElement<'a, Message>>> AsRefreshFor<GElement<'a, Message>> for T {
//     fn as_refresh_for(&self) -> &dyn RefreshFor<GElement<'a, Message>> {
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
// impl<'a, Message> AsNodeBuilder<Message> for Box<dyn DynGElement<'a, Message>> {
//     fn as_node_builder(&self) -> &dyn NodeBuilder<Message> {
//         self.as_ref()
//     }
// }

#[derive(Clone, Display, From)]
pub enum GElement<'a, Message>
where
    Message: 'static,
{
    //TODO cow
    Builder_(Box<GElement<'a, Message>>, NodeBuilderWidget<'a, Message>),
    Layer_(Layer<'a, Message>),
    Text_(Text),
    Button_(Button<'a, Message>),
    Refresher_(Rc<dyn RefreshFor<GElement<'a, Message>> + 'a>),
    Event_(EventNode<Message>),
    //internal
    Generic_(Box<dyn DynGElement<'a, Message>>),
    #[from(ignore)]
    NodeRef_(String), // IntoE(Rc<dyn Into<Element<'a, Message>>>),
}

pub fn node_ref<'a, Message>(str: impl Into<String>) -> GElement<'a, Message> {
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

impl<'a, Message: std::clone::Clone + 'static> GElement<'a, Message> {
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

    pub fn as_node_ref_(&self) -> Option<&String> {
        if let Self::NodeRef_(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

impl<'a, Message: std::fmt::Debug + std::clone::Clone> std::fmt::Debug for GElement<'a, Message> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use GElement::{Builder_, Button_, Event_, Generic_, Layer_, Refresher_, Text_};
        let nbw = "NodeBuilderWidget<'a, Message>".to_string();

        match self {
            Layer_(l) => f.debug_tuple("GElement::GContainer").field(l).finish(),
            Text_(t) => f.debug_tuple("GElement::Text").field(t).finish(),
            Refresher_(_) => f
                .debug_tuple("GElement::GUpdater(Rc<dyn RtUpdateFor<GElement<'a, Message>>>)")
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
        }
    }
}

impl<'a, Message> TryFrom<GElement<'a, Message>> for Element<'a, Message>
where
    Message: 'static + Clone,
{
    type Error = ();

    ///  Refresher_(_)|Event_(_) can't to Element
    fn try_from(ge: GElement<'a, Message>) -> Result<Self, Self::Error> {
        use match_any::match_any;
        use GElement::{Builder_, Button_, Event_, Generic_, Layer_, NodeRef_, Refresher_, Text_};

        // if let GElement::Builder_(gel, builder) = ge {
        //     let x = gel.borrow_mut().as_mut();
        //     let ff = Rc::new(f);
        // }

        match_any!(ge,
            Builder_(gel, mut builder) => {

                builder.set_widget(gel);
                Ok(builder.into())
            },
            Layer_(x) | Text_(x) | Button_(x) => Ok(x.into()),
            Refresher_(_) | Event_(_) => Err(()),
            Generic_(x) => Ok(x.generate_element()),
            NodeRef_(_)=> panic!("TryFrom<GElement to Element: \n     GElement::NodeIndex_() should handle before.")



        )
    }
}
