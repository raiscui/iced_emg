/*
 * @Author: Rais
 * @Date: 2021-02-26 14:57:02
 * @LastEditTime: 2022-08-09 17:56:57
 * @LastEditors: Rais
 * @Description:
 */

use crate::EventNode;
use crate::GElement;
use emg::EdgeIndex;
use emg_common::IdStr;
use emg_layout::{EmgEdgeItem, GenericSizeAnchor};
use emg_shaping::{EqShaping, Shaping};
use emg_state::{Dict, StateAnchor, StateVar};
use std::rc::Rc;

type SaBuilderFn<T> = dyn Fn(&StateAnchor<Rc<T>>) -> StateAnchor<Rc<T>>;

#[allow(clippy::module_name_repetitions)]
#[allow(dead_code)]
#[derive(Clone)]
pub enum GTreeBuilderElement<Message, Ix = IdStr>
where
    Ix: Clone + std::hash::Hash + Ord + Default + 'static,
    Message: 'static + std::cmp::PartialEq + std::clone::Clone,
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

impl<Message, Ix> From<StateVar<Dict<Ix, Self>>> for GTreeBuilderElement<Message, Ix>
where
    Ix: Clone + std::hash::Hash + Ord + Default + 'static,
    Message: 'static + std::cmp::PartialEq + Clone,
{
    fn from(value: StateVar<Dict<Ix, Self>>) -> Self {
        //TODO check ix use default value or build uuid ?
        Self::Dyn(Ix::default(), vec![], value)
    }
}

impl<Message> std::fmt::Debug for GTreeBuilderElement<Message>
where
    Message: std::fmt::Debug + std::clone::Clone + std::cmp::PartialEq,
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

#[allow(clippy::module_name_repetitions)]
pub trait GTreeBuilderFn<Message>
where
    Self::Ix: Clone + Default + std::hash::Hash + Ord,
    Message: std::cmp::PartialEq + Clone,
{
    type Ix;

    /// # Errors
    ///
    /// Will return `Err` if node insert `edge_index` falls
    fn setup_edge_in_topo(
        &self,
        edge_index: EdgeIndex<Self::Ix>,

        size: (GenericSizeAnchor, GenericSizeAnchor),
        origin: (GenericSizeAnchor, GenericSizeAnchor, GenericSizeAnchor),
        align: (GenericSizeAnchor, GenericSizeAnchor, GenericSizeAnchor),
    ) -> Result<EmgEdgeItem<Self::Ix>, String>;

    /// # Errors
    ///
    /// Will return `Err` if node insert `edge_index` falls
    fn setup_default_edge_in_topo(
        &self,
        edge_index: EdgeIndex<Self::Ix>,
    ) -> Result<EmgEdgeItem<Self::Ix>, String>;

    fn handle_root_in_topo(&self, tree_element: &GTreeBuilderElement<Message>);
    fn handle_children_in_topo(
        &self,
        replace_id: Option<&Self::Ix>,
        tree_element: &'_ GTreeBuilderElement<Message>,
    );
}
