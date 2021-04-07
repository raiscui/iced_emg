/*
 * @Author: Rais
 * @Date: 2021-03-16 15:45:57
 * @LastEditTime: 2021-04-05 10:46:09
 * @LastEditors: Rais
 * @Description:
 */
use crate::{runtime::Element, GElement, NodeBuilderWidget};
pub use emg::EdgeIndex;
pub use emg::Graph;
pub use emg::NodeIndex;
use emg::{Edge, Outgoing};
use emg_layout::{EdgeData, EdgeData, EdgeItemNode};
use emg_refresh::RefreshUseFor;
use emg_state::StateVar;
use std::{
    convert::{TryFrom, TryInto},
    hash::Hash,
};
// ────────────────────────────────────────────────────────────────────────────────

pub type N<'a, Message> = GElement<'a, Message>;
// pub type N<'a, Message> = RefCell<GElement<'a, Message>>;
pub type E = EdgeItemNode;
pub type GraphType<'a, Message> = Graph<N<'a, Message>, E>;

pub trait GraphView<'a, Message> {
    type N;
    type Ix;
    type E;

    fn gelement_refresh_and_comb(
        &self,
        cix: &Self::Ix,
        opt_parent_e: Option<Self::E>,
        opt_eix: Option<&EdgeIndex<Self::Ix>>,
        // current_node: &RefCell<GElement<'a, Message>>,
    ) -> GElement<'a, Message>
    where
        <Self as GraphView<'a, Message>>::Ix: Clone + Hash + Eq;

    fn children_to_elements(&self, cix: &Self::Ix) -> Vec<GElement<'a, Message>>;

    fn view(&self, ix: Self::Ix) -> Element<'a, Message>;
    // fn global_view(ix: Self::Ix) -> Element<'a, Message>;
}

impl<'a, Message, Ix> GraphView<'a, Message> for Graph<N<'a, Message>, E, Ix>
where
    Ix: Clone + Hash + Eq + std::fmt::Debug,
    // E: Clone + std::fmt::Debug,
    Message: 'static + Clone + std::fmt::Debug,
{
    type Ix = Ix;
    type N = N<'a, Message>;
    type E = E;

    fn gelement_refresh_and_comb(
        &self,
        cix: &Self::Ix,
        opt_parent_e: Option<Self::E>,
        edge_for_cix: &Edge<Self::E, Self::Ix>,
        // current_node: &RefCell<GElement<'a, Message>>,
    ) -> GElement<'a, Message> {
        // buildingTime original GElement
        let opt_edge_item = edge_for_cix.
        if let Some(e) = opt_edge_item {
            match e {
                EdgeItemNode::EdgeData(ed) => {
                    let layout = &mut ed.layout;
                    //TODO more humanization
                    let parent_e = opt_parent_e.unwrap();
                    let pwe
                }
                EdgeItemNode::String(_) => {}
                EdgeItemNode::Empty => {}
            }
        }
        let mut current_node_clone = self.get_node_weight_use_ix(cix).unwrap().clone();

        let mut children_s = self.children_to_elements(cix);

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

    fn children_to_elements(&self, cix: &Self::Ix) -> Vec<GElement<'a, Message>> {
        self.edges_iter_use_ix(cix, Outgoing)
            .map(|eix| {
                let this_child_ix = eix.ix_by_dir(Outgoing);
                // let a_child = self.get_node_weight_use_ix(child_ix).unwrap();
                self.gelement_refresh_and_comb(this_child_ix, Some(eix))
            })
            .collect()
    }

    fn view(&self, cix: Self::Ix) -> Element<'a, Message> {
        let root_edge = self.root_edge().as_ref().unwrap();
        //TODO: get cix get  and edgeitem
        self.gelement_refresh_and_comb(&cix, None)
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
