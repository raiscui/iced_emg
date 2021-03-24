/*
 * @Author: Rais
 * @Date: 2021-02-26 14:57:02
 * @LastEditTime: 2021-03-24 12:48:40
 * @LastEditors: Rais
 * @Description:
 */
use std::{borrow::Borrow, ops::Deref};

use crate::{runtime::Element, EventNode, GElement, GraphType, Layer, NodeIndex};
use emg_layout::{e, EdgeData};
use emg_refresh::{RefreshFor, RefreshUseFor};
use std::rc::Rc;
#[allow(dead_code)]
pub enum GTreeBuilderElement<'a, Message> {
    Layer(
        String,
        Vec<Box<dyn RefreshFor<EdgeData>>>,
        Vec<GTreeBuilderElement<'a, Message>>,
    ),
    El(String, Element<'a, Message>),
    GElementTree(
        String,
        Vec<Box<dyn RefreshFor<EdgeData>>>,
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
pub fn handle_root<'a, Message>(
    g: &mut GraphType<'a, Message>,
    tree_layer: &GTreeBuilderElement<'a, Message>,
) where
    Message: Clone + std::fmt::Debug,
{
    match tree_layer.borrow() {
        GTreeBuilderElement::Layer(id, _, children_list) => {
            log::debug!("{:?}==>{:?}", &id, &children_list);
            let nix = g.insert_node(id.clone(), Layer::new(id).into());
            illicit::Layer::new().offer(nix.clone()).enter(|| {
                assert_eq!(*illicit::expect::<NodeIndex<String>>(), nix.clone());
                log::debug!("{:?}", *illicit::expect::<NodeIndex<String>>());
                children_list
                    .iter()
                    .for_each(|child_layer| handle_layer(g, child_layer));
            });
        }
        _ => {
            panic!("not allow this , first element must layer ")
        }
    };
}
pub fn handle_layer<'a, Message>(
    g: &mut GraphType<'a, Message>,
    tree_layer: &'_ GTreeBuilderElement<'a, Message>,
) where
    Message: Clone + std::fmt::Debug,
{
    let parent_nix = illicit::expect::<NodeIndex<String>>();
    match tree_layer.borrow() {
        GTreeBuilderElement::Layer(id, edge_refreshers, children_list) => {
            log::debug!("{:?}==>{:?}", &id, &children_list);
            let nix = g.insert_node(id.clone(), Layer::new(id).into());
            // let edge = format!("{} -> {}", parent_nix.index(), nix.index());
            let mut e = e();
            e.refresh_use(edge_refreshers);
            // log::debug!("{}", &edge);
            g.insert_update_edge(parent_nix.deref(), &nix, e.into());
            illicit::Layer::new().offer(nix.clone()).enter(|| {
                assert_eq!(*illicit::expect::<NodeIndex<String>>(), nix.clone());
                children_list
                    .iter()
                    .for_each(|child_layer| handle_layer(g, child_layer));
            });
        }
        GTreeBuilderElement::El(id, element) => {
            let nix = g.insert_node(id.to_string(), element.clone().into());
            let edge = format!("{} -> {}", parent_nix.index(), nix.index());
            log::debug!("{}", &edge);
            g.insert_update_edge(parent_nix.deref(), &nix, edge.into());
        }
        GTreeBuilderElement::GElementTree(id, edge_refreshers, gel, refreshers) => {
            let nix = g.insert_node(id.to_string(), gel.clone());
            // let edge = format!("{} -> {}", parent_nix.index(), nix.index());
            let mut e = e();
            e.refresh_use(edge_refreshers);

            // log::debug!("{}", &edge);
            g.insert_update_edge(parent_nix.deref(), &nix, e.into());
            illicit::Layer::new().offer(nix.clone()).enter(|| {
                assert_eq!(*illicit::expect::<NodeIndex<String>>(), nix.clone());
                refreshers
                    .iter()
                    .for_each(|child_layer| handle_layer(g, child_layer));
            });
        }
        GTreeBuilderElement::RefreshUse(id, u) => {
            let nix = g.insert_node(
                id.to_string(),
                // Refresher_(Rc::<dyn RefreshFor<GElement<'a, Message>> + 'a>::from(u)),
                u.clone().into(),
            );
            let edge = format!("{} -> {}", parent_nix.index(), nix.index());
            log::debug!("{}", &edge);
            g.insert_update_edge(parent_nix.deref(), &nix, edge.into());
        }
        GTreeBuilderElement::Cl(_id, dyn_fn) => {
            dyn_fn();
        }
        // TODO make RC remove most clones
        GTreeBuilderElement::Event(id, callback) => {
            // TODO: make all into() style?
            let nix = g.insert_node(id.to_string(), callback.clone().into());
            let edge = format!("{} -> {}", parent_nix.index(), nix.index());
            log::debug!("{}", &edge);
            g.insert_update_edge(parent_nix.deref(), &nix, edge.into());
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
