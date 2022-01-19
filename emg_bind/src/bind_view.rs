/*
 * @Author: Rais
 * @Date: 2021-03-16 15:45:57
 * @LastEditTime: 2022-01-19 13:24:54
 * @LastEditors: Rais
 * @Description:
 */
use crate::{Element, GElement, NodeBuilderWidget};
pub use emg::EdgeIndex;
pub use emg::Graph;
pub use emg::NodeIndex;
use emg::{edge_index_no_source, Outgoing};
use emg_core::vector;
use emg_core::IdStr;
use emg_layout::{EPath, EmgEdgeItem, GraphEdgesDict};
use emg_refresh::RefreshForUse;
use emg_state::{CloneStateAnchor, StateAnchor};
use std::{cell::RefCell, convert::TryInto, hash::Hash, ops::DerefMut, rc::Rc};
use tracing::{instrument, trace, trace_span};

// ────────────────────────────────────────────────────────────────────────────────

pub type N<Message> = StateAnchor<GElement<Message>>;
// pub type N< Message> = RefCell<GElement< Message>>;
pub type E<Ix> = EmgEdgeItem<Ix>;
pub type GraphType<Message, Ix = IdStr> = Graph<N<Message>, E<Ix>, Ix>;

pub trait GraphView<Message> {
    type N;
    type Ix: std::fmt::Debug + std::fmt::Display;
    type E;

    fn gelement_refresh_and_comb(
        &self,
        edges: &GraphEdgesDict<Self::Ix>,
        cix: &Self::Ix,
        paths: &EPath<Self::Ix>,
        // opt_parent_e: Option<Self::E>,
        // opt_eix: Option<&EdgeIndex<Ix>>,
        // current_node: &RefCell<GElement< Message>>,
    ) -> Rc<RefCell<GElement<Message>>>
    where
        // <Self as GraphView<Message>>::Ix: Clone + Hash + Eq + Ord + Default;
        Self::Ix: Clone + Hash + Eq + Ord + Default;

    fn children_to_elements(
        &self,
        edges: &GraphEdgesDict<Self::Ix>,
        cix: &Self::Ix,
        paths: &EPath<Self::Ix>,
    ) -> Vec<Rc<RefCell<GElement<Message>>>>
    where
        // <Self as GraphView<Message>>::Ix: Clone + Hash + Eq + Ord + Default;
        Self::Ix: Clone + Hash + Eq + Ord + Default;

    fn view(&self, into_ix: impl Into<Self::Ix>) -> Element<Message>;
    // fn global_view(ix: Ix) -> Element< Message>;
}

// impl<Message> GraphView<Message> for GraphType<Message>
impl<Message> GraphView<Message> for GraphType<Message>
where
    Message: 'static + Clone + std::fmt::Debug,
{
    type Ix = IdStr;
    type N = N<Message>;
    type E = E<IdStr>;

    // #[instrument(skip(self, edges))]
    fn gelement_refresh_and_comb(
        &self,
        edges: &GraphEdgesDict<Self::Ix>,
        cix: &Self::Ix,
        paths: &EPath<Self::Ix>,
        // edge_for_cix: &Edge<Self::E, Ix>,
        // current_node: &RefCell<GElement< Message>>,
    ) -> Rc<RefCell<GElement<Message>>> {
        // debug!("run here 01");
        //TODO has no drop clone for AnimationE inside,need bumpalo do drop
        let current_node_clone = Rc::new(RefCell::new(
            //TODO maybe no need rc RefCell
            self.get_node_weight_use_ix(cix).unwrap().get(),
        )); //TODO cache
            // debug!("run here 01.1");

        let mut children_s = self.children_to_elements(edges, cix, paths);

        let event_callbacks = children_s
            .drain_filter(|gel| gel.borrow().is_event_())
            .collect::<Vec<_>>();

        //make node_ref real

        //TODO link node use refresh_for_use
        //NOTE NodeRef_ 处理
        children_s
            .iter()
            .filter(|gel| gel.borrow().is_node_ref_())
            .for_each(|gel| {
                gel.replace_with(|gelmut| {
                    let node = gelmut
                        .as_node_ref_()
                        .and_then(|str| self.get_node_weight_use_ix(str))
                        .cloned()
                        .expect("expect get node id");

                    //TODO use cow 因为 children_s 只是刷新current_node_clone
                    node.get()
                });
            });
        //TODO edge gel 一起 refresh?
        // The const / dyn child node performs the change
        // TODO: cache.    use edge type?
        for child in &children_s {
            //  TODO use COW
            current_node_clone
                .borrow_mut()
                .deref_mut()
                .refresh_for_use(&*child.borrow());
        }
        if let Ok(mut node_builder_widget) =
            NodeBuilderWidget::<Message>::try_new_use(&*current_node_clone.clone().borrow())
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
                    for event_callback in event_callbacks {
                        //TODO maybe just directly push event
                        node_builder_widget.refresh_for_use(&*event_callback.borrow());
                    }
                }

                Rc::new(RefCell::new(GElement::Builder_(
                    current_node_clone,
                    node_builder_widget,
                )))
            }
        } else {
            trace!(
                "NodeBuilderWidget::<Message>::try_from  error use: {}",
                current_node_clone.borrow()
            );
            current_node_clone
        }
    }

    #[instrument(skip(self, edges))]
    fn children_to_elements(
        &self,
        edges: &GraphEdgesDict<Self::Ix>,
        cix: &Self::Ix,
        paths: &EPath<Self::Ix>,
    ) -> Vec<Rc<RefCell<GElement<Message>>>> {
        self.edges_iter(cix, Outgoing)
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

    fn view(&self, into_ix: impl Into<Self::Ix>) -> Element<Message> {
        let cix: Self::Ix = into_ix.into();
        let _g = trace_span!("graph view-", ?cix);
        {
            let edges = self.raw_edges().store_get_rc(&self.store());
            let paths = EPath::new(vector![edge_index_no_source(cix.clone())]);
            // TODO add store in gelement_refresh_and_comb
            let gel = self.gelement_refresh_and_comb(&edges, &cix, &paths);
            gel.replace(GElement::EmptyNeverUse).try_into().unwrap()
        }
    }

    // fn global_view(cix: Ix) -> Element< Message> {
    //     G_STORE.with(|g_store_refcell| {
    //         // g_store_refcell.borrow_mut().set_graph(g);
    //         g_store_refcell
    //             .borrow()
    //             .get_graph::<Self::N, Self::E, Ix>()
    //             .gelement_comb_and_refresh(&cix)
    //             .try_into()
    //             .unwrap()
    //     })
    // }
}
