/*
 * @Author: Rais
 * @Date: 2022-08-18 17:52:26
 * @LastEditTime: 2023-01-30 13:48:38
 * @LastEditors: Rais
 * @Description:
 */
mod for_node_item_rc_sv;
use crate::{EventNode, GElement};
use emg_common::IdStr;
use emg_layout::EmgEdgeItem;
use emg_shaping::{EqShaping, Shaping};
use emg_state::{Dict, StateVar};
use std::{
    cell::{Ref, RefMut},
    rc::Rc,
};

pub use for_node_item_rc_sv::{GraphEdgeBuilder, GraphNodeBuilder};
// type SaBuilderFn<T> = dyn Fn(&StateAnchor<Rc<T>>) -> StateAnchor<Rc<T>>;

pub enum GTreeBuilderElement<Message, Ix = IdStr>
where
    Ix: Clone + std::hash::Hash + Ord + Default + 'static,
    Message: 'static,
{
    Layer(
        Ix,
        Vec<Rc<dyn Shaping<EmgEdgeItem<Ix>>>>, //NOTE Rc for clone
        Vec<GTreeBuilderElement<Message, Ix>>,
    ),
    // El(Ix, Element< Message>),
    GElementTree(
        Ix,
        Vec<Rc<dyn Shaping<EmgEdgeItem<Ix>>>>,
        GElement<Message>,
        Vec<GTreeBuilderElement<Message, Ix>>,
    ),
    // SaMapEffectGElementTree(
    //     Ix,
    //     Vec<Rc<dyn Shaping<EmgEdgeItem<Ix>>>>,
    //     Rc< SaBuilderFn< GElement<Message>>>,
    //     Vec<GTreeBuilderElement<Message, Ix>>,
    // ),
    ShapingUse(Ix, Rc<dyn EqShaping<GElement<Message>>>),
    Cl(Ix, Rc<dyn Fn()>),
    Event(Ix, EventNode<Message>),
    Dyn(
        Ix,
        Vec<Rc<dyn Shaping<EmgEdgeItem<Ix>>>>,
        StateVar<Dict<Ix, GTreeBuilderElement<Message, Ix>>>,
    ),
    // Fragment(Vec<GTreeBuilderElement< Message, Ix>>),
    // GenericTree(
    //     Ix,
    //     Vec<Box<dyn Shaping<EmgEdgeItem<Ix>>>>,
    //     Box<dyn DynGElement< Message> + 'static>,
    //     Vec<GTreeBuilderElement< Message, Ix>>,
    // )
}

impl<Message, Ix> Clone for GTreeBuilderElement<Message, Ix>
where
    Ix: Clone + std::hash::Hash + Ord + Default + 'static,
    Message: 'static,
{
    fn clone(&self) -> Self {
        match self {
            Self::Layer(arg0, arg1, arg2) => Self::Layer(arg0.clone(), arg1.clone(), arg2.clone()),
            Self::GElementTree(arg0, arg1, arg2, arg3) => {
                Self::GElementTree(arg0.clone(), arg1.clone(), arg2.clone(), arg3.clone())
            }
            Self::ShapingUse(arg0, arg1) => Self::ShapingUse(arg0.clone(), arg1.clone()),
            Self::Cl(arg0, arg1) => Self::Cl(arg0.clone(), arg1.clone()),
            Self::Dyn(arg0, arg1, arg2) => Self::Dyn(arg0.clone(), arg1.clone(), arg2.clone()),
            Self::Event(a, b) => Self::Event(a.clone(), b.clone()),
        }
    }
}

impl<Message, Ix> From<StateVar<Dict<Ix, Self>>> for GTreeBuilderElement<Message, Ix>
where
    Ix: Clone + std::hash::Hash + Ord + Default + 'static,
    Message: 'static,
{
    fn from(value: StateVar<Dict<Ix, Self>>) -> Self {
        //TODO check ix use default value or build uuid ?(先检查在哪里用了)
        Self::Dyn(Ix::default(), vec![], value)
    }
}

impl<Message> std::fmt::Debug for GTreeBuilderElement<Message>
// where
//     Message: std::fmt::Debug + std::clone::Clone + std::cmp::PartialEq,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Layer(id, _, children_list) => f
                .debug_tuple("GTreeBuilderElement::Layer")
                .field(id)
                .field(&"with-Edge-Vector")
                .field(children_list)
                .finish(),
            // GTreeBuilderElement::El(id, el) => f
            //     .debug_tuple("GTreeBuilderElement::El")
            //     .field(id)
            //     .field(el)
            //     .finish(),
            Self::GElementTree(id, _, gel, updaters) => f
                .debug_tuple("GTreeBuilderElement::GElementTree")
                .field(id)
                .field(&"with-Edge-Vector")
                .field(gel)
                .field(updaters)
                .finish(),
            // Self::SaMapEffectGElementTree(id, _, _builder_fn, updaters) => {
            //     f.debug_tuple("GTreeBuilderElement::SaMapEffectGElementTree")
            //         .field(id)
            //         .field(&"with-Edge-Vector")
            //         .field(&"builder_fn")
            //         .field(updaters)
            //         .finish()
            // }
            Self::ShapingUse(id, _) => f
                .debug_tuple("GTreeBuilderElement::Updater")
                .field(id)
                .field(&"Box<dyn Shaping<GElement< Message>>>")
                .finish(),
            Self::Cl(id, _) => f.debug_tuple("GTreeBuilderElement::Cl").field(id).finish(),
            Self::Event(id, e) => f
                .debug_tuple("GTreeBuilderElement::Event")
                .field(id)
                .field(&e)
                .finish(),
            Self::Dyn(id, _e, _sa_dict_gbe) => f
                .debug_tuple("GTreeBuilderElement::Dyn")
                .field(id)
                .field(&"StateVar<Dict<Ix, GTreeBuilderElement<Message, Ix>>>")
                .finish(), // GTreeBuilderElement::GenericTree(id, _, dyn_gel, updaters) => {
                           //     let edge_str = "with-Edge-Vector";
                           //     let dyn_name = format!("DynGElement({})", dyn_gel.type_name());
                           //     // let name = "DynGElement";
                           //     f.debug_tuple("GTreeBuilderElement::GenericTree")
                           //         .field(id)
                           //         .field(&edge_str)
                           //         .field(&dyn_name)
                           //         .field(updaters)
                           //         .finish()
                           // }
        }
    }
}

pub trait GTreeBuilderFn<Message>
where
    Self::Ix: Clone + Default + std::hash::Hash + Ord,
{
    type Ix;
    type GraphType;

    fn graph(&self) -> Ref<Self::GraphType>;
    fn graph_mut(&mut self) -> RefMut<Self::GraphType>;

    fn handle_root_in_topo(&self, tree_element: &GTreeBuilderElement<Message>);
    fn handle_children_in_topo(
        &self,
        replace_id: Option<&Self::Ix>,
        tree_element: &'_ GTreeBuilderElement<Message>,
    );
}
