/*
 * @Author: Rais
 * @Date: 2021-02-26 14:57:02
 * @LastEditTime: 2021-03-04 16:02:06
 * @LastEditors: Rais
 * @Description:
 */
use std::{borrow::Borrow, ops::Deref};

use crate::{runtime::Element, GElement, GraphType, Layer, NodeIndex, RefreshFor, Uuid};
use std::rc::Rc;
use GElement::*;
#[allow(dead_code)]
pub enum GTreeBuilderElement<'a, Message> {
    Layer(String, Vec<GTreeBuilderElement<'a, Message>>),
    El(Element<'a, Message>),
    WhoWithUpdater(GElement<'a, Message>, Vec<GTreeBuilderElement<'a, Message>>),
    Updater(Rc<dyn RefreshFor<GElement<'a, Message>> + 'a>),
    Cl(Box<dyn Fn() + 'a>),
}
impl<Message> Default for GTreeBuilderElement<'_, Message> {
    fn default() -> Self {
        GTreeBuilderElement::Cl(Box::new(|| {}))
    }
}

impl<'a, Message> std::fmt::Debug for GTreeBuilderElement<'a, Message> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GTreeBuilderElement::Layer(s, children_list) => f
                .debug_tuple("GTreeBuilderElement::Layer")
                .field(s)
                .field(children_list)
                .finish(),
            GTreeBuilderElement::El(el) => {
                f.debug_tuple("GTreeBuilderElement::El").field(el).finish()
            }
            GTreeBuilderElement::WhoWithUpdater(_, updaters) => {
                let who = "GElement<'a, Message> (dyn RefreshUseFor<GElement<'a, Message>>)";
                f.debug_tuple("GTreeBuilderElement::WhoWithUpdater")
                    .field(&who)
                    .field(updaters)
                    .finish()
            }
            GTreeBuilderElement::Updater(_) => {
                let updater = "Box<dyn RefreshFor<GElement<'a, Message>>>";
                f.debug_tuple("GTreeBuilderElement::Updater")
                    .field(&updater)
                    .finish()
            }
            GTreeBuilderElement::Cl(_) => f.debug_tuple("GTreeBuilderElement::Cl").finish(),
        }
    }
}
pub fn handle_root<'a, Message>(
    g: &mut GraphType<'a, Message>,
    tree_layer: Rc<GTreeBuilderElement<'a, Message>>,
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
                    .into_iter()
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
                    .into_iter()
                    .for_each(|child_layer| handle_layer(g, child_layer));
            });
        }
        GTreeBuilderElement::El(element) => {
            let mut id = Uuid::new_v4()
                .to_simple()
                .encode_lower(&mut Uuid::encode_buffer())
                .to_string();
            id.push_str("-Element");
            let nix = g.insert_node(id, Element_(element.clone()));
            let edge = format!("{} -> {}", parent_nix.index(), nix.index());
            log::debug!("{}", &edge);
            g.insert_update_edge(parent_nix.deref(), &nix, edge);
        }
        GTreeBuilderElement::WhoWithUpdater(gel, updaters) => {
            let mut id = Uuid::new_v4()
                .to_simple()
                .encode_lower(&mut Uuid::encode_buffer())
                .to_string();
            id.push_str(format!("-{}", &gel).as_ref());
            let nix = g.insert_node(id, gel.clone());
            let edge = format!("{} -> {}", parent_nix.index(), nix.index());
            log::debug!("{}", &edge);
            g.insert_update_edge(parent_nix.deref(), &nix, edge);
            illicit::Layer::new().offer(nix.clone()).enter(|| {
                assert_eq!(*illicit::expect::<NodeIndex<String>>(), nix.clone());
                updaters
                    .into_iter()
                    .for_each(|child_layer| handle_layer(g, child_layer));
            });
        }
        GTreeBuilderElement::Updater(u) => {
            let mut id = Uuid::new_v4()
                .to_simple()
                .encode_lower(&mut Uuid::encode_buffer())
                .to_string();
            id.push_str("-Refresher");
            let nix = g.insert_node(
                id,
                // Refresher_(Rc::<dyn RefreshFor<GElement<'a, Message>> + 'a>::from(u)),
                Refresher_(u.clone()),
            );
            let edge = format!("{} -> {}", parent_nix.index(), nix.index());
            log::debug!("{}", &edge);
            g.insert_update_edge(parent_nix.deref(), &nix, edge);
        }
        GTreeBuilderElement::Cl(dyn_fn) => {
            dyn_fn();
        }
    };
}
