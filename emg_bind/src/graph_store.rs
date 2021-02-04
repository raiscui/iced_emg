/*
 * @Author: Rais
 * @Date: 2021-01-21 11:05:55
 * @LastEditTime: 2021-02-04 11:48:04
 * @LastEditors: Rais
 * @Description:
 */
pub use emg::Graph;
pub use emg::NodeIndex;

use crate::{runtime::Element, runtime::Widget, Layer};

use anymap::any::CloneAny;
use std::hash::Hash;
use std::{cell::RefCell, rc::Rc};

thread_local! {
    pub static G_STORE: RefCell<GStore> = RefCell::new(
         GStore::default()
    );
}

pub use GElement::{GContainer, GSurface};
#[derive(Clone, Debug)]
pub enum GElement<'a, Message> {
    GContainer(Layer<'a, Message>),
    GSurface(Element<'a, Message>),
}

impl<'a, Message> Into<Element<'a, Message>> for GElement<'a, Message>
where
    Message: 'static + Clone,
{
    fn into(self) -> Element<'a, Message> {
        match self {
            GElement::GContainer(l) => l.into(),

            GElement::GSurface(e) => e,
        }
    }
}

#[derive(Clone, Debug)]
pub struct GStore {
    pub anymap: anymap::Map<dyn CloneAny>,
    // pub graph: Graph<N, E, Ix>,
}

pub trait GraphStore<'a, Message> {
    type N;
    type Ix;
    type E;
    fn init();
    fn get_mut_graph_with<F: FnOnce(&mut Self) -> R, R>(func: F) -> R;
    fn add_el(&mut self, key: Self::Ix, e_item: Self::E, n_item: Self::N) -> NodeIndex<Self::Ix>
    where
        Self::Ix: Clone;
    fn view(ix: Self::Ix) -> Element<'a, Message>;
}

impl<'a, Message, E, Ix> GraphStore<'a, Message> for Graph<Rc<GElement<'a, Message>>, E, Ix>
where
    Ix: Clone + Hash + Eq + std::fmt::Debug,
    E: Clone,
    // N: Clone,
    Self: 'static,
    Message: 'static + Clone,
{
    type Ix = Ix;
    type N = Rc<GElement<'a, Message>>;
    type E = E;
    fn init() {
        G_STORE.with(|g_store_refcell| {
            // g_store_refcell.borrow_mut().set_graph(g);
            g_store_refcell.borrow_mut().set_graph(Self::default());
        });
    }

    fn get_mut_graph_with<F, R>(func: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        G_STORE.with(|g_store_refcell| {
            // g_store_refcell.borrow_mut().set_graph(g);
            g_store_refcell.borrow_mut().get_mut_graph_with(func)
        })
    }

    fn add_el(&mut self, key: Self::Ix, e_item: Self::E, n_item: Self::N) -> NodeIndex<Self::Ix> {
        match illicit::get::<NodeIndex<Self::Ix>>() {
            Ok(p_nix) => {
                let nix = self.insert_node(key, n_item);
                self.insert_update_edge(&*p_nix, &nix, e_item);
                nix
            }
            Err(_) => self.insert_node(key, n_item),
        }
    }

    fn view(ix: Self::Ix) -> Element<'a, Message> {
        G_STORE.with(|g_store_refcell| {
            // g_store_refcell.borrow_mut().set_graph(g);
            g_store_refcell
                .borrow_mut()
                .get_mut_graph_with(|g: &mut Self| {
                    let rc_e = g.get_mut_node_weight_use_ix(ix).unwrap();
                    // Rc::make_mut(&mut Rc::clone(rc_e)).clone()
                    // rc_e.clone().into()
                    Rc::make_mut(rc_e).clone().into()
                })
        })
    }
}

impl Default for GStore {
    fn default() -> Self {
        GStore {
            anymap: anymap::Map::new(),
        }
    }
}

impl GStore {
    pub fn new_with_graph<N, E, Ix>(g: Graph<N, E, Ix>) -> Self
    where
        N: Clone,
        E: Clone,
        Ix: std::cmp::Eq + Clone + std::hash::Hash,
        Graph<N, E, Ix>: 'static,
    {
        let mut gs = GStore::default();
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
    fn get_graph<N, E, Ix>(&mut self) -> &Graph<N, E, Ix>
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
