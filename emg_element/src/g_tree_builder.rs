/*
 * @Author: Rais
 * @Date: 2022-08-18 17:52:26
 * @LastEditTime: 2023-03-01 21:45:12
 * @LastEditors: Rais
 * @Description:
 */
mod impl_for_node_item_rc_sv;
use crate::{
    graph_edit::{GraphEdit, GraphEditManyMethod},
    EventNode, GElement,
};

use derive_more::From;
use emg_common::{
    better_any::{Tid, TidAble},
    im::HashSet,
    IdStr,
};
use emg_hasher::CustomHasher;
use emg_layout::EmgEdgeItem;
use emg_shaping::{EqShaping, Shaping};
use emg_state::{Dict, StateVar};
use std::{any::TypeId, hash::BuildHasherDefault, panic::Location, rc::Rc};

pub use impl_for_node_item_rc_sv::{GraphEdgeBuilder, GraphNodeBuilder};
// type SaBuilderFn<T> = dyn Fn(&StateAnchor<Rc<T>>) -> StateAnchor<Rc<T>>;
// ─────────────────────────────────────────────────────────────────────────────

#[derive(From)]
pub enum InitdTree<Message: 'static> {
    Builder(GTreeBuilderElement<Message>),
    Gel(GElement<Message>),
}

type EdgesAndChildren<Message> = (
    Vec<Rc<dyn Shaping<EmgEdgeItem>>>,
    Vec<GTreeBuilderElement<Message>>,
);

impl<Message> InitdTree<Message> {
    fn merge_es_and_children(
        opt_es: Option<Vec<Rc<dyn Shaping<EmgEdgeItem>>>>,
        mut o_es: Vec<Rc<dyn Shaping<EmgEdgeItem>>>,
        opt_children: Option<Vec<GTreeBuilderElement<Message>>>,
        mut o_children: Vec<GTreeBuilderElement<Message>>,
    ) -> EdgesAndChildren<Message> {
        let new_es = opt_es
            .map(|mut es| {
                es.append(&mut o_es);
                es
            })
            .unwrap_or_else(|| o_es);
        let new_children = opt_children
            .map(|mut children| {
                children.append(&mut o_children);
                children
            })
            .unwrap_or_else(|| o_children);
        (new_es, new_children)
    }

    //TODO work here make fn with_id_edge_children
    ///NOTE gtree macro will call this
    pub fn with_id_edge_children(
        self,
        id: IdStr, //outSide generated id
        opt_es: Option<Vec<Rc<dyn Shaping<EmgEdgeItem>>>>,
        opt_children: Option<Vec<GTreeBuilderElement<Message>>>,
    ) -> GTreeBuilderElement<Message> {
        match self {
            InitdTree::Gel(gel) => {
                GTreeBuilderElement::GElementTree(id, opt_es.unwrap(), gel, opt_children.unwrap())
            }
            InitdTree::Builder(b) => match b {
                // GTreeBuilderElement::Layer(_, o_es, o_children) => {
                //     unreachable!("deprecated");
                //     // let (new_es, new_children) =
                //     //     Self::merge_es_and_children(opt_es, o_es, opt_children, o_children);

                //     // GTreeBuilderElement::Layer(id, new_es, new_children)
                // }
                GTreeBuilderElement::GElementTree(_inside_generated_id, o_es, gel, o_children) => {
                    let (new_es, new_children) =
                        Self::merge_es_and_children(opt_es, o_es, opt_children, o_children);
                    //TODO check if id is not defined ,then use self id
                    GTreeBuilderElement::GElementTree(id, new_es, gel, new_children)
                }
                GTreeBuilderElement::ShapingUse(_, _) => todo!(),
                GTreeBuilderElement::Cl(_, _) => todo!(),
                GTreeBuilderElement::Event(_, _) => todo!(),
                GTreeBuilderElement::Dyn(_, _, _) => todo!(),
            },
        }
    }
}

pub trait GTreeInit<Message> {
    fn tree_init(
        self,
        _id: &IdStr,
        _es: &[Rc<dyn Shaping<EmgEdgeItem>>],
        _children: &[GTreeBuilderElement<Message>],
    ) -> InitdTree<Message>;
}
pub trait GtreeInitCall<Message> {
    //NOTE for the loopback check
    fn tree_init_calling(
        self,
        _id: &IdStr,
        _es: &[Rc<dyn Shaping<EmgEdgeItem>>],
        _children: &[GTreeBuilderElement<Message>],
    ) -> InitdTree<Message>;
}

type TypeIdSets = HashSet<TypeId, BuildHasherDefault<CustomHasher>>;

impl<T, Message> GtreeInitCall<Message> for T
where
    // Message: Clone + PartialEq + for<'a> emg_common::any::MessageTid<'a>,
    T: GTreeInit<Message> + for<'a> Tid<'a>,
{
    ///NOTE for the loopback check
    ///NOTE gtree macro will call this
    #[track_caller]
    fn tree_init_calling(
        self,
        id: &IdStr,
        es: &[Rc<dyn Shaping<EmgEdgeItem>>],
        children: &[GTreeBuilderElement<Message>],
    ) -> InitdTree<Message> {
        let new_type_sets = if let Some(type_sets) = illicit::get::<TypeIdSets>().ok().as_deref() {
            let self_id = self.self_id();

            //  checking loopback
            if !type_sets.contains(&self_id) {
                type_sets.update(self_id)
            } else {
                panic!(
                    "tree_init is loopback because type:{}  at: {}",
                    std::any::type_name::<Self>(),
                    Location::caller()
                );
            }
        } else {
            //first time
            TypeIdSets::default()
        };

        illicit::Layer::new()
            .offer(new_type_sets)
            .enter(|| self.tree_init(id, es, children))
    }
}

#[derive(Tid)]
pub enum GTreeBuilderElement<Message>
where
    Message: 'static,
{
    // Layer(
    //     IdStr,
    //     Vec<Rc<dyn Shaping<EmgEdgeItem>>>, //NOTE Rc for clone
    //     Vec<GTreeBuilderElement<Message>>,
    // ),
    // El(IdStr, Element< Message>),
    GElementTree(
        IdStr,
        Vec<Rc<dyn Shaping<EmgEdgeItem>>>,
        GElement<Message>,
        Vec<GTreeBuilderElement<Message>>,
    ),
    // SaMapEffectGElementTree(
    //     IdStr,
    //     Vec<Rc<dyn Shaping<EmgEdgeItem>>>,
    //     Rc< SaBuilderFn< GElement<Message>>>,
    //     Vec<GTreeBuilderElement<Message>>,
    // ),
    ShapingUse(IdStr, Rc<dyn EqShaping<GElement<Message>>>),
    Cl(IdStr, Rc<dyn Fn()>),
    Event(IdStr, EventNode<Message>),
    Dyn(
        IdStr,
        Vec<Rc<dyn Shaping<EmgEdgeItem>>>,
        StateVar<Dict<IdStr, GTreeBuilderElement<Message>>>,
    ),
    // Fragment(Vec<GTreeBuilderElement< Message>>),
    // GenericTree(
    //     IdStr,
    //     Vec<Box<dyn Shaping<EmgEdgeItem>>>,
    //     Box<dyn DynGElement< Message> + 'static>,
    //     Vec<GTreeBuilderElement< Message>>,
    // )
}

impl<Message> GTreeInit<Message> for GTreeBuilderElement<Message>
where
    Message: 'static,
{
    fn tree_init(
        self,
        _id: &IdStr,
        _es: &[Rc<dyn Shaping<EmgEdgeItem>>],
        _children: &[GTreeBuilderElement<Message>],
    ) -> InitdTree<Message> {
        self.into()
    }
}

impl<Message> Clone for GTreeBuilderElement<Message>
where
    Message: 'static,
{
    fn clone(&self) -> Self {
        match self {
            // Self::Layer(arg0, arg1, arg2) => Self::Layer(arg0.clone(), arg1.clone(), arg2.clone()),
            Self::GElementTree(arg0, arg1, arg2, arg3) => {
                Self::GElementTree(arg0.clone(), arg1.clone(), arg2.clone(), arg3.clone())
            }
            Self::ShapingUse(arg0, arg1) => Self::ShapingUse(arg0.clone(), arg1.clone()),
            Self::Cl(arg0, arg1) => Self::Cl(arg0.clone(), arg1.clone()),
            Self::Dyn(arg0, arg1, arg2) => Self::Dyn(arg0.clone(), arg1.clone(), *arg2),
            Self::Event(a, b) => Self::Event(a.clone(), b.clone()),
        }
    }
}

impl<Message> From<StateVar<Dict<IdStr, Self>>> for GTreeBuilderElement<Message>
where
    Message: 'static,
{
    fn from(value: StateVar<Dict<IdStr, Self>>) -> Self {
        //TODO check ix use default value or build uuid ?(先检查在哪里用了)
        Self::Dyn(IdStr::default(), vec![], value)
    }
}

impl<Message> std::fmt::Debug for GTreeBuilderElement<Message>
// where
//     Message: std::fmt::Debug + std::clone::Clone + std::cmp::PartialEq,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // Self::Layer(id, es, children_list) => {
            //     let es_size = es.len();

            //     f.debug_tuple("GTreeBuilderElement::Layer")
            //         .field(id)
            //         .field(&format!("with Some Edge Vector...size:{es_size}"))
            //         .field(children_list)
            //         .finish()
            // }
            Self::GElementTree(id, es, gel, updaters) => {
                let es_size = es.len();

                f.debug_tuple("GTreeBuilderElement::GElementTree")
                    .field(id)
                    .field(&format!("with Some Edge Vector...size:{es_size}"))
                    .field(gel)
                    .field(updaters)
                    .finish()
            }
            // Self::SaMapEffectGElementTree(id, _, _builder_fn, updaters) => {
            //     f.debug_tuple("GTreeBuilderElement::SaMapEffectGElementTree")
            //         .field(id)
            //         .field(&"with Some Edge Vector...")
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
                .field(&"StateVar<Dict<IdStr, GTreeBuilderElement<Message>>>")
                .finish(), // GTreeBuilderElement::GenericTree(id, _, dyn_gel, updaters) => {
                           //     let edge_str = "with Some Edge Vector...";
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

pub trait GTreeBuilderFn<Message> {
    type GraphType;
    type GraphEditor: GraphEdit + GraphEditManyMethod;

    fn editor(&self) -> Self::GraphEditor;

    fn graph(&self) -> &Self::GraphType;

    // #[deprecated = 直接使用handle_children_in_topo]
    fn handle_root_in_topo(&self, tree_element: &GTreeBuilderElement<Message>);
    fn handle_children_in_topo(
        &self,
        replace_id: Option<&IdStr>,
        tree_element: &'_ GTreeBuilderElement<Message>,
    );
}
