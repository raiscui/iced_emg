/*
 * @Author: Rais
 * @Date: 2022-08-18 10:47:07
 * @LastEditTime: 2023-04-14 11:01:27
 * @LastEditors: Rais
 * @Description:
 */

// pub mod impls;
use crate::{
    node_builder::EventNode,
    widget::{Layer, Widget},
    GTreeBuilderElement, GTreeInit, InitdTree, NodeBuilderWidget,
};
use dyn_clone::DynClone;
use emg_state::{StateAnchor, StateMultiAnchor};
use match_any_cfg::match_any;

use emg_common::{
    any::MessageTid,
    better_any::{impl_tid, Tid, TidAble},
    dyn_partial_eq::DynPartialEq,
    IdStr, TypeCheckObjectSafe, TypeCheckObjectSafeTid,
};
use emg_shaping::{EqShaping, Shaping, ShapingAny, ShapingUse, ShapingUseAny};
// extern crate derive_more;
use derive_more::From;
// use dyn_clonable::clonable;
use std::{any::Any, rc::Rc};
use strum::Display;
use tracing::{instrument, warn};

#[allow(clippy::module_name_repetitions)]
// #[clonable]
//NOTE 继承Tid:从这个转换 , TidAble: 可以转换成这个
pub trait DynGElement<Message:for <'a> MessageTid<'a>>:
    // AsRefreshFor<GElement< Message>>
    for<'a> Tid<'a>
     +Shaping<GElement< Message>>
     +ShapingUse<GElement<Message>>
    // + GenerateElement<Message>
    + Widget<SceneCtxType = crate::renderer::SceneFrag>
    + TypeCheckObjectSafeTid
    + TypeCheckObjectSafe
    + DynPartialEq
    + DynClone
    + core::fmt::Debug
    + ShapingUseAny
    + ShapingUse<i32>
    + ShapingAny

{ }

#[impl_tid]
impl<'a, Message> TidAble<'a> for Box<dyn DynGElement<Message> + 'a> {}

dyn_clone::clone_trait_object!(<Message> DynGElement<Message>);

impl<Message> core::cmp::Eq for dyn DynGElement<Message> + '_ {}

impl<Message> core::cmp::PartialEq for dyn DynGElement<Message> + '_ {
    fn eq(&self, other: &Self) -> bool {
        self.box_eq(other.as_any())
    }
}
impl<Message> core::cmp::PartialEq<dyn DynGElement<Message>> for Box<dyn DynGElement<Message>>
where
    Message: 'static,
{
    fn eq(&self, other: &dyn DynGElement<Message>) -> bool {
        self.box_eq(other.as_any())
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    // use emg_piet_gpu::SceneCtx;
    use emg_shaping::{EqShaping, Shaper};
    use emg_state::use_state;

    use crate::GElement;

    #[derive(Clone, PartialEq, Eq)]
    #[allow(dead_code)]
    enum Message {
        A,
    }

    #[test]
    fn it_works() {
        let _f = GElement::<Message>::Shaper_(
            Rc::new(Shaper::new(|| 1i32)) as Rc<dyn EqShaping<GElement<Message>>>
        );
        let _a = use_state(|| 2i32);

        let _f = GElement::<Message>::Shaper_(Rc::new(_a.watch()));

        // let ff: Rc<dyn EqShaping<GElement<Message>>> = f;
        // Rc<dyn EqShaping<GElement<Message>>>, found Rc<Shaper<u32>>
    }
}

#[derive(Display, From, Tid)]
// #[eq_opt(no_self_where, where_add = "Message: PartialEq+'static,")]
#[non_exhaustive]
pub enum GElement<Message> {
    // TODO cow
    //NOTE can render element
    Builder_(NodeBuilderWidget<Message>),
    Layer_(Layer<Message>),
    // Text_(Text),
    // Button_(Button<Message>),
    Shaper_(Rc<dyn EqShaping<Self>>),
    Event_(EventNode<Message>),
    //NOTE internal
    Generic_(Box<dyn DynGElement<Message>>), //范型 //TODO check batter when use rc?
    #[from(ignore)]
    NodeRef_(IdStr),      // IntoE(Rc<dyn Into<Element< Message>>>),
    // #[from(ignore)]
    // InsideDirectUseSa_(StateAnchor<Rc<Self>>),
    //NOTE generate by tree builder use into()
    #[from(ignore)]
    SaNode_(StateAnchor<Rc<Self>>),
    EvolutionaryFactor(Rc<dyn Evolution<StateAnchor<Rc<Self>>>>),
    EmptyNeverUse,
    //@ accesskit Role ─────────────────────────────────────────────────────────────────────#[cfg(feature = "video-player")]
    #[cfg(feature = "video-player")]
    Video_(crate::component::Video),
}

impl<Message> GTreeInit<Message> for GElement<Message> {
    fn tree_init(
        self,
        _id: &IdStr,
        _es: &[Rc<dyn Shaping<emg_layout::EmgEdgeItem>>],
        _children: &[GTreeBuilderElement<Message>],
        //TODO use either like <GTreeBuilderElement,GElement> for speed??
    ) -> InitdTree<Message> {
        self.into()
    }
}

impl<Message> Clone for GElement<Message> {
    fn clone(&self) -> Self {
        match self {
            Self::Builder_(arg0) => Self::Builder_(arg0.clone()),
            Self::Layer_(arg0) => Self::Layer_(arg0.clone()),
            Self::Shaper_(arg0) => Self::Shaper_(arg0.clone()),
            Self::Generic_(arg0) => Self::Generic_(arg0.clone()),
            Self::NodeRef_(arg0) => Self::NodeRef_(arg0.clone()),
            Self::SaNode_(arg0) => Self::SaNode_(arg0.clone()),
            Self::EvolutionaryFactor(arg0) => Self::EvolutionaryFactor(arg0.clone()),
            Self::Event_(x) => Self::Event_(x.clone()),
            Self::EmptyNeverUse => Self::EmptyNeverUse,
            //@accesskit
            #[cfg(feature = "video-player")]
            Self::Video_(x) => Self::Video_(x.clone()),
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

type SaMapAction<Message, Use> = Rc<dyn Fn(&Rc<GElement<Message>>, &Use) -> Rc<GElement<Message>>>;

pub struct SaWithMapFn<Use, Message> {
    u_s_e: StateAnchor<Use>,
    map_action: SaMapAction<Message, Use>,
}

impl<Use: Clone, Message> Clone for SaWithMapFn<Use, Message> {
    fn clone(&self) -> Self {
        Self {
            u_s_e: self.u_s_e.clone(),
            map_action: self.map_action.clone(),
        }
    }
}

impl<Use: Clone, Message> SaWithMapFn<Use, Message> {
    pub fn new(u_s_e: StateAnchor<Use>, map_action: SaMapAction<Message, Use>) -> Self {
        Self { u_s_e, map_action }
    }
}

impl<Use: PartialEq, Message> PartialEq for SaWithMapFn<Use, Message> {
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
    Use: PartialEq + 'static,
    Message: 'static,
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
    Use: PartialEq + EqShaping<GElement<Message>> + 'static,
    Use: std::fmt::Debug,
    StateAnchor<Use>: std::fmt::Debug,
    Message: 'static,
    // Message: PartialEq + Clone + 'static + std::fmt::Debug,
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
    Use: 'static,
    Message: 'static,
    // Message: Clone + PartialEq + 'static,
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
    enum Message {}

    #[test]
    fn test() {
        let a = use_state(|| 1);
        let f = SaWithMapFn {
            u_s_e: a.watch(),
            map_action: Rc::new(|p, _num| p.clone()),
        };

        let ge = use_state(|| GElement::<Message>::EmptyNeverUse).watch();
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

impl<Message> Eq for GElement<Message> {}
impl<Message> PartialEq for GElement<Message>
// where
//     Message: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        //[] allways check when add GElement number;
        //CHECK: allways check when add(modifier) GElement number;
        match (self, other) {
            (Self::Builder_(l0), Self::Builder_(r0)) => l0 == r0,
            (Self::Layer_(l0), Self::Layer_(r0)) => l0 == r0,
            (Self::Shaper_(l0), Self::Shaper_(r0)) => (**l0) == (**r0),
            (Self::Event_(l0), Self::Event_(r0)) => l0 == r0,
            (Self::Generic_(l0), Self::Generic_(r0)) => l0 == r0,
            (Self::NodeRef_(l0), Self::NodeRef_(r0)) => l0 == r0,
            (Self::SaNode_(l0), Self::SaNode_(r0)) => l0 == r0,
            (Self::EvolutionaryFactor(l0), Self::EvolutionaryFactor(r0)) => l0 == r0,
            //@ accesskit ─────────────────────────────────────────────────────
            #[cfg(feature = "video-player")]
            (Self::Video_(l), Self::Video_(r)) => l == r,
            // ─────────────────────────────────────────────────────

            // with EmptyNeverUse
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

impl<Message> GElement<Message>
// where
//     Message: PartialEq,
{
    /// Returns `true` if the `g_element` is [`EventCallBack_`].
    #[must_use]
    pub const fn is_event_(&self) -> bool {
        matches!(self, Self::Event_(..))
    }

    /// Returns `true` if the g element is [`NodeIndex_`].
    ///
    /// [`NodeIndex_`]: GElement::NodeIndex_
    #[must_use]
    pub const fn is_node_ref_(&self) -> bool {
        matches!(self, Self::NodeRef_(..))
    }

    #[must_use]
    pub const fn as_node_ref_(&self) -> Option<&IdStr> {
        if let Self::NodeRef_(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn as_dyn_node_widget(&self) -> &dyn Widget<SceneCtxType = crate::renderer::SceneFrag>
    where
        Message: 'static,
    {
        use GElement::{
            Builder_, EmptyNeverUse, Event_, EvolutionaryFactor, Generic_, Layer_, NodeRef_,
            SaNode_, Shaper_,
        };

        match_any!(self,

            Builder_( x)| Layer_(x)
            => x as &(dyn Widget<SceneCtxType = crate::renderer::SceneFrag>),
            Shaper_(_)  => panic!("Refresher_|Event_ can't convert to dyn widget."),
            Generic_(x) => {
                 &**x as &(dyn Widget<SceneCtxType = crate::renderer::SceneFrag>)
                },
            NodeRef_(_)=> panic!("TryFrom<GElement to dyn Widget: \n     GElement::NodeIndex_() should handle before."),
            SaNode_(_)=>todo!(),
            EmptyNeverUse=> panic!("EmptyNeverUse never here"),
            EvolutionaryFactor(_)=> todo!(),
            Event_(_)=>todo!(),
             // @ accesskit ─────────────────────────────────────────────────────
             #[cfg(feature = "video-player")]
             Self::Video_(x) => x as &(dyn Widget<SceneCtxType = crate::renderer::SceneFrag>)



        )
    }

    #[must_use]
    pub fn as_generic(&self) -> Option<&dyn DynGElement<Message>> {
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

    #[must_use]
    pub fn as_builder(&self) -> Option<&NodeBuilderWidget<Message>> {
        if let Self::Builder_(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Returns `true` if the gelement is [`Builder_`].
    ///
    /// [`Builder_`]: GElement::Builder_
    #[must_use]
    pub fn is_builder(&self) -> bool {
        matches!(self, Self::Builder_(..))
    }
}

impl<Message> std::fmt::Debug for GElement<Message>
where
    Message: 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use GElement::{Builder_, EmptyNeverUse, Event_, Generic_, Layer_, NodeRef_, Shaper_};

        match self {
            Layer_(l) => f.debug_tuple("GElement::Layer").field(l).finish(),
            // Text_(t) => f.debug_tuple("GElement::Text").field(t).finish(),
            Shaper_(_) => f
                .debug_tuple("GElement::GUpdater(Rc<dyn RtUpdateFor<GElement< Message>>>)")
                .finish(),
            Builder_(builder) => {
                // if let Some(gel) = builder.widget() {
                //     f.debug_tuple("GElement::Builder_").field(gel).finish()
                // } else {
                //     f.debug_tuple("GElement::Builder_").field(&nbw).finish()
                // }
                f.debug_tuple("GElement::Builder_").field(builder).finish()
            }
            Event_(e) => f.debug_tuple("GElement::EventCallBack_").field(e).finish(),
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
            //@ accesskit ─────────────────────────────────────────────────────
            #[cfg(feature = "video-player")]
            Self::Video_(_) => write!(f, "GElement::Video"),
        }
    }
}

impl<Message> Widget for GElement<Message>
where
    Message: 'static,
{
    // #[instrument(skip(ctx), name = "GElement paint")]
    // fn paint(&self, ctx: &mut emg_native::PaintCtx<RenderContext>) {
    //     self.as_dyn_node_widget().paint(ctx)
    // }
    type SceneCtxType = crate::renderer::SceneFrag;

    #[instrument(skip(self, painter), name = "GElement paint",fields(self = %self))]
    #[inline]
    fn paint_sa(
        &self,
        painter: &StateAnchor<emg_native::PaintCtx>,
    ) -> StateAnchor<Rc<Self::SceneCtxType>> {
        // match self {
        //     GElement::Builder_(b) => b.paint_sa(ctx),
        //     _ => unreachable!("not builder element no paint_sa {:?}", &self),
        // }

        use GElement::{
            Builder_, EmptyNeverUse, Event_, EvolutionaryFactor, Generic_, Layer_, NodeRef_,
            SaNode_, Shaper_,
        };
        match_any!(self,

            // Builder_( x)| Layer_(x) | Text_(x) | Button_(x) => x as &dyn Widget<Message>,
            Builder_( x)| Layer_(x) => x.paint_sa(painter),
            // Refresher_(_) | Event_(_) => panic!("Refresher_|Event_ can't convert to dyn widget."),
            Shaper_(_)  => panic!("Refresher_|Event_ can't convert to dyn widget."),
            Generic_(x) => {
                warn!("Generic_:: Generic_ paint_sa");
                (x.as_ref() as &dyn Widget<SceneCtxType = Self::SceneCtxType>).paint_sa(painter)

                // panic!("Generic_ should be Builder here");
                },
            NodeRef_(_)=> panic!("TryFrom<GElement to dyn Widget: \n     GElement::NodeIndex_() should handle before."),
            SaNode_(_)=>todo!(),
            EmptyNeverUse=> panic!("EmptyNeverUse never here"),
            EvolutionaryFactor(_)=> todo!(),
            Event_(_)=> panic!("Event_() should never here"),
             //@ accesskit ─────────────────────────────────────────────────────
            #[cfg(feature = "video-player")]
            Self::Video_(x) => x.paint_sa(painter)



        )
    }
}
