/*
 * @Author: Rais
 * @Date: 2021-03-08 16:50:04
 * @LastEditTime: 2022-09-10 15:30:39
 * @LastEditors: Rais
 * @Description:
 */
// pub mod impls;
use crate::{
    widget::{Button, Layer, Text, Widget},
    EventNode, NodeBuilderWidget,
};
use dyn_clone::DynClone;
use emg_state::{StateAnchor, StateMultiAnchor};
use match_any::match_any;

use emg_common::{better_any::Tid, dyn_partial_eq::DynPartialEq, IdStr, TypeCheckObjectSafe};
use emg_shaping::{EqShaping, Shaping, ShapingUse, ShapingUseAny};
// extern crate derive_more;
use derive_more::From;
// use dyn_clonable::clonable;
use std::{any::Any, rc::Rc};
use strum::Display;
use tracing::warn;

#[allow(clippy::module_name_repetitions)]
// #[clonable]
pub trait DynGElement<Message>:
    // AsRefreshFor<GElement< Message>>
    for<'a> Tid<'a>
     +Shaping<GElement< Message>>
     +ShapingUse<GElement<Message>>
    // + GenerateElement<Message>
    + Widget<Message>
    + TypeCheckObjectSafe
    + DynPartialEq
    + DynClone
    + core::fmt::Debug
    + ShapingUseAny
    + ShapingUse<i32>
{
}
dyn_clone::clone_trait_object!(<Message> DynGElement<Message>);

impl<Message> core::cmp::Eq for dyn DynGElement<Message> + '_ {}

impl<Message> core::cmp::PartialEq for dyn DynGElement<Message> + '_ {
    fn eq(&self, other: &Self) -> bool {
        self.box_eq(other.as_any())
    }
}
impl<Message: 'static> core::cmp::PartialEq<dyn DynGElement<Message>>
    for Box<dyn DynGElement<Message>>
{
    fn eq(&self, other: &dyn DynGElement<Message>) -> bool {
        self.box_eq(other.as_any())
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use emg_shaping::{EqShaping, Shaper};
    use emg_state::use_state;

    use crate::GElement;

    #[derive(Clone, PartialEq, Eq)]
    enum Message {
        A,
    }

    #[test]
    fn it_works() {
        let _f = GElement::<Message>::Refresher_(
            Rc::new(Shaper::new(|| 1i32)) as Rc<dyn EqShaping<GElement<Message>>>
        );
        let _a = use_state(2i32);

        let _f = GElement::<Message>::Refresher_(Rc::new(_a.watch()));

        // let ff: Rc<dyn EqShaping<GElement<Message>>> = f;
        // Rc<dyn EqShaping<GElement<Message>>>, found Rc<Shaper<u32>>
    }
}

#[derive(Clone, Display, From)]
// #[eq_opt(no_self_where, where_add = "Message: PartialEq+'static,")]
pub enum GElement<Message> {
    //TODO cow
    Builder_(NodeBuilderWidget<Message>),
    Layer_(Layer<Message>),
    Text_(Text),
    Button_(Button<Message>),
    Refresher_(Rc<dyn EqShaping<Self>>),
    Event_(EventNode<Message>),
    //internal
    Generic_(Box<dyn DynGElement<Message>>), //范型 //TODO check batter when use rc?
    #[from(ignore)]
    NodeRef_(IdStr),      // IntoE(Rc<dyn Into<Element< Message>>>),
    // #[from(ignore)]
    // InsideDirectUseSa_(StateAnchor<Rc<Self>>),//NOTE generate by tree builder use into()
    #[from(ignore)]
    SaNode_(StateAnchor<Rc<Self>>),
    EvolutionaryFactor(Rc<dyn Evolution<StateAnchor<Rc<Self>>>>),
    EmptyNeverUse,
}

pub trait Evolution<Who>: DynPartialEq {
    fn evolution(&self, who: &Who) -> Who;
}

impl<Who> core::cmp::Eq for dyn Evolution<Who> + '_ {}

impl<Who> core::cmp::PartialEq for dyn Evolution<Who> + '_ {
    fn eq(&self, other: &Self) -> bool {
        self.box_eq(other.as_any())
    }
}
impl<Who: 'static> core::cmp::PartialEq<dyn Evolution<Who>> for Box<dyn Evolution<Who>> {
    fn eq(&self, other: &dyn Evolution<Who>) -> bool {
        self.box_eq(other.as_any())
    }
}

type SaMapAction<Message, Use> = Rc<dyn Fn(&Rc<GElement<Message>>, &Use) -> Rc<GElement<Message>>>;

#[derive(Clone)]
pub struct SaWithMapFn<Use: Clone, Message> {
    u_s_e: StateAnchor<Use>,
    map_action: SaMapAction<Message, Use>,
}

impl<Use: Clone, Message> SaWithMapFn<Use, Message> {
    pub fn new(u_s_e: StateAnchor<Use>, map_action: SaMapAction<Message, Use>) -> Self {
        Self { u_s_e, map_action }
    }
}

impl<Use: PartialEq + Clone, Message> PartialEq for SaWithMapFn<Use, Message> {
    fn eq(&self, other: &Self) -> bool {
        self.u_s_e == other.u_s_e
            && std::ptr::eq(
                (std::ptr::addr_of!(*self.map_action)).cast::<u8>(),
                (std::ptr::addr_of!(*other.map_action)).cast::<u8>(),
            )
    }
}

impl<Use, Message> Evolution<StateAnchor<Rc<GElement<Message>>>> for SaWithMapFn<Use, Message>
where
    Use: Clone + PartialEq + 'static,
    Message: PartialEq + Clone + 'static,
{
    fn evolution(
        &self,
        who: &StateAnchor<Rc<GElement<Message>>>,
    ) -> StateAnchor<Rc<GElement<Message>>> {
        let func = self.map_action.clone();
        (who, &self.u_s_e).map(move |gel, u_s_e| func(gel, u_s_e))
    }
}

impl<Use, Message> Evolution<StateAnchor<Rc<GElement<Message>>>> for StateAnchor<Use>
where
    Use: PartialEq + EqShaping<GElement<Message>> + 'static + std::fmt::Debug + Clone,
    Message: PartialEq + Clone + 'static + std::fmt::Debug,
{
    fn evolution(
        &self,
        who: &StateAnchor<Rc<GElement<Message>>>,
    ) -> StateAnchor<Rc<GElement<Message>>> {
        warn!(
            "[Evolution] Evolution<{:?}> use {:?}:{}",
            &who,
            &self,
            std::any::type_name::<Self>()
        );

        (who, self).map(|gel, u_s_e| {
            warn!(
                "[Evolution] in -- Evolution<{:?}> use {:?}:{}",
                &gel,
                &u_s_e,
                std::any::type_name::<Use>()
            );

            let mut new_gel = (**gel).clone();
            new_gel.shaping_use(u_s_e);
            Rc::new(new_gel)
        })
    }
}

impl<Use, Message> From<SaWithMapFn<Use, Message>> for GElement<Message>
where
    Use: PartialEq + Clone + 'static,
    Message: Clone + PartialEq + 'static,
    //TODO check StateAnchor<Use>: Evolution<StateAnchor<Rc<Self>>>  or "SaWithMapFn<Use, Message>: Evolution<StateAnchor<Rc<Self>>>"
    SaWithMapFn<Use, Message>: Evolution<StateAnchor<Rc<Self>>>,
    // StateAnchor<Use>: Evolution<StateAnchor<Rc<Self>>>,
{
    fn from(sa_with_fn: SaWithMapFn<Use, Message>) -> Self {
        Self::EvolutionaryFactor(Rc::new(sa_with_fn))
    }
}
impl<Use, Message> From<StateAnchor<Use>> for GElement<Message>
where
    Use: 'static,
    Message: 'static,
    StateAnchor<Use>: Evolution<StateAnchor<Rc<Self>>>,
{
    fn from(sa_use: StateAnchor<Use>) -> Self {
        (&sa_use as &dyn Any)
            //TODO is should check can downcast_ref to StateAnchor<Self>?
            .downcast_ref::<StateAnchor<Rc<Self>>>()
            .cloned()
            .map_or_else(
                || Self::EvolutionaryFactor(Rc::new(sa_use)),
                |s| Self::SaNode_(s), //NOTE: is StateAnchor<Rc<Self>>
            )
    }
}

// impl<Message> From<StateAnchor<Rc<Self>>> for GElement<Message>
//     {
//         fn from(sa_use: StateAnchor<Rc<Self>>) -> Self {
//                 Self::InsideDirectUseSa_(sa_use)

//         }
//     }

#[cfg(test)]
mod evolution_test {
    use std::rc::Rc;

    use emg_state::{use_state, StateAnchor};

    use crate::GElement;

    use super::{Evolution, SaWithMapFn};

    #[derive(Clone, Debug, PartialEq, Eq)]
    enum Message {
        A,
    }

    #[test]
    fn test() {
        let a = use_state(1);
        let f = SaWithMapFn {
            u_s_e: a.watch(),
            map_action: Rc::new(|p, num| p.clone()),
        };

        let ge = use_state(GElement::<Message>::EmptyNeverUse).watch();
        let _x = GElement::<Message>::EvolutionaryFactor(Rc::new(f.clone()));
        let _xxx: GElement<Message> = f.into();
        let _x2 = GElement::<Message>::EvolutionaryFactor(
            Rc::new(a.watch()) as Rc<dyn Evolution<StateAnchor<Rc<GElement<Message>>>>>
        );
        let _x2 = GElement::<Message>::EvolutionaryFactor(
            Rc::new(ge) as Rc<dyn Evolution<StateAnchor<Rc<GElement<Message>>>>>
        );
        let _x3: GElement<Message> = a.watch().into();
    }
}

impl<Message> Eq for GElement<Message> where Message: PartialEq {}
impl<Message> PartialEq for GElement<Message>
where
    Message: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        //[] allways check when add GElement number;
        //CHECK allways check when add(modifier) GElement number;
        match (self, other) {
            (Self::Builder_(l0), Self::Builder_(r0)) => l0 == r0,
            (Self::Layer_(l0), Self::Layer_(r0)) => l0 == r0,
            (Self::Text_(l0), Self::Text_(r0)) => l0 == r0,
            (Self::Button_(l0), Self::Button_(r0)) => l0 == r0,
            (Self::Refresher_(l0), Self::Refresher_(r0)) => (**l0) == (**r0),
            (Self::Event_(l0), Self::Event_(r0)) => l0 == r0,
            (Self::Generic_(l0), Self::Generic_(r0)) => l0 == r0,
            (Self::NodeRef_(l0), Self::NodeRef_(r0)) => l0 == r0,

            (Self::SaNode_(l0), Self::SaNode_(r0)) => l0 == r0,
            (Self::EvolutionaryFactor(l0), Self::EvolutionaryFactor(r0)) => l0 == r0,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
            // std::ptr::eq(
            //     (std::ptr::addr_of!(**l0)).cast::<u8>(),
            //     (std::ptr::addr_of!(**r0)).cast::<u8>(),
            // )
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

    pub fn as_dyn_node_widget(&self) -> &dyn Widget<Message>
    where
        Message: Clone + 'static,
    {
        use GElement::{
            Builder_, Button_, EmptyNeverUse, Event_, EvolutionaryFactor, Generic_, Layer_,
            NodeRef_, Refresher_, SaNode_, Text_,
        };
        match_any!(self,

            Builder_( x)| Layer_(x) | Text_(x) | Button_(x) => x as &dyn Widget<Message>,
            Refresher_(_) | Event_(_) => panic!("Refresher_|Event_ can't convert to dyn widget."),
            Generic_(x) => {
                // debug!("Generic_:: from Generic_ to dyn Widget");
                 &**x as &dyn Widget<Message>
                // panic!("Generic_ should be Builder here");
                },
            NodeRef_(_)=> panic!("TryFrom<GElement to dyn Widget: \n     GElement::NodeIndex_() should handle before."),
            SaNode_(_)=>todo!(),
            EmptyNeverUse=> panic!("EmptyNeverUse never here"),
            EvolutionaryFactor(_)=> todo!()



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

    pub fn as_generic(&self) -> Option<&dyn DynGElement<Message>> {
        if let Self::Generic_(v) = self {
            Some(v.as_ref())
        } else {
            None
        }
    }

    pub const fn as_text(&self) -> Option<&Text> {
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
            Generic_(x) => f.debug_tuple("GElement::Generic_").field(&x).finish(),
            NodeRef_(nid) => {
                write!(f, "GElement::NodeIndex(\"{}\")", nid)
            }
            EmptyNeverUse => write!(f, "GElement::EmptyNeverUse"),
            Self::SaNode_(_) => write!(f, "GElement::SaNode"),
            Self::EvolutionaryFactor(_) => write!(f, "GElement::EvolutionaryFactor"),
        }
    }
}
