/*
 * @Author: Rais
 * @Date: 2022-08-18 17:58:00
 * @LastEditTime: 2023-01-30 18:38:35
 * @LastEditors: Rais
 * @Description:
 */

use crate::{
    error::Error,
    g_node::{EmgNodeItem, GelType, GraphType, NItem},
    g_tree_builder::{GTreeBuilderElement, GTreeBuilderFn},
    widget::Layer,
    GElement,
};
use emg::{edge_index_no_source, node_index, Edge, EdgeCollect, EdgeIndex, NodeIndex};

use emg_common::{im::vector, IdStr};
use emg_hasher::CustomHasher;
use emg_layout::{global_height, global_width, EPath, EmgEdgeItem, GenericSizeAnchor};
use emg_shaping::ShapeOfUse;
use emg_state::{
    topo::{self, call_in_slot},
    use_state,
    use_state_impl::TopoKey,
    CloneStateVar, StateAnchor,
};
use indexmap::IndexSet;
use std::{
    cell::{Ref, RefCell, RefMut},
    hash::BuildHasherDefault,
    rc::Rc,
};
use tracing::{debug, info, info_span, instrument, trace, trace_span, warn};

pub struct GraphNodeBuilder<Message, Ix = IdStr>
where
    Message: 'static,
    Ix: std::clone::Clone + std::hash::Hash + std::cmp::Ord + std::default::Default + 'static,
{
    ix: Ix,
    opt_gel_sa: Option<NItem<Message>>,
    opt_incoming_eix_set: Option<EdgeCollect<Ix>>,
    opt_outgoing_eix_set: Option<EdgeCollect<Ix>>,
}

impl<Message> GraphNodeBuilder<Message>
where
    Message: 'static,
    // Ix: std::clone::Clone + std::hash::Hash + std::cmp::Ord + std::default::Default + 'static,
{
    pub fn new(ix: IdStr) -> Self {
        Self {
            ix,
            opt_gel_sa: None,
            opt_incoming_eix_set: None,
            opt_outgoing_eix_set: None,
        }
    }

    #[allow(clippy::missing_const_for_fn)]
    pub fn with_gel_sa(mut self, gel_state: NItem<Message>) -> Self {
        self.opt_gel_sa = Some(gel_state);
        self
    }

    #[allow(clippy::missing_const_for_fn)]
    pub fn with_incoming_eix_set(mut self, incoming_eix_set: EdgeCollect<IdStr>) -> Self {
        self.opt_incoming_eix_set = Some(incoming_eix_set);
        self
    }

    #[allow(clippy::missing_const_for_fn)]
    pub fn with_outgoing_eix_set(mut self, outgoing_eix_set: EdgeCollect<IdStr>) -> Self {
        self.opt_outgoing_eix_set = Some(outgoing_eix_set);
        self
    }
    #[allow(clippy::missing_const_for_fn)]
    pub fn with_outgoing_eix_set_with_default_capacity(mut self, capacity: usize) -> Self {
        self.opt_outgoing_eix_set = Some(IndexSet::with_capacity_and_hasher(
            capacity,
            BuildHasherDefault::<CustomHasher>::default(),
        ));
        self
    }
    #[allow(clippy::missing_const_for_fn)]
    pub fn with_outgoing_eix_set_with_default(mut self) -> Self {
        self.opt_outgoing_eix_set = Some(IndexSet::with_hasher(
            BuildHasherDefault::<CustomHasher>::default(),
        ));
        self
    }
    #[topo::nested]
    pub fn build_in_topo(self, graph_rc: &Rc<RefCell<GraphType<Message, IdStr>>>) {
        let incoming_eix_set = use_state(self.opt_incoming_eix_set.unwrap());
        let outgoing_eix_set = use_state(self.opt_outgoing_eix_set.unwrap());

        let node_item = EmgNodeItem::<NItem<Message>, GelType<Message>>::new(
            self.ix.clone(),
            self.opt_gel_sa.unwrap(),
            &incoming_eix_set.watch(),
            &outgoing_eix_set.watch(),
            graph_rc.clone(),
        );
        //TODO all use or_insert_node? 目前没有使用 or_insert_node , same key 会覆盖
        graph_rc.borrow_mut().or_insert_node_with_plugs(
            self.ix,
            node_item,
            incoming_eix_set,
            outgoing_eix_set,
        );
    }
}

pub struct GraphEdgeBuilder<Ix = IdStr>
where
    Ix: std::clone::Clone + std::hash::Hash + std::cmp::Ord + std::default::Default + 'static,
{
    edge_ix: EdgeIndex<Ix>,
    opt_size: Option<(GenericSizeAnchor, GenericSizeAnchor)>,
    opt_origin: Option<(GenericSizeAnchor, GenericSizeAnchor, GenericSizeAnchor)>,
    opt_align: Option<(GenericSizeAnchor, GenericSizeAnchor, GenericSizeAnchor)>,
}

impl GraphEdgeBuilder {
    pub fn new(edge_ix: EdgeIndex<IdStr>) -> Self {
        Self {
            edge_ix,
            opt_size: None,
            opt_origin: None,
            opt_align: None,
        }
    }

    pub fn with_size(
        mut self,
        (w, h): (impl Into<GenericSizeAnchor>, impl Into<GenericSizeAnchor>),
    ) -> Self {
        self.opt_size = Some((w.into(), h.into()));
        self
    }
    pub fn with_origin(
        mut self,
        (origin_x, origin_y, origin_z): (
            impl Into<GenericSizeAnchor>,
            impl Into<GenericSizeAnchor>,
            impl Into<GenericSizeAnchor>,
        ),
    ) -> Self {
        self.opt_origin = Some((origin_x.into(), origin_y.into(), origin_z.into()));
        self
    }
    pub fn with_align(
        mut self,
        (align_x, align_y, align_z): (
            impl Into<GenericSizeAnchor>,
            impl Into<GenericSizeAnchor>,
            impl Into<GenericSizeAnchor>,
        ),
    ) -> Self {
        self.opt_align = Some((align_x.into(), align_y.into(), align_z.into()));
        self
    }

    #[topo::nested]
    pub fn build_in_topo<Message>(
        self,
        graph_rc: &Rc<RefCell<GraphType<Message, IdStr>>>,
    ) -> Result<EmgEdgeItem<IdStr>, Error> {
        let mut g = graph_rc.borrow_mut();
        let size = self.opt_size.unwrap_or_default();
        let origin = self.opt_origin.unwrap_or_default();
        let align = self.opt_align.unwrap_or_default();
        // ─────────────────────────────────────────────────────────────

        g.nodes_connect_eix(&self.edge_ix)
            .ok_or(Error::EdgeIndexNotExistInNode)?;
        // ─────────────────────────────────────────────────────
        let source = use_state(self.edge_ix.source_nix().clone());
        let target = use_state(self.edge_ix.target_nix().clone());
        // ─────────────────────────────────────────────────────────────
        let edge_item = EmgEdgeItem::new_in_topo(
            source.watch(),
            target.watch(),
            g.get_raw_edges_watch(),
            size,
            origin,
            align,
        );
        // ─────────────────────────────────────────────────────────────

        g.just_insert_edge(self.edge_ix, Edge::new(source, target, edge_item.clone()));

        Ok(edge_item)
    }
}

//TODO make trait use GraphNodeBuilder GraphNodeBuilder for Rc<RefCell<GraphType<Message>>> and GraphType<Message> for easy use in update

impl<Message> GTreeBuilderFn<Message> for Rc<RefCell<GraphType<Message>>>
// where
//     Message: std::clone::Clone + std::cmp::PartialEq + std::fmt::Debug,
{
    type Ix = IdStr;
    type GraphType = GraphType<Message>;
    type RcRefCellGraphType = Self;

    fn rc_refcell_self(&self) -> Self::RcRefCellGraphType {
        self.clone()
    }

    fn graph(&self) -> Ref<GraphType<Message>> {
        self.borrow()
    }
    fn graph_mut(&mut self) -> RefMut<GraphType<Message>> {
        self.borrow_mut()
    }

    #[topo::nested]
    fn handle_root_in_topo(&self, tree_element: &GTreeBuilderElement<Message>) {
        match tree_element {
            GTreeBuilderElement::Layer(root_id, edge_shapers, children_list) => {
                let _span = trace_span!("=> handle_root [layer] ",%root_id).entered();
                trace!(
                    "handle_root_in_topo: {:?}==>{:#?}",
                    &root_id,
                    &children_list
                );
                info!("handle_root_in_topo: {:?}", &root_id);

                // ─────────────────────────────────────────────────────────────────

                let nix: NodeIndex<Self::Ix> = node_index(root_id.clone());

                let edge_ix = edge_index_no_source(root_id.clone());
                GraphNodeBuilder::new(root_id.clone())
                    .with_gel_sa(use_state(StateAnchor::constant(Rc::new(
                        Layer::<Message>::new(root_id.clone()).into(),
                    ))))
                    .with_incoming_eix_set([edge_ix.clone()].into_iter().collect())
                    .with_outgoing_eix_set_with_default_capacity(5)
                    .build_in_topo(self);

                let width = global_width();
                let height = global_height();

                let mut root_ei = GraphEdgeBuilder::new(edge_ix.clone())
                    .with_size((width, height))
                    .build_in_topo(self)
                    .unwrap();

                // let mut root_ei = self
                //     .setup_edge_in_topo(
                //         edge_ix.clone(),
                //         (width.into(), height.into()),
                //         (
                //             GenericSize::default().into(),
                //             GenericSize::default().into(),
                //             GenericSize::default().into(),
                //         ),
                //         (
                //             GenericSize::default().into(),
                //             GenericSize::default().into(),
                //             GenericSize::default().into(),
                //         ),
                //     )
                //     // .setup_wh_edge_in_topo(edge_index_no_source(root_id.clone()), 1920, 1080)
                //     .unwrap();
                debug_assert_eq!(
                    self.borrow()
                        .get_node_use_ix(edge_ix.target_nix().as_ref().unwrap().index())
                        .unwrap()
                        .incoming_len(),
                    1
                );

                let path = EPath::<Self::Ix>::new(vector![edge_ix]);
                // • • • • •

                illicit::Layer::new().offer(path.clone()).enter(|| {
                    debug_assert_eq!(*illicit::expect::<EPath<Self::Ix>>(), path);

                    root_ei.shape_of_use(edge_shapers);

                    illicit::Layer::new().offer(nix.clone()).enter(|| {
                        assert_eq!(*illicit::expect::<NodeIndex<Self::Ix>>(), nix);
                        trace!("{:?}", *illicit::expect::<NodeIndex<Self::Ix>>());
                        for child_layer in children_list.iter() {
                            self.handle_children_in_topo(None, child_layer);
                        }
                    });
                });
            }
            _ => {
                panic!("not allow this , first element must layer ");
            }
        }
    }

    #[allow(clippy::too_many_lines)]
    #[instrument(skip(self, tree_element))]
    #[topo::nested]
    fn handle_children_in_topo(
        &self,
        replace_id: Option<&Self::Ix>,
        tree_element: &'_ GTreeBuilderElement<Message>,
    ) {
        debug!("handle_children");
        let parent_nix = (*illicit::expect::<NodeIndex<Self::Ix>>()).clone();
        match tree_element {
            //
            GTreeBuilderElement::Layer(org_id, edge_shapers, children_list) => {
                let id = replace_id.unwrap_or(org_id);
                let _span = info_span!("-> [layer] ", ?org_id, ?id, ?parent_nix).entered();

                trace!("handle_children:\n{:?}==>{:#?}", &id, &children_list);

                //NOTE current node 因为dyn 节点 插入新节点时候 没有删除原存在节点,所以会重复走 handle_children_in_topo, 当前这里处理是ID存在就全部跳过

                if self.borrow().nodes_contains_key(id) {
                    warn!("children:Layer id:{} already exists , pass", id);
                    return;
                }
                // node index
                //TODO 检查重复插入节点时候 会出现什么问题,build_in_topo 没有使用 or_insert_node

                let nix: NodeIndex<Self::Ix> = node_index(id.clone());
                let edge_ix = EdgeIndex::new(parent_nix, nix.clone());
                GraphNodeBuilder::new(id.clone())
                    .with_gel_sa(use_state(StateAnchor::constant(Rc::new(
                        Layer::<Message>::new(id.clone()).into(),
                    ))))
                    .with_incoming_eix_set([edge_ix.clone()].into_iter().collect())
                    .with_outgoing_eix_set_with_default_capacity(2)
                    .build_in_topo(self);

                trace!("\nhandle_children:\n inserted node: {:#?}", &nix);

                // edge
                let mut new_def_ei = GraphEdgeBuilder::new(edge_ix).build_in_topo(self).unwrap();

                // let mut new_def_ei = self.setup_default_edge_in_topo(edge_ix).unwrap();
                trace!("\nhandle_children:\n inserted edge: {:#?}", &nix);

                let path = (*illicit::expect::<EPath<Self::Ix>>()).link_ref(nix.clone());

                illicit::Layer::new().offer(path.clone()).enter(|| {
                    debug_assert_eq!(*illicit::expect::<EPath<Self::Ix>>(), path.clone());
                    new_def_ei.shape_of_use(edge_shapers);

                    // next
                    #[cfg(debug_assertions)]
                    illicit::Layer::new().offer(nix.clone()).enter(|| {
                        debug_assert_eq!(*illicit::expect::<NodeIndex<Self::Ix>>(), nix.clone());
                        children_list.iter().for_each(|child_layer| {
                            self.handle_children_in_topo(None, child_layer)
                        });
                    });
                    #[cfg(not(debug_assertions))]
                    illicit::Layer::new().offer(nix).enter(|| {
                        children_list.iter().for_each(|child_layer| {
                            self.handle_children_in_topo(None, child_layer)
                        });
                    });
                });
            }

            // GTreeBuilderElement::El(id, element) => {
            //     let _span =
            //         trace_span!("-> handle_children_in_topo [El] ", ?id, ?parent_nix).entered();

            //     let nix = self.insert_node(id.clone(), element.clone().into());

            //     //TODO string style nodes impl  or edge:empty
            //     // let e = format!("{} -> {}", parent_nix.index(), nix.index()).into();
            //     // trace!("{}", &e);
            //     // self.insert_update_edge(&parent_nix, &nix, e);
            //     let _ei = self
            //         .setup_default_edge_in_topo(EdgeIndex::new(parent_nix.clone(), nix))
            //         .unwrap();
            // }
            GTreeBuilderElement::GElementTree(org_id, edge_shapers, gel, children_list) => {
                let id = replace_id.unwrap_or(org_id);
                info!(
                    "\n handle children [GElementTree]: org_id: {:?},  id : {:?}",
                    org_id, id
                );
                // warn!("\n handle children [GElementTree]: org_id: {:?},  id : {:?}", org_id, id);

                let _span =
                    trace_span!("-> handle_children [GElementTree] ", ?id, ?parent_nix).entered();

                //node index

                let nix: NodeIndex<Self::Ix> = node_index(id.clone());
                let edge_ix = EdgeIndex::new(parent_nix.clone(), nix.clone());

                match gel{

                    GElement::SaNode_(gel_sa) => {
                        GraphNodeBuilder::new(id.clone())
                        //TODO GTreeBuilderElement use Rc
                        .with_gel_sa(use_state(gel_sa.clone()))
                        .with_incoming_eix_set([edge_ix.clone()].into_iter().collect())
                        .with_outgoing_eix_set_with_default()
                        .build_in_topo(self);
                    },
                    GElement::EvolutionaryFactor(evo) => {
                        let mut g = self.borrow_mut();
                        let parent_item = g.get_mut_node_item(&parent_nix).unwrap();
                        let rc_sa_rc_parent = parent_item.get_gel_rc_sa();
                        warn!("---- parent anchor: {}",&rc_sa_rc_parent);
                        let gel_sa = evo.evolution(&*rc_sa_rc_parent);
                        warn!("---- before run evolution , anchor: {}",&rc_sa_rc_parent);
                        parent_item.set_gel_sa( gel_sa);
                        return
                    },

                    GElement::Builder_(_) |
                    GElement::Layer_(_) |
                    // GElement::Text_(_) |
                    // GElement::Button_(_) |
                    GElement::Shaper_(_) |
                    GElement::Event_(_) |
                    GElement::Generic_(_) |
                    GElement::NodeRef_(_)   =>{
                        GraphNodeBuilder::new(id.clone())
                        //TODO GTreeBuilderElement use Rc
                        .with_gel_sa(use_state(StateAnchor::constant(Rc::new(gel.clone()))))
                        .with_incoming_eix_set([edge_ix.clone()].into_iter().collect())
                        .with_outgoing_eix_set_with_default_capacity(1)
                        .build_in_topo(self);
                    },
                    GElement::EmptyNeverUse => unreachable!(),

                };

                // .map_or_else(|| {

                // }, |gel_sa| {

                // });

                //edge
                let mut new_def_ei = GraphEdgeBuilder::new(edge_ix).build_in_topo(self).unwrap();
                // let mut new_def_ei = self.setup_default_edge_in_topo(edge_ix).unwrap();

                let path = (*illicit::expect::<EPath<Self::Ix>>()).link_ref(nix.clone());

                illicit::Layer::new().offer(path.clone()).enter(|| {
                    debug_assert_eq!(*illicit::expect::<EPath<Self::Ix>>(), path.clone());
                    new_def_ei.shape_of_use(edge_shapers);
                    debug!("new_def_ei: {}", &new_def_ei);

                    //next
                    illicit::Layer::new().offer(nix.clone()).enter(|| {
                        // #[cfg(debug_assertions)]
                        debug_assert_eq!(*illicit::expect::<NodeIndex<Self::Ix>>(), nix.clone());

                        for child_gtree_builder in children_list.iter() {
                            self.handle_children_in_topo(None, child_gtree_builder);
                        }
                    });
                });
            }
            // GTreeBuilderElement::SaMapEffectGElementTree(org_id, _edge_shapers, builder_fn, _children_list) => {
            //     let id = replace_id.unwrap_or(org_id);
            //     // info!("\n handle children [SaMapEffectGElementTree]: org_id: {:?},  id : {:?}", org_id, id);
            //     warn!("\n handle children [SaMapEffectGElementTree]: org_id: {:?},  id : {:?}", org_id, id);

            //     let _span =
            //         trace_span!("-> handle_children [SaMapEffectGElementTree] ", ?id, ?parent_nix).entered();

            //     // let parent = self.borrow_mut().get_mut_node_item(&parent_nix).unwrap();
            //     // parent.set_gel_sa( (**builder_fn)(parent.gel_sa()));

            //     let mut g = self.borrow_mut();
            //     let parent_item = g.get_mut_node_item(&parent_nix).unwrap();
            //     let rc_p = parent_item.get_gel_rc_sa();
            //     warn!("---- anchor: {}",&rc_p);
            //     let gel_sa = (**builder_fn)(&*rc_p);
            //     warn!("---- before run builder , anchor: {}",&rc_p);
            //     parent_item.set_gel_sa( gel_sa);

            // }
            //TODO _edge_shaper use for  inject element
            GTreeBuilderElement::Dyn(org_id, _edge_shaper, sa_dict_gbe) => {
                let id = replace_id.unwrap_or(org_id);
                info!(
                    "\n handle children [Dyn]: org_id: {:?},  id : {:?}",
                    org_id, id
                );

                let _span = trace_span!("-> handle_children [SA] ", ?parent_nix).entered();
                debug!("builder:: GTreeBuilderElement::Dyn id:{:?}", id);

                let this = self.clone();
                let this2 = self.clone();

                let current_path = (*illicit::expect::<EPath<Self::Ix>>()).clone();

                // let parent_nix = (*illicit::expect::<NodeIndex<Self::Ix>>()).clone();
                // let update_id = TopoKey::new(topo::CallId::current());
                let update_id = TopoKey::new(topo::root(|| {
                    topo::call_in_slot(sa_dict_gbe.id(), topo::CallId::current)
                }));
                //TODO move it , for  use StateAnchor
                sa_dict_gbe
                    .insert_before_fn(
                        update_id,
                        move |_skip, current, new_v| {
                            //// TODO use graph find all parent
                            // let parent = parent_nix.clone();
                            debug!("builder::running before_fn");
                            trace!("builder::before_fn: is current has? {}", &current.is_some());
                            if let Some(old_data) = current {
                                let old_data_clone = (**old_data).clone();

                                let new_v_removed =
                                    old_data_clone.relative_complement(new_v.clone());
                                for k in new_v_removed.keys() {
                                    this.borrow_mut().remove_node(node_index(k.clone()));
                                }

                                //INFO like: https://stackoverflow.com/questions/56261476/why-is-finding-the-intersection-of-integer-sets-faster-with-a-vec-compared-to-bt
                            }
                        },
                        false,
                    )
                    .unwrap();

                // let update_id2 =TopoKey::new(topo::CallId::current());
                // let update_id2 = TopoKey::new(topo::call(topo::CallId::current));
                let gtbe_id = id.clone();

                sa_dict_gbe
                    .insert_after_fn(
                        update_id,
                        move |_skip, value| {
                            debug!("builder::running after_fn");

                            let cur_parent_nix = illicit::get::<NodeIndex<Self::Ix>>()
                                .map_or(parent_nix.clone(), |p| (*p).clone());

                            let cur_path = illicit::get::<EPath<Self::Ix>>()
                                .map_or(current_path.clone(), |p| (*p).clone());

                            illicit::Layer::new()
                                .offer(cur_parent_nix)
                                .offer(cur_path)
                                .enter(|| {
                                    value.iter().for_each(|(k, v)| {
                                    info!("builder::after_fn >> illicit env , \n ---- run handle_children_in_topo for key:{}",k);
                                     trace_span!(
                                        "builder::illicit ",
                                        ).in_scope(||{
                                            //TODO: use dyn key id in root 区分每个dyn, 这样可以使用same key
                                            topo::root(||{
                                                call_in_slot(&gtbe_id,||{
                                                    call_in_slot(k,||
                                                        {
                                                            this2.handle_children_in_topo(Some(k),v);
                                                        }
                                                    );
                                                });

                                            });

                                        });
                                    });
                                });
                        },
                        false, //TODO make true (false for debug)
                    )
                    .unwrap();

                debug!("builder:: sa_dict_gbe run handle_children_in_topo");
                let rc_sa = sa_dict_gbe.get_rc();
                for (k, v) in rc_sa.iter() {
                    debug!("builder:: sa_dict_gbe handle_children_in_topo call");
                    topo::root(|| {
                        call_in_slot(id, || {
                            call_in_slot(k, || {
                                self.handle_children_in_topo(Some(k), v);
                            });
                        });
                    });
                }

                // value.iter().for_each(|(_k, v)| {
                //     debug!("builder:: sa_dict_gbe handle_children_in_topo call");

                //     self.handle_children_in_topo(v);
                // });
            }

            GTreeBuilderElement::ShapingUse(org_id, u) => {
                let id = replace_id.unwrap_or(org_id);
                info!(
                    "\n handle children [ShapingUse]: org_id: {:?},  id : {:?}",
                    org_id, id
                );

                let _span =
                    trace_span!("-> handle_children_in_topo [ShapingUse] ", ?id, ?parent_nix)
                        .entered();

                //node index

                let nix: NodeIndex<Self::Ix> = node_index(id.clone());
                let edge_ix = EdgeIndex::new(parent_nix, nix);
                GraphNodeBuilder::new(id.clone())
                    .with_gel_sa(use_state(StateAnchor::constant(Rc::new(u.clone().into()))))
                    .with_incoming_eix_set([edge_ix.clone()].into_iter().collect())
                    .with_outgoing_eix_set(IndexSet::with_hasher(
                        BuildHasherDefault::<CustomHasher>::default(),
                    ))
                    .build_in_topo(self);

                let _new_def_ei = GraphEdgeBuilder::new(edge_ix).build_in_topo(self).unwrap();
                // let _ei = self.setup_default_edge_in_topo(edge_ix).unwrap();
            }

            GTreeBuilderElement::Cl(org_id, dyn_fn) => {
                let id = replace_id.unwrap_or(org_id);
                info!(
                    "\n handle children [Cl]: org_id: {:?},  id : {:?}",
                    org_id, id
                );

                let _span = trace_span!(
                    "-> handle_children_in_topo [Cl] dyn_fn running",
                    ?id,
                    ?parent_nix
                )
                .entered();

                dyn_fn();
            }

            // // TODO make RC remove most clones
            GTreeBuilderElement::Event(org_id, callback) => {
                debug!("GTreeBuilderElement::Event : {:?} {:?}", org_id, replace_id);
                let id = replace_id.unwrap_or(org_id);
                info!(
                    "\n handle children [Event]: org_id: {:?},  id : {:?}",
                    org_id, id
                );

                let _span =
                    trace_span!("-> handle_children_in_topo [Event] ", ?id, ?parent_nix).entered();

                // TODO: make all into() style?
                // node index

                let nix: NodeIndex<Self::Ix> = node_index(id.clone());
                let edge_ix = EdgeIndex::new(parent_nix, nix);
                GraphNodeBuilder::new(id.clone())
                    .with_gel_sa(use_state(StateAnchor::constant(Rc::new(
                        callback.clone().into(),
                    ))))
                    .with_incoming_eix_set([edge_ix.clone()].into_iter().collect())
                    .with_outgoing_eix_set_with_default()
                    .build_in_topo(self);

                let _new_def_ei = GraphEdgeBuilder::new(edge_ix).build_in_topo(self).unwrap();
                // let _ei = self.setup_default_edge_in_topo(edge_ix).unwrap();
            } // GTreeBuilderElement::GenericTree(id, edge_shapers, dyn_gel, shapers) => {
              //     panic!("test here");
              //     let _span =
              //         trace_span!("-> handle_children [GElementTree] ", ?id, ?parent_nix).entered();

              //     //node index
              //     let nix = self.insert_node(id.clone(), dyn_gel.clone().into());

              //     //edge
              //     let mut ei = self
              //         .setup_default_edge_in_topo(EdgeIndex::new(parent_nix.clone(), nix.clone()))
              //         .unwrap();

              //     let path = (&*illicit::expect::<EPath<Self::Ix>>()).add_build(nix.clone());

              //     illicit::Layer::new().offer(path.clone()).enter(|| {
              //         debug_assert_eq!(*illicit::expect::<EPath<Self::Ix>>(), path.clone());
              //         ei.shaping_use(edge_shapers);

              //         //next
              //         #[cfg(debug_assertions)]
              //         illicit::Layer::new().offer(nix.clone()).enter(|| {
              //             assert_eq!(*illicit::expect::<NodeIndex<Self::Ix>>(), nix.clone());
              //             shapers
              //                 .iter()
              //                 .for_each(|child_layer| self.handle_children_in_topo(child_layer));
              //         });
              //         #[cfg(not(debug_assertions))]
              //         illicit::Layer::new().offer(nix).enter(|| {
              //             shapers
              //                 .iter()
              //                 .for_each(|child_layer| self.handle_children_in_topo(child_layer));
              //         });
              //     });
              // }
        };
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use emg::edge_index_no_source;
    use emg_common::IdStr;
    use emg_state::{use_state, StateAnchor};

    use crate::{widget::Layer, GraphType};

    use super::GraphNodeBuilder;

    #[derive(Debug, Clone, PartialEq)]
    enum Message {}

    #[test]
    fn node_item_build() {
        let emg_graph = GraphType::<Message>::default();
        let emg_graph_rc_refcell = Rc::new(RefCell::new(emg_graph));

        // ────────────────────────────────────────────────────────────────────────────────
        let root_id = IdStr::new_inline("a");
        let root_edge_ix = edge_index_no_source(root_id.clone());
        // ────────────────────────────────────────────────────────────────────────────────

        GraphNodeBuilder::new(root_id.clone())
            .with_gel_sa(use_state(StateAnchor::constant(Rc::new(
                Layer::<Message>::new(root_id.clone()).into(),
            ))))
            .with_incoming_eix_set([root_edge_ix.clone()].into_iter().collect())
            .with_outgoing_eix_set_with_default_capacity(5)
            .build_in_topo(&emg_graph_rc_refcell);
    }
}
