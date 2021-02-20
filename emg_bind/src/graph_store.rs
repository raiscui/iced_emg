/*
 * @Author: Rais
 * @Date: 2021-01-21 11:05:55
 * @LastEditTime: 2021-02-19 19:51:58
 * @LastEditors: Rais
 * @Description:
 */
pub use emg::Graph;
pub use emg::NodeIndex;
use emg::Outgoing;

use crate::{runtime::Element, runtime::Text, Layer, RefreshFor, RefreshUseFor};
use anymap::any::CloneAny;
use match_any::match_any;
use std::{
    borrow::{Borrow, BorrowMut},
    cell::RefCell,
    convert::{TryFrom, TryInto},
    ops::DerefMut,
    rc::Rc,
};
use std::{hash::Hash, ops::Deref};

use log::Level;

pub type N<'a, Message> = RefCell<GElement<'a, Message>>;
pub type E = String;
pub type GraphType<'a, Message> = Graph<N<'a, Message>, E>;

thread_local! {
    pub static G_STORE: RefCell<GStore> = RefCell::new(
         GStore::default()
    );
}

// impl<'a, T, Message> From<T> for Element<'a, Message>
// where
//     Message: 'static + Clone,
//     T: Into<Element<'a, Message>> + Clone,
// {
//     fn from(can_into_element: &T) -> Element<'a, Message> {
//         can_into_element.clone().into()
//     }
// }

pub use GElement::*;
#[derive(Clone)]
pub enum GElement<'a, Message> {
    Layer_(Layer<'a, Message>),
    Text_(Text),
    Refresher_(Rc<dyn RefreshFor<GElement<'a, Message>>>),
}

impl<'a, Message: std::fmt::Debug> std::fmt::Debug for GElement<'a, Message> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Layer_(l) => f.debug_tuple("GElement::GContainer").field(l).finish(),
            Text_(t) => f.debug_tuple("GElement::Text").field(t).finish(),
            Refresher_(_) => f
                .debug_tuple("GElement::GUpdater(Rc<dyn RtUpdateFor<GElement<'a, Message>>>)")
                .finish(),
        }
    }
}

impl<'a, Message> TryFrom<GElement<'a, Message>> for Element<'a, Message>
where
    Message: 'static + Clone,
{
    type Error = ();

    fn try_from(ge: GElement<'a, Message>) -> Result<Self, Self::Error> {
        // match ge {
        //     Layer_(l) => Ok(l.into()),
        //     Text_(t) => Ok(t.into()),
        //     Refresher_(_) => Err(()),
        // }
        match_any! (ge,
            Layer_(x)|Text_(x) => Ok(x.into()),
            Refresher_(_)=>Err(())
        )
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
    // fn add_el(&mut self, key: Self::Ix, e_item: Self::E, n_item: Self::N) -> NodeIndex<Self::Ix>
    // where
    //     Self::Ix: Clone;

    fn gelement_refresh(
        &self,
        cix: &Self::Ix,
        // current_node: &RefCell<GElement<'a, Message>>,
    ) -> GElement<'a, Message>;

    fn children_to_elements(&self, cix: &Self::Ix) -> Vec<GElement<'a, Message>>;

    fn view(ix: Self::Ix) -> Element<'a, Message>;
}

impl<'a, Message, E, Ix> GraphStore<'a, Message> for Graph<RefCell<GElement<'a, Message>>, E, Ix>
where
    Ix: Clone + Hash + Eq + std::fmt::Debug,
    E: Clone + std::fmt::Debug,
    // N: Clone,
    Self: 'static,
    Message: 'static + Clone + std::fmt::Debug,
{
    type Ix = Ix;
    type N = RefCell<GElement<'a, Message>>;
    type E = E;
    fn init() {
        console_log::init_with_level(Level::Debug).ok();

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

    fn children_to_elements(&self, cix: &Self::Ix) -> Vec<GElement<'a, Message>> {
        self.edges_iter_use_ix(cix, Outgoing)
            .map(|eix| {
                let this_child_ix = eix.ix_dir(Outgoing);
                // let a_child = self.get_node_weight_use_ix(child_ix).unwrap();
                self.gelement_refresh(this_child_ix)
            })
            .collect()
    }

    fn gelement_refresh(
        &self,
        cix: &Self::Ix, // current_node: &RefCell<GElement<'a, Message>>,
    ) -> GElement<'a, Message> {
        let current_node_clone = self.get_node_weight_use_ix(cix).unwrap().clone();

        let children_s = self.children_to_elements(cix);

        for child in children_s {
            current_node_clone
                .borrow_mut()
                .deref_mut()
                .refresh_use(&child)
        }

        current_node_clone.into_inner()
    }

    fn view(cix: Self::Ix) -> Element<'a, Message> {
        G_STORE.with(|g_store_refcell| {
            // g_store_refcell.borrow_mut().set_graph(g);
            g_store_refcell
                .borrow_mut()
                .get_mut_graph_with(|g: &mut Self| {
                    log::info!("graph==> {:#?}", &g);

                    // Rc::make_mut(&mut Rc::clone(rc_e)).clone()
                    // rc_e.clone().into()
                    // Rc::make_mut(rc_e).clone().into()
                    g.gelement_refresh(&cix).try_into().unwrap()
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
