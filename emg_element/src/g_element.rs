/*
 * @Author: Rais
 * @Date: 2022-08-18 10:47:07
 * @LastEditTime: 2022-08-23 00:28:45
 * @LastEditors: Rais
 * @Description:
 */

// pub mod impls;
use crate::{
    widget::{Layer, Widget},
    NodeBuilderWidget,
};
use dyn_clone::DynClone;
use emg_native::Bus;
use emg_state::{StateAnchor, StateMultiAnchor};
use match_any::match_any;

use emg_common::{better_any::Tid, dyn_partial_eq::DynPartialEq, IdStr, TypeCheckObjectSafe};
use emg_refresh::{EqRefreshFor, RefreshFor, RefreshUse, TryRefreshUse};
// extern crate derive_more;
use derive_more::From;
// use dyn_clonable::clonable;
use std::{any::Any, rc::Rc};
use strum::Display;
use tracing::{instrument, warn};

#[allow(clippy::module_name_repetitions)]
// #[clonable]
pub trait DynGElement<Message,RenderContext>:
    // AsRefreshFor<GElement< Message>>
    for<'a> Tid<'a>
     +RefreshFor<GElement< Message,RenderContext>>
     +RefreshUse<GElement<Message,RenderContext>>
    // + GenerateElement<Message>
    + Widget<Message,RenderContext>
    + TypeCheckObjectSafe
    + DynPartialEq
    + DynClone
    + core::fmt::Debug
    + TryRefreshUse
    + RefreshUse<i32>
    where
    RenderContext: crate::RenderContext ,
{
}
dyn_clone::clone_trait_object!(<Message,RenderContext> DynGElement<Message,RenderContext>);

impl<Message, RenderContext> core::cmp::Eq for dyn DynGElement<Message, RenderContext> + '_ {}

impl<Message, RenderContext> core::cmp::PartialEq for dyn DynGElement<Message, RenderContext> + '_ {
    fn eq(&self, other: &Self) -> bool {
        self.box_eq(other.as_any())
    }
}
impl<Message, RenderContext> core::cmp::PartialEq<dyn DynGElement<Message, RenderContext>>
    for Box<dyn DynGElement<Message, RenderContext>>
where
    Message: 'static,
    RenderContext: 'static,
{
    fn eq(&self, other: &dyn DynGElement<Message, RenderContext>) -> bool {
        self.box_eq(other.as_any())
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use emg_piet_gpu::RenderCtx;
    use emg_refresh::{EqRefreshFor, Refresher};
    use emg_state::use_state;

    use crate::GElement;

    #[derive(Clone, PartialEq, Eq)]
    enum Message {
        A,
    }

    #[test]
    fn it_works() {
        let _f =
            GElement::<Message, RenderCtx>::Refresher_(Rc::new(Refresher::new(|| 1i32))
                as Rc<dyn EqRefreshFor<GElement<Message, RenderCtx>>>);
        let _a = use_state(2i32);

        let _f = GElement::<Message, RenderCtx>::Refresher_(Rc::new(_a.watch()));

        // let ff: Rc<dyn EqRefreshFor<GElement<Message>>> = f;
        // Rc<dyn EqRefreshFor<GElement<Message>>>, found Rc<Refresher<u32>>
    }
}

#[derive(Display, From)]
// #[eq_opt(no_self_where, where_add = "Message: PartialEq+'static,")]
pub enum GElement<Message, RenderContext> {
    // TODO cow
    //NOTE can render element
    Builder_(NodeBuilderWidget<Message, RenderContext>),
    Layer_(Layer<Message, RenderContext>),
    // Text_(Text),
    // Button_(Button<Message>),
    Refresher_(Rc<dyn EqRefreshFor<Self>>),
    //NOTE temp comment
    // Event_(EventNode<Message>),
    //NOTE internal
    Generic_(Box<dyn DynGElement<Message, RenderContext>>), //范型 //TODO check batter when use rc?
    #[from(ignore)]
    NodeRef_(IdStr),                    // IntoE(Rc<dyn Into<Element< Message>>>),
    // #[from(ignore)]
    // InsideDirectUseSa_(StateAnchor<Rc<Self>>),//NOTE generate by tree builder use into()
    #[from(ignore)]
    SaNode_(StateAnchor<Rc<Self>>),
    EvolutionaryFactor(Rc<dyn Evolution<StateAnchor<Rc<Self>>>>),
    EmptyNeverUse,
}

impl<Message, RenderContext> Clone for GElement<Message, RenderContext> {
    fn clone(&self) -> Self {
        match self {
            Self::Builder_(arg0) => Self::Builder_(arg0.clone()),
            Self::Layer_(arg0) => Self::Layer_(arg0.clone()),
            Self::Refresher_(arg0) => Self::Refresher_(arg0.clone()),
            Self::Generic_(arg0) => Self::Generic_(arg0.clone()),
            Self::NodeRef_(arg0) => Self::NodeRef_(arg0.clone()),
            Self::SaNode_(arg0) => Self::SaNode_(arg0.clone()),
            Self::EvolutionaryFactor(arg0) => Self::EvolutionaryFactor(arg0.clone()),
            Self::EmptyNeverUse => Self::EmptyNeverUse,
        }
    }
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

type SaMapAction<Message, RenderContext, Use> =
    Rc<dyn Fn(&Rc<GElement<Message, RenderContext>>, &Use) -> Rc<GElement<Message, RenderContext>>>;

pub struct SaWithMapFn<Use, Message, RenderContext> {
    u_s_e: StateAnchor<Use>,
    map_action: SaMapAction<Message, RenderContext, Use>,
}

impl<Use: Clone, Message, RenderContext> Clone for SaWithMapFn<Use, Message, RenderContext> {
    fn clone(&self) -> Self {
        Self {
            u_s_e: self.u_s_e.clone(),
            map_action: self.map_action.clone(),
        }
    }
}

impl<Use: Clone, Message, RenderContext> SaWithMapFn<Use, Message, RenderContext> {
    pub fn new(
        u_s_e: StateAnchor<Use>,
        map_action: SaMapAction<Message, RenderContext, Use>,
    ) -> Self {
        Self { u_s_e, map_action }
    }
}

impl<Use: PartialEq, Message, RenderContext> PartialEq
    for SaWithMapFn<Use, Message, RenderContext>
{
    fn eq(&self, other: &Self) -> bool {
        self.u_s_e == other.u_s_e
            && std::ptr::eq(
                (std::ptr::addr_of!(*self.map_action)).cast::<u8>(),
                (std::ptr::addr_of!(*other.map_action)).cast::<u8>(),
            )
    }
}

impl<Use, Message, RenderContext> Evolution<StateAnchor<Rc<GElement<Message, RenderContext>>>>
    for SaWithMapFn<Use, Message, RenderContext>
where
    Use: PartialEq + 'static,
    Message: 'static,
    RenderContext: 'static,
{
    fn evolution(
        &self,
        who: &StateAnchor<Rc<GElement<Message, RenderContext>>>,
    ) -> StateAnchor<Rc<GElement<Message, RenderContext>>> {
        let func = self.map_action.clone();
        (who, &self.u_s_e).map(move |gel, u_s_e| func(gel, u_s_e))
    }
}

impl<Use, Message, RenderContext> Evolution<StateAnchor<Rc<GElement<Message, RenderContext>>>>
    for StateAnchor<Use>
where
    Use: PartialEq + EqRefreshFor<GElement<Message, RenderContext>> + 'static,
    Use: std::fmt::Debug,
    StateAnchor<Use>: std::fmt::Debug,
    RenderContext: 'static,
    Message: 'static,
    // Message: PartialEq + Clone + 'static + std::fmt::Debug,
{
    fn evolution(
        &self,
        who: &StateAnchor<Rc<GElement<Message, RenderContext>>>,
    ) -> StateAnchor<Rc<GElement<Message, RenderContext>>> {
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
            new_gel.refresh_use(u_s_e);
            Rc::new(new_gel)
        })
    }
}

impl<Use, Message, RenderContext> From<SaWithMapFn<Use, Message, RenderContext>>
    for GElement<Message, RenderContext>
where
    Use: 'static,
    RenderContext: 'static,
    Message: 'static,
    // Message: Clone + PartialEq + 'static,
    //TODO check StateAnchor<Use>: Evolution<StateAnchor<Rc<Self>>>  or "SaWithMapFn<Use, Message>: Evolution<StateAnchor<Rc<Self>>>"
    SaWithMapFn<Use, Message, RenderContext>: Evolution<StateAnchor<Rc<Self>>>,
    // StateAnchor<Use>: Evolution<StateAnchor<Rc<Self>>>,
{
    fn from(sa_with_fn: SaWithMapFn<Use, Message, RenderContext>) -> Self {
        Self::EvolutionaryFactor(Rc::new(sa_with_fn))
    }
}
impl<Use, Message, RenderContext> From<StateAnchor<Use>> for GElement<Message, RenderContext>
where
    Use: 'static,
    Message: 'static,
    RenderContext: 'static,
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

    use emg_piet_gpu::RenderCtx;
    use emg_state::{use_state, StateAnchor};

    use crate::GElement;

    use super::{Evolution, SaWithMapFn};

    #[derive(Clone, Debug, PartialEq, Eq)]
    enum Message {}

    #[test]
    fn test() {
        let a = use_state(1);
        let f = SaWithMapFn {
            u_s_e: a.watch(),
            map_action: Rc::new(|p, _num| p.clone()),
        };

        let ge = use_state(GElement::<Message, RenderCtx>::EmptyNeverUse).watch();
        let _x = GElement::<Message, RenderCtx>::EvolutionaryFactor(Rc::new(f.clone()));
        let _xxx: GElement<Message, RenderCtx> = f.into();
        let _x2 =
            GElement::<Message, RenderCtx>::EvolutionaryFactor(Rc::new(a.watch())
                as Rc<dyn Evolution<StateAnchor<Rc<GElement<Message, RenderCtx>>>>>);
        let _x2 = GElement::<Message, RenderCtx>::EvolutionaryFactor(
            Rc::new(ge) as Rc<dyn Evolution<StateAnchor<Rc<GElement<Message, RenderCtx>>>>>
        );
        let _x3: GElement<Message, RenderCtx> = a.watch().into();
    }
}

impl<Message, RenderContext> Eq for GElement<Message, RenderContext> {}
impl<Message, RenderContext> PartialEq for GElement<Message, RenderContext>
// where
//     Message: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        //[] allways check when add GElement number;
        //CHECK allways check when add GElement number;
        match (self, other) {
            (Self::Builder_(l0), Self::Builder_(r0)) => l0 == r0,
            (Self::Layer_(l0), Self::Layer_(r0)) => l0 == r0,
            // (Self::Text_(l0), Self::Text_(r0)) => l0 == r0,
            // (Self::Button_(l0), Self::Button_(r0)) => l0 == r0,
            (Self::Refresher_(l0), Self::Refresher_(r0)) => (**l0) == (**r0),
            // (Self::Event_(l0), Self::Event_(r0)) => l0 == r0,
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

pub fn node_ref<Message, RenderContext>(str: impl Into<IdStr>) -> GElement<Message, RenderContext> {
    GElement::NodeRef_(str.into())
}

impl<Message, RenderContext> GElement<Message, RenderContext>
// where
//     Message: PartialEq,
{
    /// Returns `true` if the `g_element` is [`EventCallBack_`].
    #[must_use]
    // pub const fn is_event_(&self) -> bool {
    //     matches!(self, Self::Event_(..))
    // }

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

    pub fn as_dyn_node_widget(&self) -> &dyn Widget<Message, RenderContext>
    where
        Message: 'static,
        RenderContext: crate::RenderContext + 'static,
    {
        use GElement::{
            Builder_, EmptyNeverUse, EvolutionaryFactor, Generic_, Layer_, NodeRef_, Refresher_,
            SaNode_,
        };
        match_any!(self,

            // Builder_( x)| Layer_(x) | Text_(x) | Button_(x) => x as &dyn Widget<Message>,
            Builder_( x)| Layer_(x) => x as &dyn Widget<Message,RenderContext>,
            // Refresher_(_) | Event_(_) => panic!("Refresher_|Event_ can't convert to dyn widget."),
            Refresher_(_)  => panic!("Refresher_|Event_ can't convert to dyn widget."),
            Generic_(x) => {
                // debug!("Generic_:: from Generic_ to dyn Widget");
                 &**x as &dyn Widget<Message,RenderContext>
                // panic!("Generic_ should be Builder here");
                },
            NodeRef_(_)=> panic!("TryFrom<GElement to dyn Widget: \n     GElement::NodeIndex_() should handle before."),
            SaNode_(_)=>todo!(),
            EmptyNeverUse=> panic!("EmptyNeverUse never here"),
            EvolutionaryFactor(_)=> todo!()



        )
    }

    pub fn as_generic(&self) -> Option<&dyn DynGElement<Message, RenderContext>> {
        if let Self::Generic_(v) = self {
            Some(v.as_ref())
        } else {
            None
        }
    }

    // pub const fn as_text(&self) -> Option<&Text> {
    //     if let Self::Text_(v) = self {
    //         Some(v)
    //     } else {
    //         None
    //     }
    // }
}

impl<Message, RenderContext> std::fmt::Debug for GElement<Message, RenderContext>
where
// Message: std::fmt::Debug,
// RenderContext: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use GElement::{Builder_, EmptyNeverUse, Generic_, Layer_, NodeRef_, Refresher_};
        let nbw = "NodeBuilderWidget< Message>(empty Widget)".to_string();

        match self {
            Layer_(l) => f.debug_tuple("GElement::Layer").field(l).finish(),
            // Text_(t) => f.debug_tuple("GElement::Text").field(t).finish(),
            Refresher_(_) => f
                .debug_tuple("GElement::GUpdater(Rc<dyn RtUpdateFor<GElement< Message>>>)")
                .finish(),
            Builder_(builder) => {
                // if let Some(gel) = builder.widget() {
                //     f.debug_tuple("GElement::Builder_").field(gel).finish()
                // } else {
                //     f.debug_tuple("GElement::Builder_").field(&nbw).finish()
                // }
                f.debug_tuple("GElement::Builder_")
                    .field(builder.widget())
                    .finish()
            }
            // Event_(e) => f.debug_tuple("GElement::EventCallBack_").field(e).finish(),
            // Button_(_) => {
            //     write!(f, "GElement::Button_")
            // }
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

impl<Message, RenderContext> Widget<Message, RenderContext> for GElement<Message, RenderContext>
where
    RenderContext: emg_native::RenderContext + 'static,
    Message: 'static,
{
    #[instrument(skip(ctx), name = "GElement paint")]
    fn paint(&self, ctx: &mut emg_native::PaintCtx<RenderContext>) {
        self.as_dyn_node_widget().paint(ctx);
    }
}
