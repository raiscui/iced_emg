/*
 * @Author: Rais
 * @Date: 2021-03-08 16:50:04
 * @LastEditTime: 2022-06-20 13:22:11
 * @LastEditors: Rais
 * @Description:
 */
// pub mod impls;
use crate::{
    emg_runtime::{Button,  EventNode, Layer, Text},
    NodeBuilderWidget, Widget,
};
use emg_state::{StateAnchor};
use match_any::match_any;

pub use better_any;
use better_any::{Tid, TidAble, TidExt};
use emg_core::{IdStr, TypeCheckObjectSafe, dyn_partial_eq::DynPartialEq};
use emg_refresh::{ RefreshFor, RefreshUse, EqRefreshFor};
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
    + DynPartialEq
    + Clone
 
{
}
impl<Message> core::cmp::Eq for dyn DynGElement<Message> + '_ {}

impl<Message> core::cmp::PartialEq for dyn DynGElement<Message> + '_ {
    fn eq(&self, other: &Self) -> bool {
        self.box_eq(other.as_any())
    }
}
impl<Message:'static> core::cmp::PartialEq<dyn DynGElement<Message> >
    for Box<dyn DynGElement<Message> >
{
    fn eq(&self, other: &dyn DynGElement<Message>) -> bool {
        self.box_eq(other.as_any())
    }
}
pub trait MessageTid<'a>: TidAble<'a> {}



#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use emg_refresh::{Refresher, EqRefreshFor};
    use emg_state::use_state;

    use crate::{GElement};

    #[derive(Clone,PartialEq,Eq)]
    enum Message {
        A,
    }

    #[test]
    fn it_works() {
        let _f = GElement::<Message>::Refresher_(Rc::new(Refresher::new(|| 1i32)) as Rc<dyn EqRefreshFor<GElement<Message>> >);
        let _a = use_state(2i32);

        let _f = GElement::<Message>::Refresher_(Rc::new(_a.watch()) );

        // let ff: Rc<dyn EqRefreshFor<GElement<Message>>> = f;
        // Rc<dyn EqRefreshFor<GElement<Message>>>, found Rc<Refresher<u32>>
    }
}




#[derive(Clone,Display, From)]
// #[eq_opt(no_self_where, where_add = "Message: PartialEq+'static,")]
pub enum GElement<Message> {
    //TODO cow
    Builder_(NodeBuilderWidget<Message>),
    Layer_(Layer<Message>),
    Text_(Text),
    Button_(Button<Message>),
    Refresher_(Rc<dyn EqRefreshFor<Self> >),
    Event_(EventNode<Message>),
    //internal
    Generic_(Box<dyn DynGElement<Message>>), //范型 //TODO check batter when use rc?
    #[from(ignore)]
    NodeRef_(IdStr),     // IntoE(Rc<dyn Into<Element< Message>>>),
    InsideDirectUseSa_(StateAnchor<Rc<Self>>),//NOTE generate by tree builder use into()
    #[from(ignore)]
    SaNode_(StateAnchor<Rc<Self>>),
    EmptyNeverUse,
}
impl<Message> Eq for GElement<Message> where Message: PartialEq {}
impl<Message> PartialEq for GElement<Message>
where
    Message: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        //TODO allways check when add GElement number;
        match (self, other) {
            (Self::Builder_(l0), Self::Builder_(r0)) => l0 == r0,
            (Self::Layer_(l0), Self::Layer_(r0)) => l0 == r0,
            (Self::Text_(l0), Self::Text_(r0)) => l0 == r0,
            (Self::Button_(l0), Self::Button_(r0)) => l0 == r0,
            (Self::Refresher_(l0), Self::Refresher_(r0)) => (**l0) == (**r0) ,
            (Self::Event_(l0), Self::Event_(r0)) => l0 == r0,
            (Self::Generic_(l0), Self::Generic_(r0)) => l0 == r0,
            (Self::NodeRef_(l0), Self::NodeRef_(r0)) => l0 == r0,
            (Self::InsideDirectUseSa_(l0), Self::InsideDirectUseSa_(r0)) => {
                // std::ptr::eq(
                //     (std::ptr::addr_of!(**l0)).cast::<u8>(),
                //     (std::ptr::addr_of!(**r0)).cast::<u8>(),
                // )

                l0 == r0
            },
            (Self::SaNode_(l0),Self::SaNode_(r0)) => l0 == r0,
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
            Builder_, Button_, EmptyNeverUse, Event_, Generic_, Layer_, NodeRef_, Refresher_, Text_,InsideDirectUseSa_,SaNode_
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
            InsideDirectUseSa_(_)=> unreachable!(),
            SaNode_(_)=>todo!(),
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

    /// Returns `true` if the gelement is [`React_`].
    ///
    /// [`React_`]: GElement::React_
  

    /// Returns `true` if the gelement is [`InsideUseSa_`].
    ///
    /// [`InsideUseSa_`]: GElement::InsideUseSa_
    #[must_use]
    pub const fn is_inside_direct_use_sa(&self) -> bool {
        matches!(self, Self::InsideDirectUseSa_(..))
    }

    /// # Errors
    ///
    /// Will return `Err` if `GElement<Message>` is not `InsideDirectUseSa_`
    /// permission to read it.
    #[allow(clippy::missing_const_for_fn)]
    pub fn try_into_inside_direct_use_sa(self) -> Result<StateAnchor<Rc<Self>>, Self> {
        if let Self::InsideDirectUseSa_(v) = self {
            Ok(v)
        } else {
            Err(self)
        }
    }

    pub const fn as_inside_direct_use_sa(&self) -> Option<&StateAnchor<Rc<Self>>> {
        if let Self::InsideDirectUseSa_(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn as_generic(&self) -> Option<&dyn DynGElement<Message>> {
        if let Self::Generic_( v) = self {
            Some(v.as_ref())
        } else {
            None
        }
    }

    pub fn as_text(&self) -> Option<&Text> {
        if let Self::Text_(v) = self {
            Some(v)
        } else {
            None
        }
    }
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
            Generic_(x) => {
                write!(f, "GElement::Generic_")
            },
            NodeRef_(nid) => {
                write!(f, "GElement::NodeIndex(\"{}\")", nid)
            }
            EmptyNeverUse => write!(f, "GElement::EmptyNeverUse"),
            Self::InsideDirectUseSa_(_) => write!(f, "GElement::InsideDirectUseSa_"),
            Self::SaNode_(_)=> write!(f, "GElement::SaNode"),
        }
    }
}
