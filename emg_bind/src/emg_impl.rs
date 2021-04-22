/*
 * @Author: Rais
 * @Date: 2021-03-16 15:45:57
 * @LastEditTime: 2021-04-22 14:36:37
 * @LastEditors: Rais
 * @Description:
 */
use crate::{runtime::Element, GElement, NodeBuilderWidget};
pub use emg::EdgeIndex;
pub use emg::Graph;
pub use emg::NodeIndex;
use emg::{edge_index_no_source, im::vector, Outgoing};
use emg_layout::{EPath, EmgEdgeItem, GraphEdgesDict};
use emg_refresh::RefreshUseFor;
use emg_state::CloneStateVar;
use std::{
    convert::{TryFrom, TryInto},
    hash::Hash,
};
// ────────────────────────────────────────────────────────────────────────────────

pub type N<'a, Message> = GElement<'a, Message>;
// pub type N<'a, Message> = RefCell<GElement<'a, Message>>;
pub type E<Ix> = EmgEdgeItem<Ix>;
pub type GraphType<'a, Message, Ix = String> = Graph<N<'a, Message>, E<Ix>, Ix>;

pub trait GraphView<'a, Message> {
    type N;
    type Ix;
    type E;

    fn gelement_refresh_and_comb(
        &self,
        edges: &GraphEdgesDict<Self::Ix>,
        cix: &Self::Ix,
        paths: EPath<Self::Ix>,
        // opt_parent_e: Option<Self::E>,
        // opt_eix: Option<&EdgeIndex<Self::Ix>>,
        // current_node: &RefCell<GElement<'a, Message>>,
    ) -> GElement<'a, Message>
    where
        <Self as GraphView<'a, Message>>::Ix: Clone + Hash + Eq + Ord + Default;

    fn children_to_elements(
        &self,
        edges: &GraphEdgesDict<Self::Ix>,
        cix: &Self::Ix,
        paths: EPath<Self::Ix>,
    ) -> Vec<GElement<'a, Message>>
    where
        <Self as GraphView<'a, Message>>::Ix: Clone + Hash + Eq + Ord + Default;

    fn view(&self, ix: Self::Ix) -> Element<'a, Message>;
    // fn global_view(ix: Self::Ix) -> Element<'a, Message>;
}

impl<'a, Message, Ix> GraphView<'a, Message> for Graph<N<'a, Message>, E<Ix>, Ix>
where
    Ix: Clone + Hash + Eq + std::fmt::Debug + Ord + Default,
    // E: Clone + std::fmt::Debug,
    Message: 'static + Clone + std::fmt::Debug,
{
    type Ix = Ix;
    type N = N<'a, Message>;
    type E = E<Ix>;

    fn gelement_refresh_and_comb(
        &self,
        edges: &GraphEdgesDict<Self::Ix>,
        cix: &Self::Ix,
        paths: EPath<Self::Ix>,
        // edge_for_cix: &Edge<Self::E, Self::Ix>,
        // current_node: &RefCell<GElement<'a, Message>>,
    ) -> GElement<'a, Message> {
        // buildingTime original GElement
        // ────────────────────────────────────────────────────────────────────────────────

        let ei = &edges.get(paths.back().unwrap()).unwrap().item;

        let ed = ei.get_edge_data(&paths).unwrap();
        let x = ed.styles_string();
        log::warn!("styles---------------> {}", &x);
        // ────────────────────────────────────────────────────────────────────────────────

        let mut current_node_clone = self.get_node_weight_use_ix(cix).unwrap().clone(); //TODO cache

        let mut children_s = self.children_to_elements(edges, cix, paths.clone());

        let event_callbacks = children_s
            .drain_filter(|gel| gel.is_event_())
            .collect::<Vec<_>>();

        // The const / dyn child node performs the change
        // TODO: cache.    use edge type?
        for child in children_s {
            //  TODO use COW
            current_node_clone.refresh_use(&child)
        }

        // event_callback handle -----------------------
        if event_callbacks.is_empty() {
            // if event_callbacks.is_empty() {
            current_node_clone
        } else {
            log::debug!("event_callback is not empty");
            match NodeBuilderWidget::<Message>::try_from(current_node_clone) {
                Ok(mut node_builder_widget) => {
                    log::debug!("NodeBuilderWidget::<Message>::try_from  OK");

                    for event_callback in event_callbacks {
                        node_builder_widget.refresh_use(&event_callback)
                    }
                    GElement::Element_(node_builder_widget.into())
                }
                Err(old_gel) => {
                    log::error!(
                        "NodeBuilderWidget::<Message>::try_from  error use: {}",
                        old_gel
                    );
                    old_gel
                }
            }
        }
    }

    fn children_to_elements(
        &self,
        edges: &GraphEdgesDict<Self::Ix>,
        cix: &Self::Ix,
        paths: EPath<Self::Ix>,
    ) -> Vec<GElement<'a, Message>> {
        self.edges_iter_use_ix(cix, Outgoing)
            .filter_map(|eix| {
                let opt_this_child_nix = eix.nix_by_dir(Outgoing).as_ref();

                opt_this_child_nix.map(|this_child_nix| {
                    let mut new_paths = paths.clone();
                    new_paths.set_with(|ev| ev.push_back(eix.clone()));

                    self.gelement_refresh_and_comb(edges, this_child_nix.index(), new_paths)
                })
            })
            .collect() //TODO use iter
    }

    fn view(&self, cix: Self::Ix) -> Element<'a, Message> {
        //TODO: get cix get  and edgeitem
        let edges = self.get_raw_edges().get();
        let paths: EPath<Self::Ix> = EPath::new(vector![edge_index_no_source(cix.clone())]);

        self.gelement_refresh_and_comb(&edges, &cix, paths)
            .try_into()
            .unwrap()
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
