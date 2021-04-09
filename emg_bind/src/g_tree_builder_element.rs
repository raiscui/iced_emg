/*
 * @Author: Rais
 * @Date: 2021-02-26 14:57:02
 * @LastEditTime: 2021-04-09 12:15:23
 * @LastEditors: Rais
 * @Description:
 */
use std::{borrow::Borrow, ops::Deref};

use crate::{runtime::Element, EventNode, GElement, GraphType, Layer, NodeIndex};
use emg_layout::{edge_item_data_with_parent, EdgeData, EdgeData, EdgeItemNode};
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
/// # Panics
///
/// Will panic if `tree_layer` is not `GTreeBuilderElement::Layer`

type IllicitTreeBuildEnv = (NodeIndex<String>, StateVar<Option<EdgeItemNode>>);

#[topo::nested]
pub fn handle_root<'a, Message>(
    g: &mut GraphType<'a, Message>,
    tree_layer: &GTreeBuilderElement<'a, Message>,
) where
    Message: Clone + std::fmt::Debug,
{
    match tree_layer.borrow() {
        GTreeBuilderElement::Layer(id, edge_refreshers, children_list) => {
            let _span = trace_span!("=> handle_root [layer] ",%id).entered();
            trace!("{:?}==>{:?}", &id, &children_list);

            let e = EdgeItemNode::new_root(1920, 1080);
            e.refresh_use(edge_refreshers);

            let nix = g.insert_root(id.clone(), Layer::new(id).into(), e.clone());

            let e_sv = use_state(Some(e.clone()));
            illicit::Layer::new().offer((nix.clone(), e_sv)).enter(|| {
                assert_eq!(
                    *illicit::expect::<IllicitTreeBuildEnv>(),
                    (nix.clone(), e_sv)
                );
                trace!("{:?}", *illicit::expect::<IllicitTreeBuildEnv>());
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
    g: &mut GraphType<'a, Message>,
    tree_layer: &'_ GTreeBuilderElement<'a, Message>,
) where
    Message: Clone + std::fmt::Debug,
{
    let (parent_nix, parent_sv) = *illicit::expect::<IllicitTreeBuildEnv>();
    match tree_layer.borrow() {
        //
        GTreeBuilderElement::Layer(id, edge_refreshers, children_list) => {
            let _span = trace_span!("-> handle_children [layer] ",%id,?parent_nix).entered();

            trace!("{:?}==>{:?}", &id, &children_list);
            // node index
            let nix = g.insert_node(id.clone(), Layer::new(id).into());

            // edge

            let mut e = edge_item_data_with_parent(id.clone(), parent_sv);
            e.refresh_use(edge_refreshers);

            // insert to emg graph
            g.insert_update_edge(&parent_nix, &nix, e);

            // next
            let e_sv = use_state(Some(e.clone()));
            illicit::Layer::new().offer((nix.clone(), e_sv)).enter(|| {
                assert_eq!(
                    *illicit::expect::<IllicitTreeBuildEnv>(),
                    (nix.clone(), e_sv)
                );
                children_list
                    .iter()
                    .for_each(|child_layer| handle_children(g, child_layer));
            });
        }
        GTreeBuilderElement::El(id, element) => {
            let _span = trace_span!("-> handle_children [El] ",%id,?parent_nix).entered();

            let nix = g.insert_node(id.clone(), element.clone().into());

            //TODO maybe have edge_item_data_with_parent
            let e = format!("{} -> {}", parent_nix.index(), nix.index()).into();
            trace!("{}", &e);
            g.insert_update_edge(&parent_nix, &nix, e);
        }
        GTreeBuilderElement::GElementTree(id, edge_refreshers, gel, refreshers) => {
            let _span = trace_span!("-> handle_children [GElementTree] ",%id,?parent_nix).entered();

            //node index
            let nix = g.insert_node(id.clone(), gel.clone());

            //edge
            let mut e = edge_item_data_with_parent(id.clone(), parent_sv);
            e.refresh_use(edge_refreshers);

            //insert
            g.insert_update_edge(&parent_nix, &nix, e);

            //next
            let e_sv = use_state(Some(e.clone()));
            illicit::Layer::new().offer((nix.clone(), e_sv)).enter(|| {
                assert_eq!(
                    *illicit::expect::<IllicitTreeBuildEnv>(),
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
            let nix = g.insert_node(id.clone(), u.clone().into());

            //edge
            let e = format!("{} -> {}", parent_nix.index(), nix.index()).into();
            trace!("{}", &e);
            g.insert_update_edge(&parent_nix, &nix, e);
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
            let nix = g.insert_node(id.clone(), callback.clone().into());

            //edge
            let e = format!("{} -> {}", parent_nix.index(), nix.index()).into();
            trace!("{}", &e);
            g.insert_update_edge(&parent_nix, &nix, e);
        }
    };
}

// #[must_use]
// pub fn make_id(name: &str) -> String {
//     let mut id = (*Uuid::new_v4()
//         .to_simple()
//         .encode_lower(&mut Uuid::encode_buffer()))
//     .to_string();
//     id.push_str(("-".to_owned() + name).as_str());
//     id
// }
