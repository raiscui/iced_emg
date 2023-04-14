/*
 * @Author: Rais
 * @Date: 2022-08-18 17:58:00
 * @LastEditTime: 2023-04-13 23:44:18
 * @LastEditors: Rais
 * @Description:
 */

use crate::{
    error::Error,
    g_node::{EmgNodeItem, GelType, GraphType, NItem},
    g_tree_builder::{GTreeBuilderElement, GTreeBuilderFn},
    graph_edit::GraphEditor,
    GElement,
};
use emg::{edge_index_no_source, node_index, Edge, EdgeIndex, EdgePlugsCollect, NodeIndex};

use emg_common::{im::vector, GenericSize, IdStr};
use emg_hasher::CustomHasher;
use emg_layout::{global_height, global_width, EPath, EmgEdgeItem};
use emg_shaping::ShapingUseDyn;
use emg_state::{
    anchors::{expert::CastIntoValOrAnchor, singlethread::ValOrAnchor},
    topo::{self, call_in_slot},
    use_state, use_state_voa, CloneState,
};
use indexmap::IndexSet;
use std::{cell::RefCell, hash::BuildHasherDefault, rc::Rc};
use tracing::{debug, info, instrument, trace, trace_span, warn};

pub struct GraphNodeBuilder<Message>
where
    Message: 'static,
{
    ix: IdStr,

    opt_gel_sa: Option<NItem<Message>>,
    opt_incoming_eix_set: Option<EdgePlugsCollect>,
    opt_outgoing_eix_set: Option<EdgePlugsCollect>,
}

impl<Message> GraphNodeBuilder<Message>
where
    Message: 'static,
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
    pub fn with_incoming_eix_set(mut self, incoming_eix_set: EdgePlugsCollect) -> Self {
        self.opt_incoming_eix_set = Some(incoming_eix_set);
        self
    }

    #[allow(clippy::missing_const_for_fn)]
    pub fn with_outgoing_eix_set(mut self, outgoing_eix_set: EdgePlugsCollect) -> Self {
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
    pub fn build_in_topo(self, graph_rc: &Rc<RefCell<GraphType<Message>>>) {
        let incoming_eix_set = use_state(|| self.opt_incoming_eix_set.unwrap());
        let outgoing_eix_set = use_state(|| self.opt_outgoing_eix_set.unwrap());

        let node_item = |key: IdStr| {
            EmgNodeItem::<NItem<Message>, GelType<Message>>::new(
                key,
                self.opt_gel_sa.unwrap(),
                &incoming_eix_set.watch(),
                &outgoing_eix_set.watch(),
                graph_rc.clone(),
            )
        };
        graph_rc.borrow_mut().or_insert_node_with_plugs(
            self.ix,
            node_item,
            incoming_eix_set,
            outgoing_eix_set,
        );
    }
}

pub struct GraphEdgeBuilder {
    edge_ix: EdgeIndex,
    opt_size: Option<(ValOrAnchor<GenericSize>, ValOrAnchor<GenericSize>)>,
    opt_origin: Option<(
        ValOrAnchor<GenericSize>,
        ValOrAnchor<GenericSize>,
        ValOrAnchor<GenericSize>,
    )>,
    opt_align: Option<(
        ValOrAnchor<GenericSize>,
        ValOrAnchor<GenericSize>,
        ValOrAnchor<GenericSize>,
    )>,
}

impl GraphEdgeBuilder {
    pub fn new(edge_ix: EdgeIndex) -> Self {
        Self {
            edge_ix,
            opt_size: None,
            opt_origin: None,
            opt_align: None,
        }
    }

    pub fn with_size(
        mut self,
        (w, h): (
            impl Into<ValOrAnchor<GenericSize>>,
            impl Into<ValOrAnchor<GenericSize>>,
        ),
    ) -> Self {
        self.opt_size = Some((w.into(), h.into()));
        self
    }
    pub fn with_origin(
        mut self,
        (origin_x, origin_y, origin_z): (
            impl Into<ValOrAnchor<GenericSize>>,
            impl Into<ValOrAnchor<GenericSize>>,
            impl Into<ValOrAnchor<GenericSize>>,
        ),
    ) -> Self {
        self.opt_origin = Some((origin_x.into(), origin_y.into(), origin_z.into()));
        self
    }
    pub fn with_align(
        mut self,
        (align_x, align_y, align_z): (
            impl Into<ValOrAnchor<GenericSize>>,
            impl Into<ValOrAnchor<GenericSize>>,
            impl Into<ValOrAnchor<GenericSize>>,
        ),
    ) -> Self {
        self.opt_align = Some((align_x.into(), align_y.into(), align_z.into()));
        self
    }

    #[topo::nested]
    pub fn build_in_topo<Message>(
        self,
        graph_rc: &Rc<RefCell<GraphType<Message>>>,
    ) -> Result<EmgEdgeItem, Error> {
        let g = graph_rc.borrow();
        let size = self.opt_size.unwrap_or_default();
        let origin = self.opt_origin.unwrap_or_default();
        let align = self.opt_align.unwrap_or_default();
        // ─────────────────────────────────────────────────────────────

        g.nodes_connect(&self.edge_ix)?;
        // ─────────────────────────────────────────────────────
        let source = use_state(|| self.edge_ix.source_nix().cloned());
        let target = use_state(|| self.edge_ix.target_nix().cloned());
        // ─────────────────────────────────────────────────────────────
        let edges_watch = g.get_raw_edges_watch();

        let edge_item = || {
            EmgEdgeItem::new_in_topo(
                &source.watch(),
                &target.watch(),
                edges_watch,
                size,
                origin,
                align,
            )
        };
        // ─────────────────────────────────────────────────────────────

        Ok(g.or_insert_edge_only(self.edge_ix, || Edge::new(source, target, edge_item())))
    }
}

#[cfg(test)]
mod testaa {
    use emg_state::{topo, use_state, StateVar};

    fn x(func: impl FnOnce() -> StateVar<i32>) -> StateVar<i32> {
        func()
    }

    // #[topo::nested]
    fn aa() -> (StateVar<i32>, StateVar<i32>) {
        let a = || use_state(|| 1);
        (x(a), a())
    }

    #[test]
    fn nn() {
        let (f1, f2) = aa();
        let (f3, f4) = aa();
        assert_eq!(f1.id(), f3.id());
        assert_eq!(f1.id(), f2.id());
    }
}

//TODO make trait use GraphNodeBuilder GraphNodeBuilder for Rc<RefCell<GraphType<Message>>> and GraphType<Message> for easy use in update fn

impl<Message> GTreeBuilderFn<Message> for Rc<RefCell<GraphType<Message>>>
where
    Message: 'static,
    //     Message: std::clone::Clone + std::cmp::PartialEq + std::fmt::Debug,
{
    type GraphType = Rc<RefCell<GraphType<Message>>>;
    type GraphEditor = GraphEditor<Message>;

    fn editor(&self) -> Self::GraphEditor {
        GraphEditor(self.clone())
    }

    fn graph(&self) -> &Self::GraphType {
        self
    }

    #[topo::nested]
    fn handle_root_in_topo(&self, tree_element: &GTreeBuilderElement<Message>) {
        match tree_element {
            GTreeBuilderElement::GElementTree(root_id, edge_shapers, gel, children_list) => {
                let _span = trace_span!("=> handle_root [GElementTree] ",%root_id).entered();
                let nix: NodeIndex = node_index(root_id.clone());

                let edge_ix = edge_index_no_source(root_id.clone());
                GraphNodeBuilder::new(root_id.clone())
                    .with_gel_sa(use_state_voa(|| Rc::new(gel.clone())))
                    // .with_gel_sa(use_state(|| {
                    //     StateAnchor::constant(Rc::new(
                    //         Layer::<Message>::new(root_id.clone()).into(),
                    //     ))
                    // }))
                    .with_incoming_eix_set([edge_ix.clone()].into_iter().collect())
                    .with_outgoing_eix_set_with_default_capacity(5)
                    .build_in_topo(self);

                let width = global_width();
                let height = global_height();

                let mut root_ei = GraphEdgeBuilder::new(edge_ix.clone())
                    .with_size((width.cast_into(), height.cast_into()))
                    .build_in_topo(self)
                    .unwrap();

                debug_assert_eq!(
                    self.borrow()
                        .get_node_use_ix(edge_ix.target_nix().as_ref().unwrap().index())
                        .unwrap()
                        .incoming_len(),
                    1
                );
                let path = EPath::new(vector![edge_ix]);

                illicit::Layer::new().offer(path.clone()).enter(|| {
                    debug_assert_eq!(*illicit::expect::<EPath>(), path);

                    let _ = root_ei.shaping_use_dyn(edge_shapers);

                    illicit::Layer::new().offer(nix.clone()).enter(|| {
                        assert_eq!(*illicit::expect::<NodeIndex>(), nix);
                        trace!("{:?}", *illicit::expect::<NodeIndex>());
                        for child in children_list.iter() {
                            self.handle_children_in_topo(None, child);
                        }
                    });
                });
            }
            // GTreeBuilderElement::Layer(root_id, edge_shapers, children_list) => {
            //     let _span = trace_span!("=> handle_root [layer] ",%root_id).entered();
            //     trace!(
            //         "handle_root_in_topo: {:?}==>{:#?}",
            //         &root_id,
            //         &children_list
            //     );
            //     info!("handle_root_in_topo: {:?}", &root_id);

            //     // ─────────────────────────────────────────────────────────────────

            //     let nix: NodeIndex = node_index(root_id.clone());

            //     let edge_ix = edge_index_no_source(root_id.clone());
            //     GraphNodeBuilder::new(root_id.clone())
            //         .with_gel_sa(use_state(||StateAnchor::constant(Rc::new(
            //             Layer::<Message>::new(root_id.clone()).into(),
            //         ))))
            //         .with_incoming_eix_set([edge_ix.clone()].into_iter().collect())
            //         .with_outgoing_eix_set_with_default_capacity(5)
            //         .build_in_topo(self);

            //     let width = global_width();
            //     let height = global_height();

            //     let mut root_ei = GraphEdgeBuilder::new(edge_ix.clone())
            //         .with_size((width, height))
            //         .build_in_topo(self)
            //         .unwrap();

            //     debug_assert_eq!(
            //         self.borrow()
            //             .get_node_use_ix(edge_ix.target_nix().as_ref().unwrap().index())
            //             .unwrap()
            //             .incoming_len(),
            //         1
            //     );

            //     let path = EPath::new(vector![edge_ix]);
            //     // • • • • •

            //     illicit::Layer::new().offer(path.clone()).enter(|| {
            //         debug_assert_eq!(*illicit::expect::<EPath>(), path);

            //         root_ei.shaping_use_dyn(edge_shapers);

            //         illicit::Layer::new().offer(nix.clone()).enter(|| {
            //             assert_eq!(*illicit::expect::<NodeIndex>(), nix);
            //             trace!("{:?}", *illicit::expect::<NodeIndex>());
            //             for child_layer in children_list.iter() {
            //                 self.handle_children_in_topo(None, child_layer);
            //             }
            //         });
            //     });
            // }
            _ => {
                panic!("not allow this , first element not match ");
            }
        }
    }

    #[allow(clippy::too_many_lines)]
    #[instrument(skip(self, tree_element))]
    #[topo::nested]
    fn handle_children_in_topo(
        &self,
        replace_id: Option<&IdStr>,
        tree_element: &'_ GTreeBuilderElement<Message>,
    ) {
        debug!("handle_children");
        let parent_nix = illicit::get::<NodeIndex>().ok().as_deref().cloned();
        // let parent_nix = (*illicit::expect::<NodeIndex>()).clone();
        match tree_element {
            //
            // GTreeBuilderElement::Layer(org_id, edge_shapers, children_list) => {
            //     let id = replace_id.unwrap_or(org_id);
            //     let _span = info_span!("-> [layer] ", ?org_id, ?id, ?parent_nix).entered();

            //     trace!("handle_children:\n{:?}==>{:#?}", &id, &children_list);

            //     //NOTE current node 因为dyn 节点 插入新节点时候 没有删除原存在节点,所以会重复走 handle_children_in_topo, 当前这里处理是ID存在就全部跳过

            //     if self.borrow().nodes_contains_key(id) {
            //         warn!("children:Layer id:{} already exists , pass", id);
            //         return;
            //     }
            //     // node index
            //     //TODO 检查重复插入节点时候 会出现什么问题,build_in_topo 没有使用 or_insert_node

            //     let nix: NodeIndex = node_index(id.clone());
            //     let edge_ix = EdgeIndex::new(parent_nix, nix.clone());
            //     GraphNodeBuilder::new(id.clone())
            //         .with_gel_sa(use_state(||StateAnchor::constant(Rc::new(
            //             Layer::<Message>::new(id.clone()).into(),
            //         ))))
            //         .with_incoming_eix_set([edge_ix.clone()].into_iter().collect())
            //         .with_outgoing_eix_set_with_default_capacity(2)
            //         .build_in_topo(self);

            //     trace!("\nhandle_children:\n inserted node: {:#?}", &nix);

            //     // let mut new_def_ei = self.setup_default_edge_in_topo(edge_ix).unwrap();
            //     trace!("\nhandle_children:\n inserted edge: {:#?}", &nix);

            //     let path = match illicit::get::<EPath>().ok().as_deref() {
            //         Some(path) => path.link_ref(nix.clone()),
            //         None => EPath::new(vector![edge_ix.clone()]),
            //     };

            //     // edge
            //     let mut new_def_ei = GraphEdgeBuilder::new(edge_ix).build_in_topo(self).unwrap();

            //     illicit::Layer::new().offer(path.clone()).enter(|| {
            //         debug_assert_eq!(*illicit::expect::<EPath>(), path);
            //         new_def_ei.shaping_use_dyn(edge_shapers);

            //         // next
            //         #[cfg(debug_assertions)]
            //         illicit::Layer::new().offer(nix.clone()).enter(|| {
            //             debug_assert_eq!(*illicit::expect::<NodeIndex>(), nix.clone());
            //             children_list
            //                 .iter()
            //                 .for_each(|child| self.handle_children_in_topo(None, child));
            //         });
            //         #[cfg(not(debug_assertions))]
            //         illicit::Layer::new().offer(nix).enter(|| {
            //             children_list
            //                 .iter()
            //                 .for_each(|child| self.handle_children_in_topo(None, child));
            //         });
            //     });
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

                let nix: NodeIndex = node_index(id.clone());

                let edge_ix = EdgeIndex::new(parent_nix.clone(), nix.clone());

                match gel{

                    GElement::SaNode_(gel_sa) => {
                        GraphNodeBuilder::new(id.clone())
                        //TODO GTreeBuilderElement use Rc
                        .with_gel_sa(use_state_voa(||gel_sa.clone()))
                        .with_incoming_eix_set([edge_ix.clone()].into_iter().collect())
                        .with_outgoing_eix_set_with_default()
                        .build_in_topo(self);
                    },
                    //NOTE 这个节点篡改 parent ,
                    //TODO 目前 get_gel_sa ,修改,再set_gel_sa 会丢失原 gel_sa , 这不是正确的逻辑
                    //TODO not dyn ,make it dyn
                    GElement::EvolutionaryFactor(evo) => {
                        let g = self.borrow();
                        let parent_item = g.get_node_item(&parent_nix.expect("parent nix must have in EvolutionaryFactor builder")).unwrap();
                        let sa_rc_parent = parent_item.get_gel_sa();
                        warn!("---- parent anchor: {}",&sa_rc_parent);
                        let gel_sa = evo.evolution(&sa_rc_parent);
                        warn!("---- before run evolution , anchor: {}",&sa_rc_parent);
                        //TODO 可能可以不在这里设置 parent, 在运行时设置? 这样可以变更parent 后,可以动态应用,这样做 需要check与 Dyn的区别
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
                    GElement::NodeRef_(_)
                     //
                      =>{
                        GraphNodeBuilder::new(id.clone())
                        //TODO GTreeBuilderElement use Rc
                        .with_gel_sa(use_state_voa(||Rc::new(gel.clone())))
                        .with_incoming_eix_set([edge_ix.clone()].into_iter().collect())
                        .with_outgoing_eix_set_with_default_capacity(1)
                        .build_in_topo(self);
                    },
                    GElement::EmptyNeverUse => unreachable!(),
                     //@ accesskit ─────────────────────────────────────
                     #[cfg(feature = "video-player")]
                     GElement::Video_(_)
                      //
                       =>{
                         GraphNodeBuilder::new(id.clone())
                         //TODO GTreeBuilderElement use Rc
                         .with_gel_sa(use_state_voa(||Rc::new(gel.clone())))
                         .with_incoming_eix_set([edge_ix.clone()].into_iter().collect())
                         .with_outgoing_eix_set_with_default_capacity(1)
                         .build_in_topo(self);
                     },

                };

                let path = match illicit::get::<EPath>().ok().as_deref() {
                    Some(path) => path.link_ref(nix.clone()),
                    None => EPath::new(vector![edge_ix.clone()]),
                };

                //edge
                let mut new_def_ei = GraphEdgeBuilder::new(edge_ix).build_in_topo(self).unwrap();

                illicit::Layer::new().offer(path.clone()).enter(|| {
                    debug_assert_eq!(*illicit::expect::<EPath>(), path.clone());
                    new_def_ei.shaping_use_dyn(edge_shapers);
                    debug!("new_def_ei: {}", &new_def_ei);

                    //next
                    illicit::Layer::new().offer(nix.clone()).enter(|| {
                        // #[cfg(debug_assertions)]
                        debug_assert_eq!(*illicit::expect::<NodeIndex>(), nix.clone());

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

                //NOTE  not has self
                let current_path = (*illicit::expect::<EPath>()).clone();

                // let parent_nix = (*illicit::expect::<NodeIndex>()).clone();
                // let update_id = TopoKey::new(topo::CallId::current());

                //TODO move it , for  use StateAnchor
                sa_dict_gbe.insert_before_fn_in_topo(
                    // update_id,
                    move |_skip, current, new_v| {
                        //// TODO use graph find all parent
                        // let parent = parent_nix.clone();
                        debug!("builder::running before_fn");
                        trace!("builder::before_fn: is current has? {}", &current.is_some());
                        if let Some(old_data) = current {
                            let old_data_clone = (**old_data).clone();

                            let new_v_removed = old_data_clone.relative_complement(new_v.clone());
                            for k in new_v_removed.keys() {
                                this.borrow_mut()
                                    .remove_node_and_edge_and_disconnect(node_index(k.clone()));
                            }

                            //NOTE like: https://stackoverflow.com/questions/56261476/why-is-finding-the-intersection-of-integer-sets-faster-with-a-vec-compared-to-bt
                        }
                    },
                    false,
                    &[],
                );
                // .unwrap();

                // let update_id2 =TopoKey::new(topo::CallId::current());
                // let update_id2 = TopoKey::new(topo::call(topo::CallId::current));
                let gtbe_id = id.clone();

                // let parent_nix_clone = parent_nix.cloned();

                sa_dict_gbe
                    .insert_after_fn_in_topo(
                        // update_id,
                        move |_skip, value| {
                            debug!("builder::running after_fn");
                            let cur_parent_nix = illicit::get::<NodeIndex>().ok().as_deref().cloned().unwrap();
                            // .cloned().unwrap_or(parent_nix_clone.unwrap());
                                // .map_or(parent_nix_clone.unwrap(), |p| (*p).clone());


                                //NOTE: not with self ,use for illicit self ,not illicit children
                            let cur_path = illicit::get::<EPath>()
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
                        &[]
                    );
                // .unwrap();

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

                let nix: NodeIndex = node_index(id.clone());
                let edge_ix = EdgeIndex::new(parent_nix, nix);
                GraphNodeBuilder::new(id.clone())
                    .with_gel_sa(use_state_voa(|| Rc::new(u.clone().into())))
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

                let nix: NodeIndex = node_index(id.clone());
                let edge_ix = EdgeIndex::new(parent_nix, nix);
                GraphNodeBuilder::new(id.clone())
                    .with_gel_sa(use_state_voa(|| Rc::new(callback.clone().into())))
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

              //     let path = (&*illicit::expect::<EPath>()).add_build(nix.clone());

              //     illicit::Layer::new().offer(path.clone()).enter(|| {
              //         debug_assert_eq!(*illicit::expect::<EPath>(), path.clone());
              //         ei.shaping_use(edge_shapers);

              //         //next
              //         #[cfg(debug_assertions)]
              //         illicit::Layer::new().offer(nix.clone()).enter(|| {
              //             assert_eq!(*illicit::expect::<NodeIndex>(), nix.clone());
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
    use emg_state::use_state_voa;

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
            .with_gel_sa(use_state_voa(|| {
                Rc::new(Layer::<Message>::new(root_id.clone()).into())
            }))
            .with_incoming_eix_set([root_edge_ix].into_iter().collect())
            .with_outgoing_eix_set_with_default_capacity(5)
            .build_in_topo(&emg_graph_rc_refcell);
    }
}
