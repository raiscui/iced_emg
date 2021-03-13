/*
 * @Author: Rais
 * @Date: 2021-02-26 14:57:02
 * @LastEditTime: 2021-03-13 13:52:51
 * @LastEditors: Rais
 * @Description:
 */
use std::{borrow::Borrow, ops::Deref};

use crate::{
    runtime::Element, EventCallbackType, GElement, GraphType, Layer, NodeIndex, RefreshFor, Uuid,
};
use std::rc::Rc;
use GElement::{Element_, Layer_, Refresher_};
#[allow(dead_code)]
pub enum GTreeBuilderElement<'a, Message> {
    Layer(String, Vec<GTreeBuilderElement<'a, Message>>),
    El(String, Element<'a, Message>),
    GElementTree(
        String,
        GElement<'a, Message>,
        Vec<GTreeBuilderElement<'a, Message>>,
    ),
    RefreshUse(String, Rc<dyn RefreshFor<GElement<'a, Message>> + 'a>),
    Cl(String, Box<dyn Fn() + 'a>),
    EventCallBack(String, EventCallbackType),
}

impl<'a, Message: std::fmt::Debug> std::fmt::Debug for GTreeBuilderElement<'a, Message> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GTreeBuilderElement::Layer(id, children_list) => f
                .debug_tuple("GTreeBuilderElement::Layer")
                .field(id)
                .field(children_list)
                .finish(),
            GTreeBuilderElement::El(id, el) => f
                .debug_tuple("GTreeBuilderElement::El")
                .field(id)
                .field(el)
                .finish(),
            GTreeBuilderElement::GElementTree(id, gel, updaters) => f
                .debug_tuple("GTreeBuilderElement::WhoWithUpdater")
                .field(id)
                .field(gel)
                .field(updaters)
                .finish(),
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
            GTreeBuilderElement::EventCallBack(id, _) => {
                let v = "Box<dyn EventCallbackClone>";
                f.debug_tuple("GTreeBuilderElement::EventCallBack")
                    .field(id)
                    .field(&v)
                    .finish()
            }
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
        GTreeBuilderElement::Layer(id, children_list) => {
            log::debug!("{:?}==>{:?}", &id, &children_list);
            let nix = g.insert_node(id.clone(), Layer_(Layer::new(id)));
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
        GTreeBuilderElement::Layer(id, children_list) => {
            log::debug!("{:?}==>{:?}", &id, &children_list);
            let nix = g.insert_node(id.clone(), Layer_(Layer::new(id)));
            let edge = format!("{} -> {}", parent_nix.index(), nix.index());
            log::debug!("{}", &edge);
            g.insert_update_edge(parent_nix.deref(), &nix, edge);
            illicit::Layer::new().offer(nix.clone()).enter(|| {
                assert_eq!(*illicit::expect::<NodeIndex<String>>(), nix.clone());
                children_list
                    .iter()
                    .for_each(|child_layer| handle_layer(g, child_layer));
            });
        }
        GTreeBuilderElement::El(id, element) => {
            let nix = g.insert_node(id.to_string(), Element_(element.clone()));
            let edge = format!("{} -> {}", parent_nix.index(), nix.index());
            log::debug!("{}", &edge);
            g.insert_update_edge(parent_nix.deref(), &nix, edge);
        }
        GTreeBuilderElement::GElementTree(id, gel, refreshers) => {
            let nix = g.insert_node(id.to_string(), gel.clone());
            let edge = format!("{} -> {}", parent_nix.index(), nix.index());
            log::debug!("{}", &edge);
            g.insert_update_edge(parent_nix.deref(), &nix, edge);
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
                Refresher_(u.clone()),
            );
            let edge = format!("{} -> {}", parent_nix.index(), nix.index());
            log::debug!("{}", &edge);
            g.insert_update_edge(parent_nix.deref(), &nix, edge);
        }
        GTreeBuilderElement::Cl(id, dyn_fn) => {
            dyn_fn();
        }
        // TODO make RC remove most clones
        GTreeBuilderElement::EventCallBack(id, callback) => {
            // TODO: make all into() style?
            let nix = g.insert_node(id.to_string(), callback.clone().into());
            let edge = format!("{} -> {}", parent_nix.index(), nix.index());
            log::debug!("{}", &edge);
            g.insert_update_edge(parent_nix.deref(), &nix, edge);
        }
    };
}

#[must_use]
pub fn make_id(name: &str) -> String {
    let mut id = (*Uuid::new_v4()
        .to_simple()
        .encode_lower(&mut Uuid::encode_buffer()))
    .to_string();
    id.push_str(("-".to_owned() + name).as_str());
    id
}
