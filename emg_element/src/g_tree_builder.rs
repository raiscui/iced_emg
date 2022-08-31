/*
 * @Author: Rais
 * @Date: 2022-08-18 17:52:26
 * @LastEditTime: 2022-08-31 13:06:50
 * @LastEditors: Rais
 * @Description:
 */
mod node_item_rc_sv;
use crate::GElement;
use emg::EdgeIndex;
use emg_common::IdStr;
use emg_layout::{EmgEdgeItem, GenericSizeAnchor};
use emg_refresh::{EqRefreshFor, RefreshFor};
use emg_state::{Dict, StateAnchor, StateVar};
use std::{cell::Ref, rc::Rc};

type SaBuilderFn<T> = dyn Fn(&StateAnchor<Rc<T>>) -> StateAnchor<Rc<T>>;

pub enum GTreeBuilderElement<Message, RenderCtx, Ix = IdStr>
where
    Ix: Clone + std::hash::Hash + Ord + Default + 'static,
    Message: 'static,
{
    Layer(
        Ix,
        Vec<Rc<dyn RefreshFor<EmgEdgeItem<Ix, RenderCtx>>>>, //NOTE Rc for clone
        Vec<GTreeBuilderElement<Message, RenderCtx, Ix>>,
    ),
    // El(Ix, Element< Message>),
    GElementTree(
        Ix,
        Vec<Rc<dyn RefreshFor<EmgEdgeItem<Ix, RenderCtx>>>>,
        GElement<Message, RenderCtx>,
        Vec<GTreeBuilderElement<Message, RenderCtx, Ix>>,
    ),
    // SaMapEffectGElementTree(
    //     Ix,
    //     Vec<Rc<dyn RefreshFor<EmgEdgeItem<Ix>>>>,
    //     Rc< SaBuilderFn< GElement<Message>>>,
    //     Vec<GTreeBuilderElement<Message, Ix>>,
    // ),
    RefreshUse(Ix, Rc<dyn EqRefreshFor<GElement<Message, RenderCtx>>>),
    Cl(Ix, Rc<dyn Fn()>),
    // Event(Ix, EventNode<Message>),
    Dyn(
        Ix,
        Vec<Rc<dyn RefreshFor<EmgEdgeItem<Ix, RenderCtx>>>>,
        StateVar<Dict<Ix, GTreeBuilderElement<Message, RenderCtx, Ix>>>,
    ),
    // Fragment(Vec<GTreeBuilderElement< Message, Ix>>),
    // GenericTree(
    //     Ix,
    //     Vec<Box<dyn RefreshFor<EmgEdgeItem<Ix>>>>,
    //     Box<dyn DynGElement< Message> + 'static>,
    //     Vec<GTreeBuilderElement< Message, Ix>>,
    // )
}

impl<Message, RenderContext, Ix> Clone for GTreeBuilderElement<Message, RenderContext, Ix>
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
            Self::RefreshUse(arg0, arg1) => Self::RefreshUse(arg0.clone(), arg1.clone()),
            Self::Cl(arg0, arg1) => Self::Cl(arg0.clone(), arg1.clone()),
            Self::Dyn(arg0, arg1, arg2) => Self::Dyn(arg0.clone(), arg1.clone(), arg2.clone()),
        }
    }
}

impl<Message, RenderContext, Ix> From<StateVar<Dict<Ix, Self>>>
    for GTreeBuilderElement<Message, RenderContext, Ix>
where
    Ix: Clone + std::hash::Hash + Ord + Default + 'static,
    Message: 'static,
{
    fn from(value: StateVar<Dict<Ix, Self>>) -> Self {
        //TODO check ix use default value or build uuid ?
        Self::Dyn(Ix::default(), vec![], value)
    }
}

impl<Message, RenderContext> std::fmt::Debug for GTreeBuilderElement<Message, RenderContext>
// where
//     Message: std::fmt::Debug + std::clone::Clone + std::cmp::PartialEq,
where
    RenderContext: 'static,
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
            Self::RefreshUse(id, _) => f
                .debug_tuple("GTreeBuilderElement::Updater")
                .field(id)
                .field(&"Box<dyn RefreshFor<GElement< Message>>>")
                .finish(),
            Self::Cl(id, _) => f.debug_tuple("GTreeBuilderElement::Cl").field(id).finish(),
            // Self::Event(id, e) => f
            //     .debug_tuple("GTreeBuilderElement::Event")
            //     .field(id)
            //     .field(&e)
            //     .finish(),
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

pub trait GTreeBuilderFn<Message, RenderCtx>
where
    Self::Ix: Clone + Default + std::hash::Hash + Ord,
{
    type Ix;
    type GraphType;

    fn graph(&self) -> Ref<Self::GraphType>;

    /// # Errors
    ///
    /// Will return `Err` if node insert `edge_index` falls
    fn setup_edge_in_topo(
        &self,
        edge_index: EdgeIndex<Self::Ix>,
        size: (GenericSizeAnchor, GenericSizeAnchor),
        origin: (GenericSizeAnchor, GenericSizeAnchor, GenericSizeAnchor),
        align: (GenericSizeAnchor, GenericSizeAnchor, GenericSizeAnchor),
        //TODO right error type
    ) -> Result<EmgEdgeItem<Self::Ix, RenderCtx>, String>;

    /// # Errors
    ///
    /// Will return `Err` if node insert `edge_index` falls
    fn setup_default_edge_in_topo(
        &self,
        edge_index: EdgeIndex<Self::Ix>,
    ) -> Result<EmgEdgeItem<Self::Ix, RenderCtx>, String>;

    fn handle_root_in_topo(&self, tree_element: &GTreeBuilderElement<Message, RenderCtx>);
    fn handle_children_in_topo(
        &self,
        replace_id: Option<&Self::Ix>,
        tree_element: &'_ GTreeBuilderElement<Message, RenderCtx>,
    );
}
