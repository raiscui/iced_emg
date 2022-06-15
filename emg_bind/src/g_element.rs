/*
 * @Author: Rais
 * @Date: 2021-03-08 16:50:04
 * @LastEditTime: 2022-06-15 16:35:46
 * @LastEditors: Rais
 * @Description:
 */
use crate::{
    emg_runtime::{Button,  EventNode, Layer, Text},
    NodeBuilderWidget, Widget,
};
use match_any::match_any;

pub use better_any;
use better_any::{Tid, TidAble};
use dyn_partial_eq::DynPartialEq;
use emg_core::{IdStr, TypeCheckObjectSafe};
use emg_refresh::{EqRefreshFor, RefreshFor, RefreshUse};
// extern crate derive_more;
use derive_more::From;
use dyn_clonable::clonable;
use std::rc::Rc;
use strum_macros::Display;
use tracing::debug;

#[allow(clippy::module_name_repetitions)]
#[clonable]
pub trait DynGElement<Message>:
    // AsRefreshFor<GElement< Message>>
    for<'a> Tid<'a>
     +RefreshFor<GElement< Message>>
     +RefreshUse<GElement<Message>>
    // + GenerateElement<Message>
    + Widget<Message>
    + TypeCheckObjectSafe
    +DynPartialEq
    + Clone
 
{
}
impl<Message> core::cmp::Eq for dyn DynGElement<Message> + '_ {}

impl<Message> core::cmp::PartialEq for dyn DynGElement<Message> + '_ {
    fn eq(&self, other: &Self) -> bool {
        self.box_eq(other.as_any())
    }
}
impl<Message> core::cmp::PartialEq<dyn DynGElement<Message> + '_>
    for Box<dyn DynGElement<Message> + '_>
{
    fn eq(&self, other: &dyn DynGElement<Message>) -> bool {
        self.box_eq(other.as_any())
    }
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

#[derive(Clone, Display, DynPartialEq, From)]
#[eq_opt(no_self_where, where_add = "Message: PartialEq+'static,")]
pub enum GElement<Message> {
    //TODO cow
    Builder_(NodeBuilderWidget<Message>),
    Layer_(Layer<Message>),
    Text_(Text),
    Button_(Button<Message>),
    Refresher_(Rc<dyn EqRefreshFor<Self>>),
    Event_(EventNode<Message>),
    //internal
    Generic_(Box<dyn DynGElement<Message>>), //范型
    #[from(ignore)]
    NodeRef_(IdStr),     // IntoE(Rc<dyn Into<Element< Message>>>),
    EmptyNeverUse,
}
impl<Message> Eq for GElement<Message> where Message: PartialEq {}
impl<Message> PartialEq for GElement<Message>
where
    Message: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Builder_(l0), Self::Builder_(r0)) => l0 == r0,
            (Self::Layer_(l0), Self::Layer_(r0)) => l0 == r0,
            (Self::Text_(l0), Self::Text_(r0)) => l0 == r0,
            (Self::Button_(l0), Self::Button_(r0)) => l0 == r0,
            (Self::Refresher_(l0), Self::Refresher_(r0)) => (**l0) == (**r0),
            (Self::Event_(l0), Self::Event_(r0)) => l0 == r0,
            (Self::Generic_(l0), Self::Generic_(r0)) => l0 == r0,
            (Self::NodeRef_(l0), Self::NodeRef_(r0)) => l0 == r0,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
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

impl<Message> GElement<Message>
where
    Message: PartialEq,
{
    /// Returns `true` if the `g_element` is [`EventCallBack_`].
    #[must_use]
    pub const fn is_event_(&self) -> bool {
        matches!(self, Self::Event_(..))
    }

    /// Returns `true` if the g element is [`NodeIndex_`].
    ///
    /// [`NodeIndex_`]: GElement::NodeIndex_
    pub const fn is_node_ref_(&self) -> bool {
        matches!(self, Self::NodeRef_(..))
    }

    pub const fn as_node_ref_(&self) -> Option<&IdStr> {
        if let Self::NodeRef_(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn as_dyn_node_widget(&self) -> &dyn Widget<Message> where Message: Clone +'static{
        use GElement::{
            Builder_, Button_, EmptyNeverUse, Event_, Generic_, Layer_, NodeRef_, Refresher_, Text_,
        };
        match_any!(self,
            Builder_( builder) => {

                builder  as &dyn Widget<Message>
            },
            Layer_(x) | Text_(x) | Button_(x) => x as &dyn Widget<Message>,
            Refresher_(_) | Event_(_) => panic!("Refresher_|Event_ can't convert to dyn widget."),
            Generic_(x) => {
                debug!("Generic_:: from Generic_ to dyn Widget");
                 &**x as &dyn Widget<Message>},
            NodeRef_(_)=> panic!("TryFrom<GElement to dyn Widget: \n     GElement::NodeIndex_() should handle before."),
            EmptyNeverUse=> panic!("EmptyNeverUse never here")



        )
    }

    // pub fn into_dyn_node_widget(self) -> Result<Box<dyn Widget<Message>>, String> {
    //     use GElement::{
    //         Builder_, Button_, EmptyNeverUse, Event_, Generic_, Layer_, NodeRef_, Refresher_, Text_,
    //     };
    //     match_any!(self,
    //         Builder_(gel, mut builder) => {

    //             builder.and_widget(*gel);
    //             Ok(Box::new(builder))
    //         },
    //         Layer_(x) | Text_(x) | Button_(x) => Ok(Box::new(x) as Box<dyn Widget<Message>>),
    //         Refresher_(_) | Event_(_) => Err("Refresher_|Event_ can't convert to dyn widget.".to_string()),
    //         Generic_(x) => {
    //             debug!("Generic_:: from Generic_ to element");
    //             Ok( x as Box<dyn Widget<Message>>)},
    //         NodeRef_(_)=> panic!("TryFrom<GElement to Element: \n     GElement::NodeIndex_() should handle before."),
    //         EmptyNeverUse=> panic!("EmptyNeverUse never here")

    //     )
    // }
}

impl<Message> std::fmt::Debug for GElement<Message>
where
    Message: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use GElement::{
            Builder_, Button_, EmptyNeverUse, Event_, Generic_, Layer_, NodeRef_, Refresher_, Text_,
        };
        let nbw = "NodeBuilderWidget< Message>(empty Widget)".to_string();

        match self {
            Layer_(l) => f.debug_tuple("GElement::Layer").field(l).finish(),
            Text_(t) => f.debug_tuple("GElement::Text").field(t).finish(),
            Refresher_(_) => f
                .debug_tuple("GElement::GUpdater(Rc<dyn RtUpdateFor<GElement< Message>>>)")
                .finish(),
            Builder_(builder) => {
                if let Some(gel) = builder.widget() {
                    f.debug_tuple("GElement::Builder_").field(gel).finish()
                } else {
                    f.debug_tuple("GElement::Builder_").field(&nbw).finish()
                }
            }
            Event_(e) => f.debug_tuple("GElement::EventCallBack_").field(e).finish(),
            Button_(_) => {
                write!(f, "GElement::Button_")
            }
            Generic_(_) => write!(f, "GElement::Generic_"),
            NodeRef_(nid) => {
                write!(f, "GElement::NodeIndex(\"{}\")", nid)
            }
            EmptyNeverUse => write!(f, "GElement::EmptyNeverUse"),
        }
    }
}
