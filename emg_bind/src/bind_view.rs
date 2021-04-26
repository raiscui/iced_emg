/*
 * @Author: Rais
 * @Date: 2021-03-16 15:45:57
 * @LastEditTime: 2021-04-25 15:24:47
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
use emg_state::{state_store_with, GStateStore};
use std::{
    cell::Ref,
    convert::{TryFrom, TryInto},
    hash::Hash,
};
use tracing::{error, instrument, trace, trace_span, warn};
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
        store: &Ref<GStateStore>,
        edges: &GraphEdgesDict<Self::Ix>,
        cix: &Self::Ix,
        paths: &EPath<Self::Ix>,
        // opt_parent_e: Option<Self::E>,
        // opt_eix: Option<&EdgeIndex<Self::Ix>>,
        // current_node: &RefCell<GElement<'a, Message>>,
    ) -> GElement<'a, Message>
    where
        <Self as GraphView<'a, Message>>::Ix: Clone + Hash + Eq + Ord + Default;

    fn children_to_elements(
        &self,
        store: &Ref<GStateStore>,
        edges: &GraphEdgesDict<Self::Ix>,
        cix: &Self::Ix,
        paths: &EPath<Self::Ix>,
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

    #[instrument(skip(self, store, edges))]
    fn gelement_refresh_and_comb(
        &self,
        store: &Ref<GStateStore>,
        edges: &GraphEdgesDict<Self::Ix>,
        cix: &Self::Ix,
        paths: &EPath<Self::Ix>,
        // edge_for_cix: &Edge<Self::E, Self::Ix>,
        // current_node: &RefCell<GElement<'a, Message>>,
    ) -> GElement<'a, Message> {
        let mut current_node_clone = self.get_node_weight_use_ix(cix).unwrap().clone(); //TODO cache

        let mut children_s = self.children_to_elements(store, edges, cix, paths);

        let event_callbacks = children_s
            .drain_filter(|gel| gel.is_event_())
            .collect::<Vec<_>>();

        // The const / dyn child node performs the change
        // TODO: cache.    use edge type?
        for child in children_s {
            //  TODO use COW
            current_node_clone.refresh_use(&child)
        }

        match NodeBuilderWidget::<Message>::try_from(current_node_clone) {
            Ok(mut node_builder_widget) => {
                let _g = trace_span!("-> in NodeBuilderWidget").entered();
                {
                    trace!("NodeBuilderWidget::<Message>::try_from  OK");

                    let ei = &edges.get(paths.back().unwrap()).unwrap().item;

                    let ed = ei.store_get_edge_data(store, paths).unwrap();
                    let edge_styles = ed.store_styles_string(store);
                    warn!("styles---------------> {}", &edge_styles);

                    node_builder_widget.add_styles_string(edge_styles.as_str());

                    if !event_callbacks.is_empty() {
                        for event_callback in event_callbacks {
                            //TODO maybe just directly push event
                            node_builder_widget.refresh_use(&event_callback)
                        }
                    }

                    GElement::Element_(node_builder_widget.into())
                }
            }
            Err(old_gel) => {
                error!(
                    "NodeBuilderWidget::<Message>::try_from  error use: {}",
                    old_gel
                );
                old_gel
            }
        }
    }

    #[instrument(skip(self, store, edges))]
    fn children_to_elements(
        &self,
        store: &Ref<GStateStore>,
        edges: &GraphEdgesDict<Self::Ix>,
        cix: &Self::Ix,
        paths: &EPath<Self::Ix>,
    ) -> Vec<GElement<'a, Message>> {
        self.edges_iter_use_ix(cix, Outgoing)
            .filter_map(|eix| {
                let opt_this_child_nix = eix.nix_by_dir(Outgoing).as_ref();

                opt_this_child_nix.map(|this_child_nix| {
                    let mut new_paths = paths.clone();
                    new_paths.set_with(|ev| ev.push_back(eix.clone()));

                    self.gelement_refresh_and_comb(store, edges, this_child_nix.index(), &new_paths)
                })
            })
            .collect() //TODO use iter
    }

    #[instrument(skip(self))]
    fn view(&self, cix: Self::Ix) -> Element<'a, Message> {
        let _g = trace_span!("ffffffffffffffffffffffff", ?cix);
        state_store_with(|store| {
            let edges = self.raw_edges().store_get_rc(&store);
            let paths: EPath<Self::Ix> = EPath::new(vector![edge_index_no_source(cix.clone())]);
            // TODO add store in gelement_refresh_and_comb
            self.gelement_refresh_and_comb(&store, &edges, &cix, &paths)
                .try_into()
                .unwrap()
        })
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
