/*
 * @Author: Rais
 * @Date: 2022-06-05 19:54:16
 * @LastEditTime: 2022-06-15 12:54:25
 * @LastEditors: Rais
 * @Description:
 */
use crate::{
    emg_runtime::{EventNode, Layer},
    g_node::{EmgNodeItem, GraphType, NItem},
    GTreeBuilderElement, GTreeBuilderFn,
};
use crate::{GElement, NodeIndex};
use emg::{edge_index_no_source, node_index, Edge, EdgeIndex, EdgeCollect, Node};
use emg_core::GenericSize;
use emg_core::{vector, IdStr};
use emg_hasher::CustomHasher;
// use emg_core::{GenericSize, Vector};
use emg_layout::{global_height, global_width, EPath, EmgEdgeItem, GenericSizeAnchor};
use emg_refresh::{RefreshFor, RefreshForUse};
use emg_state::{
    topo::{self, call_in_slot},
    use_state,
    use_state_impl::TopoKey,
    CloneStateVar, Dict, StateAnchor, StateVar,
};
use indexmap::{indexset, IndexSet};
use std::{cell::RefCell, hash::BuildHasherDefault, rc::Rc};
use tracing::{debug, instrument, trace, trace_span, warn, info};

struct GraphNodeBuilder<Message,Ix=IdStr> 
where 
Message: std::cmp::PartialEq + std::clone::Clone + 'static, 
Ix: std::clone::Clone + std::hash::Hash + std::cmp::Ord + std::default::Default + 'static, {
    graph_rc: Rc<RefCell<GraphType<Message,Ix>>>,

    key:Option< Ix>,
    gel_sa:Option<StateAnchor<GElement<Message>>>,
    incoming_eix_set:Option<EdgeCollect<Ix>>,
    outgoing_eix_set:Option<EdgeCollect<Ix>>,

}

impl<Message> GraphNodeBuilder<Message, IdStr>
where 
Message: std::cmp::PartialEq + std::clone::Clone + 'static, 
{
    fn new(graph_rc: Rc<RefCell<GraphType<Message,IdStr>>>) -> Self { Self { graph_rc,key:None,gel_sa: None,incoming_eix_set:None,outgoing_eix_set:None} }
    #[allow(clippy::missing_const_for_fn)]
    fn and_key(mut self, k:IdStr) ->Self {
        self.key= Some(k);
        self
    }

    #[allow(clippy::missing_const_for_fn)]
    fn and_gel_sa(mut self, gel_sa: StateAnchor<GElement<Message>>) -> Self {
        self.gel_sa =Some( gel_sa);
        self
    }

    #[allow(clippy::missing_const_for_fn)]
    fn and_incoming_eix_set(mut self, incoming_eix_set: EdgeCollect<IdStr>)->Self {
        self.incoming_eix_set = Some(incoming_eix_set);
        self
    }

    #[allow(clippy::missing_const_for_fn)]
    fn and_outgoing_eix_set(mut self, outgoing_eix_set: EdgeCollect<IdStr>)->Self {
        self.outgoing_eix_set = Some(outgoing_eix_set);
        self
    }
    #[topo::nested]
    fn build_in_topo(self){
        let incoming_eix_set = use_state(self.incoming_eix_set.unwrap());
        let outgoing_eix_set = use_state(self.outgoing_eix_set.unwrap());
        let node_item = EmgNodeItem::<NItem<Message>,IdStr>::new(
            self.key.clone().unwrap(),
            self.gel_sa.unwrap(),
            &incoming_eix_set.watch(),
            &outgoing_eix_set.watch(),
            self.graph_rc.clone(),
        );
                //TODO all use or_insert_node?
        self.graph_rc.borrow_mut().insert_node_with_edges(self.key.unwrap(), 
        node_item, incoming_eix_set, outgoing_eix_set);
    }
}

impl<Message> GTreeBuilderFn<Message> for Rc<RefCell<GraphType<Message>>>
where
    Message: std::clone::Clone + std::cmp::PartialEq + std::fmt::Debug,
{
    type Ix = IdStr;

    #[topo::nested]
    #[instrument(skip(self))]
    fn setup_wh_edge_in_topo<T: Into<f64> + std::fmt::Debug>(
        &self,
        ei: EdgeIndex<Self::Ix>,
        w: T,
        h: T,
    ) -> Result<EmgEdgeItem<Self::Ix>, String> {
        let mut g = self.borrow_mut();
        g.nodes_connect_eix(&ei).ok_or("node insert eix fails")?;
        let source = use_state(ei.source_nix().as_ref().cloned());
        let target = use_state(ei.target_nix().as_ref().cloned());
        let edge_item = EmgEdgeItem::default_with_wh_in_topo(
            source.watch(),
            target.watch(),
            self.borrow().get_raw_edges_watch(),
            w,
            h,
        );
        g.just_insert_edge(ei, Edge::new(source, target, edge_item.clone()));

        Ok(edge_item)
    }

    #[topo::nested]
    #[instrument(skip(self, size, origin, align))]
    fn setup_edge_in_topo(
        &self,
        edge_index: EdgeIndex<Self::Ix>,

        size: (GenericSizeAnchor, GenericSizeAnchor),
        origin: (GenericSizeAnchor, GenericSizeAnchor, GenericSizeAnchor),
        align: (GenericSizeAnchor, GenericSizeAnchor, GenericSizeAnchor),
    ) -> Result<EmgEdgeItem<Self::Ix>, String> {
        let mut g = self.borrow_mut();
        g.nodes_connect_eix(&edge_index)
            .ok_or("node insert eix fails")?;

        let source = use_state(edge_index.source_nix().as_ref().cloned());
        let target = use_state(edge_index.target_nix().as_ref().cloned());
        let edge_item = EmgEdgeItem::new_in_topo(
            source.watch(),
            target.watch(),
            g.get_raw_edges_watch(),
            size,
            origin,
            align,
        );
        g.just_insert_edge(edge_index, Edge::new(source, target, edge_item.clone()));
        Ok(edge_item)
    }

    #[topo::nested]
    #[instrument(skip(self))]
    fn setup_default_edge_in_topo(
        &self,
        edge_index: EdgeIndex<Self::Ix>,
    ) -> Result<EmgEdgeItem<Self::Ix>, String> {
        let mut g = self.borrow_mut();
        g.nodes_connect_eix(&edge_index)
            .ok_or("node insert eix fails")?;
        trace!(
            "\n setup_default_edge_in_topo:\n nodes_connect_eix: {:#?}",
            &edge_index
        );

        let source = use_state(edge_index.source_nix().as_ref().cloned());
        trace!("\n setup_default_edge_in_topo:\n cloned sv e source");

        let target = use_state(edge_index.target_nix().as_ref().cloned());
        trace!("\n setup_default_edge_in_topo:\n cloned sv e target");

        let edge_item =
            EmgEdgeItem::default_in_topo(source.watch(), target.watch(), g.get_raw_edges_watch());
        let edge = Edge::new(source, target, edge_item.clone());
        g.just_insert_edge(edge_index, edge);
        Ok(edge_item)
    }

    #[topo::nested]
    fn handle_root_in_topo(&self, tree_element: &crate::GTreeBuilderElement<Message>) {
        match tree_element {
            GTreeBuilderElement::Layer(root_id, edge_refreshers, children_list) => {
                let _span = trace_span!("=> handle_root [layer] ",%root_id).entered();
                trace!("handle_root_in_topo: {:?}==>{:#?}", &root_id, &children_list);
                info!("handle_root_in_topo: {:?}", &root_id);

                // ─────────────────────────────────────────────────────────────────

            
                let nix: NodeIndex<Self::Ix> = node_index(root_id.clone());

                let edge_index = edge_index_no_source(root_id.clone());
                GraphNodeBuilder::new(self.clone())
                .and_key(root_id.clone())
                .and_gel_sa(StateAnchor::constant(Layer::<Message>::new(root_id.clone()).into()))
                .and_incoming_eix_set([edge_index.clone()].into_iter().collect())
                .and_outgoing_eix_set(IndexSet::with_capacity_and_hasher(
                    5,
                    BuildHasherDefault::<CustomHasher>::default(),
                ))
                .build_in_topo();

                let width = global_width();
                let height = global_height();
                let mut root_ei = self
                    //TODO: bind browser w h.
                    .setup_edge_in_topo(
                        edge_index.clone(),
                        (width.into(), height.into()),
                        (
                            GenericSize::default().into(),
                            GenericSize::default().into(),
                            GenericSize::default().into(),
                        ),
                        (
                            GenericSize::default().into(),
                            GenericSize::default().into(),
                            GenericSize::default().into(),
                        ),
                    )
                    // .setup_wh_edge_in_topo(edge_index_no_source(root_id.clone()), 1920, 1080)
                    .unwrap();
                debug_assert_eq!(
                    self.borrow()
                        .get_node_use_ix(edge_index.target_nix().as_ref().unwrap().index())
                        .unwrap()
                        .incoming_len(),
                    1
                );

                let path = EPath::<Self::Ix>::new(vector![edge_index]);
                // • • • • •

                illicit::Layer::new().offer(path.clone()).enter(|| {
                    debug_assert_eq!(*illicit::expect::<EPath<Self::Ix>>(), path);

                    root_ei.refresh_for_use(edge_refreshers);

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
    fn handle_children_in_topo(
        &self,
        replace_id: Option<&Self::Ix>,
        tree_element: &'_ crate::GTreeBuilderElement<Message>,
    ) {
        debug!("handle_children");
        let parent_nix = (*illicit::expect::<NodeIndex<Self::Ix>>()).clone();
        match tree_element {
            //
            GTreeBuilderElement::Layer(org_id, edge_refreshers, children_list) => {
                let id = replace_id.unwrap_or(org_id);
                info!("\n handle children [Layer]: org_id: {:?},  id : {:?}", org_id, id);
                let _span =
                    trace_span!("-> handle_children_in_topo [layer] ", ?id, ?parent_nix).entered();

                trace!("\nhandle_children:\n{:?}==>{:#?}", &id, &children_list);


                //NOTE current node 因为dyn 节点 插入新节点时候 没有删除原存在节点,所以会重复走 handle_children_in_topo, 当前这里处理是ID存在就全部跳过
                //TODO make GTreeBuilderElement all type same like this
                //TODO 处理如下更新时候 内容变更情况
                // dyn_tree2.set_with_once(move|dict| {
                //     dict.update(
                //         "aa3".to_string(),
                //         new_dom
                //     )
                // });
                if self.borrow().nodes_contains_key(id){
                    warn!("children:Layer id:{} already exists , pass",id);
                    return;
                }
                // node index
                //NOTE current node 因为`or_insert_node` 1个ID 只插入一次
        

                let nix: NodeIndex<Self::Ix> = node_index(id.clone());
                let edge_index = EdgeIndex::new(parent_nix, nix.clone());
                GraphNodeBuilder::new(self.clone())
                .and_key(id.clone())
                .and_gel_sa(StateAnchor::constant(Layer::<Message>::new(id.clone()).into()))
                .and_incoming_eix_set([edge_index.clone()].into_iter().collect())
                .and_outgoing_eix_set(IndexSet::with_capacity_and_hasher(
                    2,
                    BuildHasherDefault::<CustomHasher>::default(),
                ))
                .build_in_topo();


                trace!("\nhandle_children:\n inserted node: {:#?}",&nix);


                // edge
                let mut new_def_ei = self
                    .setup_default_edge_in_topo(edge_index)
                    .unwrap();
                trace!("\nhandle_children:\n inserted edge: {:#?}",&nix);


                let path = (&*illicit::expect::<EPath<Self::Ix>>()).link_ref(nix.clone());

                illicit::Layer::new().offer(path.clone()).enter(|| {
                    debug_assert_eq!(*illicit::expect::<EPath<Self::Ix>>(), path.clone());
                    new_def_ei.refresh_for_use(edge_refreshers);

                    // next
                    #[cfg(debug_assertions)]
                    illicit::Layer::new().offer(nix.clone()).enter(|| {
                        debug_assert_eq!(*illicit::expect::<NodeIndex<Self::Ix>>(), nix.clone());
                        children_list
                            .iter()
                            .for_each(|child_layer| self.handle_children_in_topo(None,child_layer));
                    });
                    #[cfg(not(debug_assertions))]
                    illicit::Layer::new().offer(nix).enter(|| {
                        children_list
                            .iter()
                            .for_each(|child_layer| self.handle_children_in_topo(None,child_layer));
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
            GTreeBuilderElement::GElementTree(org_id, edge_refreshers, gel, children_list) => {
                let id = replace_id.unwrap_or(org_id);
                info!("\n handle children [GElementTree]: org_id: {:?},  id : {:?}", org_id, id);


                let _span =
                    trace_span!("-> handle_children [GElementTree] ", ?id, ?parent_nix).entered();

                //node index
             
                let nix: NodeIndex<Self::Ix> = node_index(id.clone());
                let edge_index = EdgeIndex::new(parent_nix, nix.clone());
                GraphNodeBuilder::new(self.clone())
                .and_key(id.clone())
                .and_gel_sa(StateAnchor::constant(gel.clone()))
                .and_incoming_eix_set([edge_index.clone()].into_iter().collect())
                .and_outgoing_eix_set(IndexSet::with_capacity_and_hasher(
                    2,
                    BuildHasherDefault::<CustomHasher>::default(),
                ))
                .build_in_topo();

                //edge
                let mut new_def_ei = self
                    .setup_default_edge_in_topo(edge_index)
                    .unwrap();

                let path = (&*illicit::expect::<EPath<Self::Ix>>()).link_ref(nix.clone());

                illicit::Layer::new().offer(path.clone()).enter(|| {
                    debug_assert_eq!(*illicit::expect::<EPath<Self::Ix>>(), path.clone());
                    new_def_ei.refresh_for_use(edge_refreshers);

                    //next
                    illicit::Layer::new().offer(nix.clone()).enter(|| {
                        // #[cfg(debug_assertions)]
                        debug_assert_eq!(*illicit::expect::<NodeIndex<Self::Ix>>(), nix.clone());

                        for child_gtree_builder in children_list.iter() {
                            self.handle_children_in_topo(None,child_gtree_builder);
                        }
                    });
                });
            }
            //TODO _edge_refresher use for  inject element
            GTreeBuilderElement::Dyn(org_id,_edge_refresher,sa_dict_gbe) => {
                let id = replace_id.unwrap_or(org_id);
                info!("\n handle children [Dyn]: org_id: {:?},  id : {:?}", org_id, id);

                let _span = trace_span!("-> handle_children [SA] ", ?parent_nix).entered();
                debug!("builder:: GTreeBuilderElement::Dyn id:{:?}",id);


                let this = self.clone();
                let this2 = self.clone();

                let current_path = (&*illicit::expect::<EPath<Self::Ix>>()).clone();

                // let parent_nix = (*illicit::expect::<NodeIndex<Self::Ix>>()).clone();
                // let update_id = TopoKey::new(topo::CallId::current());
                let update_id = TopoKey::new(topo::root(||topo::call_in_slot(sa_dict_gbe.id(),topo::CallId::current)));
                //TODO move it , for  use StateAnchor
                sa_dict_gbe
                    .insert_before_fn(
                        update_id,
                        move |_skip, current, new_v| {
                            //// TODO use graph find all parent
                            // let parent = parent_nix.clone();
                            debug!("builder::running before_fn");
                            trace!("builder::before_fn: is current has? {}",&current.is_some());
                            if let Some(old_data) = current {
                                let old_data_clone = (**old_data).clone();
                                
                                let new_v_removed =old_data_clone.relative_complement(new_v.clone());
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
                    topo::root(||{
                        call_in_slot(id,||{

                            call_in_slot(k,||
                                {
                                    self.handle_children_in_topo(Some(k),v);
                                }
                            );

                        });
                    });
                }

                // value.iter().for_each(|(_k, v)| {
                //     debug!("builder:: sa_dict_gbe handle_children_in_topo call");

                //     self.handle_children_in_topo(v);
                // });
            }

            GTreeBuilderElement::RefreshUse(org_id, u) => {
                let id = replace_id.unwrap_or(org_id);
                info!("\n handle children [RefreshUse]: org_id: {:?},  id : {:?}", org_id, id);

                let _span =
                    trace_span!("-> handle_children_in_topo [RefreshUse] ", ?id, ?parent_nix)
                        .entered();

                //node index
              
                let nix: NodeIndex<Self::Ix> = node_index(id.clone());
                let edge_index = EdgeIndex::new(parent_nix, nix);
                GraphNodeBuilder::new(self.clone())
                .and_key(id.clone())
                .and_gel_sa(StateAnchor::constant(u.clone().into()))
                .and_incoming_eix_set([edge_index.clone()].into_iter().collect())
                .and_outgoing_eix_set(IndexSet::with_hasher(
                    BuildHasherDefault::<CustomHasher>::default(),
                ))
                .build_in_topo();

                let _ei = self
                    .setup_default_edge_in_topo(edge_index)
                    .unwrap();
            }

            GTreeBuilderElement::Cl(org_id, dyn_fn) => {
                let id = replace_id.unwrap_or(org_id);
                info!("\n handle children [Cl]: org_id: {:?},  id : {:?}", org_id, id);

                let _span = trace_span!(
                    "-> handle_children_in_topo [Cl] dyn_fn running",
                    ?id,
                    ?parent_nix
                )
                .entered();

                dyn_fn();
            }

            // TODO make RC remove most clones
            GTreeBuilderElement::Event(org_id, callback) => {
                debug!("GTreeBuilderElement::Event : {:?} {:?}", org_id,replace_id);
                let id = replace_id.unwrap_or(org_id);
                info!("\n handle children [Event]: org_id: {:?},  id : {:?}", org_id, id);

                let _span =
                    trace_span!("-> handle_children_in_topo [Event] ", ?id, ?parent_nix).entered();

                // TODO: make all into() style?
                // node index
              
                let nix: NodeIndex<Self::Ix> = node_index(id.clone());
                let edge_index = EdgeIndex::new(parent_nix, nix);
                GraphNodeBuilder::new(self.clone())
                .and_key(id.clone())
                .and_gel_sa(StateAnchor::constant(callback.clone().into()))
                .and_incoming_eix_set([edge_index.clone()].into_iter().collect())
                .and_outgoing_eix_set(IndexSet::with_hasher(
                    BuildHasherDefault::<CustomHasher>::default(),
                ))
                .build_in_topo();

                let _ei = self
                    .setup_default_edge_in_topo(edge_index)
                    .unwrap();
            } // GTreeBuilderElement::GenericTree(id, edge_refreshers, dyn_gel, refreshers) => {
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
              //         ei.refresh_use(edge_refreshers);

              //         //next
              //         #[cfg(debug_assertions)]
              //         illicit::Layer::new().offer(nix.clone()).enter(|| {
              //             assert_eq!(*illicit::expect::<NodeIndex<Self::Ix>>(), nix.clone());
              //             refreshers
              //                 .iter()
              //                 .for_each(|child_layer| self.handle_children_in_topo(child_layer));
              //         });
              //         #[cfg(not(debug_assertions))]
              //         illicit::Layer::new().offer(nix).enter(|| {
              //             refreshers
              //                 .iter()
              //                 .for_each(|child_layer| self.handle_children_in_topo(child_layer));
              //         });
              //     });
              // }
        };
    }
}
