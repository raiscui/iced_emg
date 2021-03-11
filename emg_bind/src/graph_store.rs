/*
 * @Author: Rais
 * @Date: 2021-01-21 11:05:55
 * @LastEditTime: 2021-03-11 16:27:28
 * @LastEditors: Rais
 * @Description:
 */
pub use emg::Graph;
pub use emg::NodeIndex;
use emg::Outgoing;

use crate::{runtime::Element, GElement, GStateStore, NodeBuilderWidget, RefreshUseFor};
use anymap::any::CloneAny;
use std::{cell::RefCell, convert::TryInto};
use std::{convert::TryFrom, hash::Hash};

// use lazy_static::lazy_static;

// ────────────────────────────────────────────────────────────────────────────────

pub type N<'a, Message> = GElement<'a, Message>;
// pub type N<'a, Message> = RefCell<GElement<'a, Message>>;
pub type E = String;
pub type GraphType<'a, Message> = Graph<N<'a, Message>, E>;

thread_local! {
    pub static G_STORE: RefCell<GStore> = RefCell::new(
         GStore::default()
    );
}
thread_local! {
    pub static G_STATE_STORE: RefCell<GStateStore> = RefCell::new(
        GStateStore::default()
    );
}
use anchors::singlethread::Engine;
thread_local! {
    pub static ENGINE: RefCell<Engine> = RefCell::new(Engine::new());
}
// pub static ENGINE: RefCell<Engine> = RefCell::new(Engine::new());
// lazy_static! {
//     pub static ref ENGINE: RefCell<Engine> = RefCell::new(Engine::new());
// }

// impl<'a, T, Message> From<T> for Element<'a, Message>
// where
//     Message: 'static + Clone,
//     T: Into<Element<'a, Message>> + Clone,
// {
//     fn from(can_into_element: &T) -> Element<'a, Message> {
//         can_into_element.clone().into()
//     }
// }

// ────────────────────────────────────────────────────────────────────────────────

pub trait GraphView<'a, Message> {
    type N;
    type Ix;
    type E;

    fn gelement_comb_and_refresh(
        &self,
        cix: &Self::Ix,
        // current_node: &RefCell<GElement<'a, Message>>,
    ) -> GElement<'a, Message>;

    fn children_to_elements(&self, cix: &Self::Ix) -> Vec<GElement<'a, Message>>;

    fn view(&self, ix: Self::Ix) -> Element<'a, Message>;
    // fn global_view(ix: Self::Ix) -> Element<'a, Message>;
}

impl<'a, Message, E, Ix> GraphView<'a, Message> for Graph<N<'a, Message>, E, Ix>
where
    Ix: Clone + Hash + Eq + std::fmt::Debug,
    E: Clone + std::fmt::Debug,
    Message: 'static + Clone + std::fmt::Debug,
{
    type Ix = Ix;
    type N = N<'a, Message>;
    type E = E;

    fn gelement_comb_and_refresh(
        &self,
        cix: &Self::Ix, // current_node: &RefCell<GElement<'a, Message>>,
    ) -> GElement<'a, Message> {
        // buildingTime original GElement
        let mut current_node_clone = self.get_node_weight_use_ix(cix).unwrap().clone();

        let mut children_s = self.children_to_elements(cix);

        let event_callbacks = children_s
            .drain_filter(|gel| gel.is_event_call_back_())
            .collect::<Vec<_>>();

        // The const / dyn child node performs the change
        // TODO: cache.    use edge type?
        for child in children_s {
            current_node_clone.refresh_use(&child)
        }

        // event_callback handle -----------------------
        if event_callbacks.is_empty() {
            current_node_clone
        } else {
            match NodeBuilderWidget::<Message>::try_from(current_node_clone) {
                Ok(mut node_builder_widget) => {
                    for event_callback in event_callbacks {
                        node_builder_widget.refresh_use(&event_callback)
                    }
                    GElement::Element_(node_builder_widget.into())
                }
                Err(old_gel) => old_gel,
            }

            // if let Ok(node_builder_widget) =
            //     current_node_clone.try_convert_inside_to_node_builder_widget_()
            // {
            //     for event_callback in event_callbacks {
            //         node_builder_widget.refresh_use(&event_callback)
            //     }
            // }
        }
    }

    fn children_to_elements(&self, cix: &Self::Ix) -> Vec<GElement<'a, Message>> {
        self.edges_iter_use_ix(cix, Outgoing)
            .map(|eix| {
                let this_child_ix = eix.ix_dir(Outgoing);
                // let a_child = self.get_node_weight_use_ix(child_ix).unwrap();
                self.gelement_comb_and_refresh(this_child_ix)
            })
            .collect()
    }

    fn view(&self, cix: Self::Ix) -> Element<'a, Message> {
        self.gelement_comb_and_refresh(&cix).try_into().unwrap()
    }

    // fn global_view(cix: Self::Ix) -> Element<'a, Message> {
    //     G_STORE.with(|g_store_refcell| {
    //         // g_store_refcell.borrow_mut().set_graph(g);
    //         g_store_refcell
    //             .borrow()
    //             .get_graph::<Self::N, Self::E, Self::Ix>()
    //             .gelement_comb_and_refresh(&cix)
    //             .try_into()
    //             .unwrap()
    //     })
    // }
}
pub trait GraphStore<'a, Message> {
    type N;
    type Ix;
    type E;
    fn global_init();
    fn global_get_mut_graph_with<F: FnOnce(&mut Self) -> R, R>(func: F) -> R;
    // fn add_el(&mut self, key: Self::Ix, e_item: Self::E, n_item: Self::N) -> NodeIndex<Self::Ix>
    // where
    //     Self::Ix: Clone;

    fn global_gelement_comb_and_refresh(
        &self,
        cix: &Self::Ix,
        // current_node: &RefCell<GElement<'a, Message>>,
    ) -> GElement<'a, Message>;

    fn global_children_to_elements(&self, cix: &Self::Ix) -> Vec<GElement<'a, Message>>;

    // fn view(&self, ix: Self::Ix) -> Element<'_, Message>;
    fn global_view(ix: Self::Ix) -> Element<'a, Message>;
}

impl<'a, Message, E, Ix> GraphStore<'a, Message> for Graph<N<'a, Message>, E, Ix>
where
    Ix: Clone + Hash + Eq + std::fmt::Debug,
    E: Clone + std::fmt::Debug,
    // N: Clone,
    Self: 'static,
    Message: 'static + Clone + std::fmt::Debug,
{
    type Ix = Ix;
    type N = N<'a, Message>;
    type E = E;
    fn global_init() {
        // console_log::init_with_level(Level::Debug).ok();

        G_STORE.with(|g_store_refcell| {
            // g_store_refcell.borrow_mut().set_graph(g);
            g_store_refcell.borrow_mut().set_graph(Self::default());
        });
    }

    fn global_get_mut_graph_with<F, R>(func: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        G_STORE.with(|g_store_refcell| {
            // g_store_refcell.borrow_mut().set_graph(g);
            g_store_refcell.borrow_mut().get_mut_graph_with(func)
        })
    }

    fn global_children_to_elements(&self, cix: &Self::Ix) -> Vec<GElement<'a, Message>> {
        self.edges_iter_use_ix(cix, Outgoing)
            .map(|eix| {
                let this_child_ix = eix.ix_dir(Outgoing);
                // let a_child = self.get_node_weight_use_ix(child_ix).unwrap();
                self.global_gelement_comb_and_refresh(this_child_ix)
            })
            .collect()
    }

    fn global_gelement_comb_and_refresh(
        &self,
        cix: &Self::Ix, // current_node: &RefCell<GElement<'a, Message>>,
    ) -> GElement<'a, Message> {
        // buildingTime original GElement
        let mut current_node_clone = self.get_node_weight_use_ix(cix).unwrap().clone();

        let children_s = self.global_children_to_elements(cix);

        // The const / dyn child node performs the change
        // TODO: cache.    use edge type?
        for child in children_s {
            current_node_clone.refresh_use(&child)
        }

        current_node_clone
    }

    fn global_view(cix: Self::Ix) -> Element<'a, Message> {
        G_STORE.with(|g_store_refcell| {
            // g_store_refcell.borrow_mut().set_graph(g);
            g_store_refcell
                .borrow()
                .get_graph::<Self::N, Self::E, Self::Ix>()
                .global_gelement_comb_and_refresh(&cix)
                .try_into()
                .unwrap()
        })
    }
}

#[derive(Clone, Debug)]
pub struct GStore {
    pub anymap: anymap::Map<dyn CloneAny>,
}

impl Default for GStore {
    fn default() -> Self {
        Self {
            anymap: anymap::Map::new(),
        }
    }
}

impl GStore {
    #[must_use]
    pub fn new_with_graph<N, E, Ix>(g: Graph<N, E, Ix>) -> Self
    where
        N: Clone,
        E: Clone,
        Ix: std::cmp::Eq + Clone + std::hash::Hash,
        Graph<N, E, Ix>: 'static,
    {
        let mut gs = Self::default();
        gs.set_graph(g);
        gs
    }
    pub fn set_graph<N, E, Ix>(&mut self, g: Graph<N, E, Ix>) -> &Self
    where
        N: Clone,
        E: Clone,
        Ix: std::cmp::Eq + Clone + std::hash::Hash,
        Graph<N, E, Ix>: 'static,
    {
        self.anymap.insert(g);
        self
    }
    fn get_graph<N, E, Ix>(&self) -> &Graph<N, E, Ix>
    where
        N: Clone,
        E: Clone,
        Ix: std::cmp::Eq + Clone + std::hash::Hash,
        Graph<N, E, Ix>: 'static,
    {
        self.anymap
            .get::<Graph<N, E, Ix>>()
            .expect("can't get graph")
    }
    fn get_mut_graph<N, E, Ix>(&mut self) -> &mut Graph<N, E, Ix>
    where
        N: Clone,
        E: Clone,
        Ix: std::cmp::Eq + Clone + std::hash::Hash,
        Graph<N, E, Ix>: 'static,
    {
        self.anymap
            .get_mut::<Graph<N, E, Ix>>()
            .expect("can't get graph")
    }
    // fn get_mut_graph<T: Clone + 'static>(&mut self) -> &mut T {
    //     self.anymap.get_mut::<T>().expect("can't get graph")
    // }

    pub fn get_mut_graph_with<'a, F, R, N, E, Ix>(&'a mut self, func: F) -> R
    where
        F: FnOnce(&'a mut Graph<N, E, Ix>) -> R,
        N: Clone,
        E: Clone,
        Ix: std::cmp::Eq + Clone + std::hash::Hash,
        Graph<N, E, Ix>: 'static,
    {
        func(self.get_mut_graph::<N, E, Ix>())
    }
}

#[cfg(test)]
mod graph_store_test {
    use super::*;
    use crate::Layer;

    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    #[allow(dead_code)]
    fn enum_display() {
        enum Message {
            A,
            B,
        }
        let l = GElement::<Message>::Layer_(Layer::new("xx"));
        log::debug!("{}", l);
    }
}
