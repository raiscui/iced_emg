/*
 * @Author: Rais
 * @Date: 2021-02-26 14:57:02
 * @LastEditTime: 2021-04-10 16:34:14
 * @LastEditors: Rais
 * @Description:
 */
use std::{borrow::Borrow, ops::Deref};

use crate::{runtime::Element, EventNode, GElement, GraphType, GraphView,Layer, NodeIndex};
use emg::edge_index;
use emg_layout::{emg_edge_item_default, EdgeData, EdgeData, EdgeItemNode, EmgEdgeItem};
use emg_refresh::{RefreshFor, RefreshUseFor};
use emg_state::{topo, use_state, StateVar};
use std::rc::Rc;
use tracing::{instrument, trace, trace_span};
#[allow(dead_code)]
pub enum GTreeBuilderElement<'a, Message> {
    Layer(
        String,
        Vec<Box<dyn RefreshFor<EdgeItemNode>>>,
        Vec<GTreeBuilderElement<'a, Message>>,
    ),
    El(String, Element<'a, Message>),
    GElementTree(
        String,
        Vec<Box<dyn RefreshFor<EdgeItemNode>>>,
        GElement<'a, Message>,
        Vec<GTreeBuilderElement<'a, Message>>,
    ),
    RefreshUse(String, Rc<dyn RefreshFor<GElement<'a, Message>> + 'a>),
    Cl(String, Box<dyn Fn() + 'a>),
    Event(String, EventNode<Message>),
}

impl<'a, Message: std::fmt::Debug + std::clone::Clone> std::fmt::Debug
    for GTreeBuilderElement<'a, Message>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GTreeBuilderElement::Layer(id, _, children_list) => {
                let edge_str = "with-Edge-Vector";
                f.debug_tuple("GTreeBuilderElement::Layer")
                    .field(id)
                    .field(&edge_str)
                    .field(children_list)
                    .finish()
            }
            GTreeBuilderElement::El(id, el) => f
                .debug_tuple("GTreeBuilderElement::El")
                .field(id)
                .field(el)
                .finish(),
            GTreeBuilderElement::GElementTree(id, _, gel, updaters) => {
                let edge_str = "with-Edge-Vector";

                f.debug_tuple("GTreeBuilderElement::WhoWithUpdater")
                    .field(id)
                    .field(&edge_str)
                    .field(gel)
                    .field(updaters)
                    .finish()
            }
            GTreeBuilderElement::RefreshUse(id, _) => {
                let updater = "Box<dyn RefreshFor<GElement<'a, Message>>>";
                f.debug_tuple("GTreeBuilderElement::Updater")
                    .field(id)
                    .field(&updater)
                    .finish()
            }
            GTreeBuilderElement::Cl(id, _) => {
                f.debug_tuple("GTreeBuilderElement::Cl").field(id).finish()
            }
            GTreeBuilderElement::Event(id, e) => f
                .debug_tuple("GTreeBuilderElement::Event")
                .field(id)
                .field(&e)
                .finish(),
        }
    }
}
// type NodeIndex<Ix> = (NodeIndex<String>, StateVar<Option<EdgeItemNode>>);

impl<'a, Message, Ix> GraphType<'a, Message,Ix>  {

    #[topo::nested]
pub fn handle_root<'a, Message>(
    &mut self,
    tree_layer: &GTreeBuilderElement<'a, Message>,
) where
    Message: Clone + std::fmt::Debug,
{
    match tree_layer.borrow() {
        GTreeBuilderElement::Layer(root_id, edge_refreshers, children_list) => {
            let _span = trace_span!("=> handle_root [layer] ",%id).entered();
            trace!("{:?}==>{:?}", &id, &children_list);

            // let root_ei = EmgEdgeItem::<<GraphType as crate::GraphView>::Ix>::new_root(
            let root_ei = EmgEdgeItem::<Ix>::new_root_in_topo(root_id.clone(), 1920, 1080);
            root_ei.refresh_use(edge_refreshers);

            let nix = self.insert_node(root_id.clone(), Layer::new(root_id).into());

            self.set_root_edge(root_ei);

            illicit::Layer::new().offer(nix.clone()).enter(|| {
                assert_eq!(
                    *illicit::expect::<NodeIndex<Ix>>(),
                    (nix.clone(), e_sv)
                );
                trace!("{:?}", *illicit::expect::<NodeIndex<Ix>>());
                children_list
                    .iter()
                    .for_each(|child_layer| handle_children(g, child_layer));
            });
        }
        _ => {
            panic!("not allow this , first element must layer ")
        }
    };
}
#[topo::nested]
pub fn handle_children<'a, Message>(
    &mut self,
    tree_layer: &'_ GTreeBuilderElement<'a, Message>,
) where
    Message: Clone + std::fmt::Debug,
{
    let parent_nix = *illicit::expect::<NodeIndex<Ix>>();
    match tree_layer.borrow() {
        //
        GTreeBuilderElement::Layer(id, edge_refreshers, children_list) => {
            let _span = trace_span!("-> handle_children [layer] ",%id,?parent_nix).entered();

            trace!("{:?}==>{:?}", &id, &children_list);
            // node index
            let nix = self.insert_node(id.clone(), Layer::new(id).into());

            

            // edge
            let p_nix_sv = use_state(parent_nix.clone());
            let xx = self.get_raw_edges().watch().filter_map(|eix,ei|{
                if (eix.target_node_ix() ==  )
            });

            let mut ei = edge_item_data_with_parent(id.clone(), parent_sv);
            ei.refresh_use(edge_refreshers);

            // insert to emg graph
            self.insert_update_edge(&parent_nix, &nix, ei);

            // next
            let e_sv = use_state(Some(ei.clone()));
            illicit::Layer::new().offer((nix.clone(), e_sv)).enter(|| {
                assert_eq!(
                    *illicit::expect::<NodeIndex<Ix>>(),
                    (nix.clone(), e_sv)
                );
                children_list
                    .iter()
                    .for_each(|child_layer| handle_children(g, child_layer));
            });
        }
        GTreeBuilderElement::El(id, element) => {
            let _span = trace_span!("-> handle_children [El] ",%id,?parent_nix).entered();

            let nix = self.insert_node(id.clone(), element.clone().into());

            //TODO maybe have edge_item_data_with_parent
            let e = format!("{} -> {}", parent_nix.index(), nix.index()).into();
            trace!("{}", &e);
            self.insert_update_edge(&parent_nix, &nix, e);
        }
        GTreeBuilderElement::GElementTree(id, edge_refreshers, gel, refreshers) => {
            let _span = trace_span!("-> handle_children [GElementTree] ",%id,?parent_nix).entered();

            //node index
            let nix = self.insert_node(id.clone(), gel.clone());

            //edge
            let mut e = edge_item_data_with_parent(id.clone(), parent_sv);
            e.refresh_use(edge_refreshers);

            //insert
            self.insert_update_edge(&parent_nix, &nix, e);

            //next
            let e_sv = use_state(Some(e.clone()));
            illicit::Layer::new().offer((nix.clone(), e_sv)).enter(|| {
                assert_eq!(
                    *illicit::expect::<NodeIndex<Ix>>(),
                    (nix.clone(), e_sv)
                );
                refreshers
                    .iter()
                    .for_each(|child_layer| handle_children(g, child_layer));
            });
        }
        GTreeBuilderElement::RefreshUse(id, u) => {
            let _span = trace_span!("-> handle_children [RefreshUse] ",%id,?parent_nix).entered();

            //node index
            let nix = self.insert_node(id.clone(), u.clone().into());

            //edge
            let e = format!("{} -> {}", parent_nix.index(), nix.index()).into();
            trace!("{}", &e);
            self.insert_update_edge(&parent_nix, &nix, e);
        }
        GTreeBuilderElement::Cl(_id, dyn_fn) => {
            let _span = trace_span!("-> handle_children [Cl] ",%_id,?parent_nix).entered();

            dyn_fn();
        }
        // TODO make RC remove most clones
        GTreeBuilderElement::Event(id, callback) => {
            let _span = trace_span!("-> handle_children [Event] ",%id,?parent_nix).entered();

            // TODO: make all into() style?
            // node index
            let nix = self.insert_node(id.clone(), callback.clone().into());

            //edge
            let e = format!("{} -> {}", parent_nix.index(), nix.index()).into();
            trace!("{}", &e);
            self.insert_update_edge(&parent_nix, &nix, e);
        }
    };
}
}

/// # Panics
///
/// Will panic if `tree_layer` is not `GTreeBuilderElement::Layer`


// #[must_use]
// pub fn make_id(name: &str) -> String {
//     let mut id = (*Uuid::new_v4()
//         .to_simple()
//         .encode_lower(&mut Uuid::encode_buffer()))
//     .to_string();
//     id.push_str(("-".to_owned() + name).as_str());
//     id
// }
