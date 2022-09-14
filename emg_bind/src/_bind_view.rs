/*
 * @Author: Rais
 * @Date: 2021-03-16 15:45:57
 * @LastEditTime: 2022-08-10 10:20:40
 * @LastEditors: Rais
 * @Description:
 */
use crate::{GElement, NodeBuilderWidget};
pub use emg::EdgeIndex;
pub use emg::Graph;
pub use emg::NodeIndex;
use emg::{edge_index_no_source, Node, Outgoing};
use emg_common::vector;
use emg_common::IdStr;
use emg_layout::{EPath, EmgEdgeItem, GraphEdgesDict};
use emg_shaping::ShapeOfUse;
use emg_state::{CloneStateAnchor, StateAnchor};
use std::hash::Hash;
use tracing::{instrument, trace, trace_span};

// ────────────────────────────────────────────────────────────────────────────────

type N<Message> = StateAnchor<GElement<Message>>;
// pub type N< Message> = RefCell<GElement< Message>>;
type E<Ix> = EmgEdgeItem<Ix>;
type GraphType<Message, Ix = IdStr> = Graph<N<Message>, E<Ix>, Ix>;

pub trait GraphView {
    type N;
    type Ix: std::fmt::Debug + std::fmt::Display;
    type E;
    type Message: PartialEq + Clone;

    fn gelement_refresh_and_comb(
        &self,
        edges: &GraphEdgesDict<Self::Ix>,
        cix: &Self::Ix,
        paths: &EPath<Self::Ix>,
        // opt_parent_e: Option<Self::E>,
        // opt_eix: Option<&EdgeIndex<Ix>>,
        // current_node: &RefCell<GElement< Message>>,
    ) -> GElement<Self::Message>
    where
        // <Self as GraphView<Message>>::Ix: Clone + Hash + Eq + Ord + Default;
        Self::Ix: Clone + Hash + Eq + Ord + Default;

    fn children_to_elements(
        &self,
        node: &Node<Self::N, Self::Ix>,
        edges: &GraphEdgesDict<Self::Ix>,
        cix: &Self::Ix,
        paths: &EPath<Self::Ix>,
    ) -> Vec<GElement<Self::Message>>
    where
        // <Self as GraphView<Message>>::Ix: Clone + Hash + Eq + Ord + Default;
        Self::Ix: Clone + Hash + Eq + Ord + Default;

    fn view(&self, into_ix: impl Into<Self::Ix>) -> GElement<Self::Message>;
}

// impl<Message> GraphView<Message> for GraphType<Message>
impl<Message> GraphView for GraphType<Message>
where
    Message: 'static + Clone + std::fmt::Debug + std::cmp::PartialEq,
{
    type Ix = IdStr;
    type E = E<Self::Ix>;
    type Message = Message;
    type N = N<Self::Message>;

    // #[instrument(skip(self, edges))]
    fn gelement_refresh_and_comb(
        &self,
        edges: &GraphEdgesDict<Self::Ix>,
        cix: &Self::Ix,
        paths: &EPath<Self::Ix>,
        // edge_for_cix: &Edge<Self::E, Ix>,
        // current_node: &RefCell<GElement< Message>>,
    ) -> GElement<Self::Message> {
        // debug!("run here 01");
        //TODO has no drop clone for AnimationE inside,need bumpalo do drop
        let node: &Node<Self::N, Self::Ix> = self.get_node_use_ix(cix).unwrap();
        let mut current_node_item_clone = node.item.get(); //TODO cache
                                                           // debug!("run here 01.1");

        let mut children_s = self.children_to_elements(node, edges, cix, paths);

        let event_callbacks = children_s
            .drain_filter(|gel| gel.is_event_())
            .collect::<Vec<_>>();

        //make node_ref real

        //TODO link node use shape_of_use
        //NOTE NodeRef_ 处理
        children_s
            .iter_mut()
            .filter(|gel| gel.is_node_ref_())
            .for_each(|gel| {
                *gel = gel
                    .as_node_ref_()
                    .and_then(|str| self.get_node_item_use_ix(str))
                    .cloned()
                    .expect("expect get node id")
                    .get();
            });
        //TODO edge gel 一起 refresh?
        // The const / dyn child node performs the change
        // TODO: cache.    use edge type?
        //TODO illicit::Layer path
        for child in &children_s {
            //  TODO use COW
            current_node_item_clone.shape_of_use(child);
        }
        if let Ok(mut node_builder_widget) =
            NodeBuilderWidget::<Message>::try_new_use(&current_node_item_clone)
        {
            let _g = trace_span!("-> in NodeBuilderWidget").entered();
            {
                trace!("NodeBuilderWidget::<Message>::try_from  OK");
                // node_builder_widget.set_id(format!("{}", cix));
                node_builder_widget.set_id(cix.clone());

                let ei = &edges.get(paths.last().unwrap()).unwrap().item;

                let store = self.store();

                //TODO use StateAnchor ? for child edge change
                trace!("edge::path:  {}", &paths);
                let edge_styles = {
                    let ed = ei.store_edge_data_with(&store, paths, |ed| {
                        ed.unwrap_or_else(|| panic!("not find EdgeData for path:{}", &paths))
                            .clone()
                    });
                    ed.store_styles_string(&store)
                };

                trace!("styles---------------> {}", &edge_styles);

                node_builder_widget.add_styles_string(edge_styles.as_str());

                if !event_callbacks.is_empty() {
                    for event_callback in &event_callbacks {
                        //TODO maybe just directly push event
                        node_builder_widget.shape_of_use(event_callback);
                    }
                }

                GElement::Builder_(node_builder_widget.and_widget(current_node_item_clone))
            }
        } else {
            trace!(
                "NodeBuilderWidget::<Message>::try_from  error use:",
                // current_node_clone.borrow()
            );
            current_node_item_clone
        }
    }

    #[instrument(skip(self, edges))]
    fn children_to_elements(
        &self,
        node: &Node<Self::N, Self::Ix>,
        edges: &GraphEdgesDict<Self::Ix>,
        cix: &Self::Ix,
        paths: &EPath<Self::Ix>,
    ) -> Vec<GElement<Message>> {
        node.edge_out_ixs()
            .as_ref()
            .iter()
            // self.edges_consuming_iter(cix, Outgoing)
            .filter_map(|eix| {
                let opt_this_child_nix = eix.nix_by_dir(Outgoing).as_ref();

                opt_this_child_nix.map(|this_child_nix| {
                    let mut new_paths = paths.clone();
                    new_paths.push_back(eix.clone());

                    self.gelement_refresh_and_comb(edges, this_child_nix.index(), &new_paths)
                })
            })
            .collect() //TODO use iter
    }

    fn view(&self, into_ix: impl Into<Self::Ix>) -> GElement<Self::Message> {
        let cix: Self::Ix = into_ix.into();
        let _g = trace_span!("graph view-", ?cix);
        {
            let edges = self.raw_edges().store_get_rc(&self.store());
            let paths = EPath::<IdStr>::new(vector![edge_index_no_source(cix.clone())]);
            // TODO add store in gelement_refresh_and_comb

            self.gelement_refresh_and_comb(&edges, &cix, &paths)
        }
    }
}
