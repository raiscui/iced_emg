/*
 * @Author: Rais
 * @Date: 2021-02-26 14:57:02
 * @LastEditTime: 2021-09-08 15:30:52
 * @LastEditors: Rais
 * @Description:
 */

use crate::emg_runtime::{EventNode, Layer};
use crate::{GElement, GraphType, NodeIndex};
use emg::{edge_index_no_source, im_rc::vector, Edge, EdgeIndex};
use emg_core::GenericSize;
use emg_layout::{global_height, global_width, EPath, EmgEdgeItem, GenericSizeAnchor};
use emg_refresh::{RefreshFor, RefreshForUse};
use emg_state::{topo, use_state};
use std::rc::Rc;
use tracing::{instrument, trace, trace_span};
#[allow(dead_code)]
pub enum GTreeBuilderElement<'a, Message, Ix = String>
where
    Ix: Clone + std::hash::Hash + Ord + Default + 'static,
    Message: 'static,
{
    Layer(
        Ix,
        Vec<Box<dyn RefreshFor<EmgEdgeItem<Ix>>>>,
        Vec<GTreeBuilderElement<'a, Message, Ix>>,
    ),
    // El(Ix, Element<'a, Message>),
    GElementTree(
        Ix,
        Vec<Box<dyn RefreshFor<EmgEdgeItem<Ix>>>>,
        GElement<'a, Message>,
        Vec<GTreeBuilderElement<'a, Message, Ix>>,
    ),
    RefreshUse(Ix, Rc<dyn RefreshFor<GElement<'a, Message>> + 'a>),
    Cl(Ix, Box<dyn Fn() + 'a>),
    Event(Ix, EventNode<Message>),
    // GenericTree(
    //     Ix,
    //     Vec<Box<dyn RefreshFor<EmgEdgeItem<Ix>>>>,
    //     Box<dyn DynGElement<'a, Message> + 'static>,
    //     Vec<GTreeBuilderElement<'a, Message, Ix>>,
    // ),
}

// impl<'a, Message>
//     From<(
//         String,
//         Vec<Box<dyn RefreshFor<EmgEdgeItem<String>>>>,
//         Result<GElement<'a, Message>, GTreeBuilderElement<'a, Message>>,
//         Vec<GTreeBuilderElement<'a, Message>>,
//     )> for GTreeBuilderElement<'a, Message>
// // where
// // Ix: Clone + std::hash::Hash + Ord + Default + 'static,
// {
//     fn from(
//         f: (
//             String,
//             Vec<Box<dyn RefreshFor<EmgEdgeItem<String>>>>,
//             Result<GElement<'a, Message>, GTreeBuilderElement<'a, Message>>,
//             Vec<GTreeBuilderElement<'a, Message>>,
//         ),
//     ) -> Self {
//         match f.2 {
//             Ok(ge) => Self::GElementTree(f.0, f.1, ge, f.3),
//             Err(gtbe) => Self::Layer(f.0, f.1, vec![gtbe]),
//         }
//     }
// }

impl<'a, Message: std::fmt::Debug + std::clone::Clone> std::fmt::Debug
    for GTreeBuilderElement<'a, Message>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GTreeBuilderElement::Layer(id, _, children_list) => {
                let edge_str = "with-Edge-Vector";
                f.debug_tuple("GTreeBuilderElement::Layer")
                    .field(id)
                    .field(&edge_str)
                    .field(children_list)
                    .finish()
            }
            // GTreeBuilderElement::El(id, el) => f
            //     .debug_tuple("GTreeBuilderElement::El")
            //     .field(id)
            //     .field(el)
            //     .finish(),
            GTreeBuilderElement::GElementTree(id, _, gel, updaters) => {
                let edge_str = "with-Edge-Vector";

                f.debug_tuple("GTreeBuilderElement::WhoWithUpdater")
                    .field(id)
                    .field(&edge_str)
                    .field(gel)
                    .field(updaters)
                    .finish()
            }
            GTreeBuilderElement::RefreshUse(id, _) => {
                let updater = "Box<dyn RefreshFor<GElement<'a, Message>>>";
                f.debug_tuple("GTreeBuilderElement::Updater")
                    .field(id)
                    .field(&updater)
                    .finish()
            }
            GTreeBuilderElement::Cl(id, _) => {
                f.debug_tuple("GTreeBuilderElement::Cl").field(id).finish()
            }
            GTreeBuilderElement::Event(id, e) => f
                .debug_tuple("GTreeBuilderElement::Event")
                .field(id)
                .field(&e)
                .finish(),
            // GTreeBuilderElement::GenericTree(id, _, dyn_gel, updaters) => {
            //     let edge_str = "with-Edge-Vector";
            //     let dyn_name = format!("DynGElement({})", dyn_gel.type_name());
            //     // let name = "DynGElement";
            //     f.debug_tuple("GTreeBuilderElement::GenericTree")
            //         .field(id)
            //         .field(&edge_str)
            //         .field(&dyn_name)
            //         .field(updaters)
            //         .finish()
            // }
        }
    }
}
pub trait GTreeBuilderFn<'a, Message>
where
    Self::Ix: Clone + Default + std::hash::Hash + Ord,
{
    type Ix;

    /// # Errors
    ///
    /// Will return `Err` if node insert `edge_index` falls
    fn setup_wh_edge_in_topo<T: Into<f64> + std::fmt::Debug>(
        &mut self,
        ei: EdgeIndex<Self::Ix>,

        w: T,
        h: T,
    ) -> Result<EmgEdgeItem<Self::Ix>, String>;

    /// # Errors
    ///
    /// Will return `Err` if node insert `edge_index` falls
    fn setup_edge_in_topo(
        &mut self,
        edge_index: EdgeIndex<Self::Ix>,

        size: (GenericSizeAnchor, GenericSizeAnchor),
        origin: (GenericSizeAnchor, GenericSizeAnchor, GenericSizeAnchor),
        align: (GenericSizeAnchor, GenericSizeAnchor, GenericSizeAnchor),
    ) -> Result<EmgEdgeItem<Self::Ix>, String>;

    /// # Errors
    ///
    /// Will return `Err` if node insert `edge_index` falls
    fn setup_default_edge_in_topo(
        &mut self,
        edge_index: EdgeIndex<Self::Ix>,
    ) -> Result<EmgEdgeItem<Self::Ix>, String>;

    fn handle_root_in_topo(&mut self, tree_layer: &GTreeBuilderElement<'a, Message>);
    fn handle_children_in_topo(&mut self, tree_layer: &'_ GTreeBuilderElement<'a, Message>);
}

impl<'a, Message> GTreeBuilderFn<'a, Message> for GraphType<'a, Message>
where
    Message: std::clone::Clone + std::fmt::Debug,
    // Ix: std::hash::Hash
    //     + std::clone::Clone
    //     + std::cmp::Ord
    //     + std::default::Default
    //     + std::fmt::Debug,
{
    type Ix = String;
    // TODO: use builder ?
    #[topo::nested]
    #[instrument(skip(self))]
    fn setup_wh_edge_in_topo<T: Into<f64> + std::fmt::Debug>(
        &mut self,
        ei: EdgeIndex<Self::Ix>,
        w: T,
        h: T,
    ) -> Result<EmgEdgeItem<Self::Ix>, String> {
        self.nodes_connect_eix(&ei).ok_or("node insert eix fails")?;
        let source = use_state(ei.source_nix().as_ref().cloned());
        let target = use_state(ei.target_nix().as_ref().cloned());
        let edge_item = EmgEdgeItem::default_with_wh_in_topo(
            source.watch(),
            target.watch(),
            self.get_raw_edges_watch(),
            w,
            h,
        );
        self.just_insert_edge(ei, Edge::new(source, target, edge_item.clone()));

        Ok(edge_item)
    }

    #[topo::nested]
    #[instrument(skip(self, size, origin, align))]
    fn setup_edge_in_topo(
        &mut self,
        ei: EdgeIndex<Self::Ix>,
        size: (GenericSizeAnchor, GenericSizeAnchor),
        origin: (GenericSizeAnchor, GenericSizeAnchor, GenericSizeAnchor),
        align: (GenericSizeAnchor, GenericSizeAnchor, GenericSizeAnchor),
    ) -> Result<EmgEdgeItem<Self::Ix>, String> {
        self.nodes_connect_eix(&ei).ok_or("node insert eix fails")?;

        let source = use_state(ei.source_nix().as_ref().cloned());
        let target = use_state(ei.target_nix().as_ref().cloned());
        let edge_item = EmgEdgeItem::new_in_topo(
            source.watch(),
            target.watch(),
            self.get_raw_edges_watch(),
            size,
            origin,
            align,
        );
        self.just_insert_edge(ei, Edge::new(source, target, edge_item.clone()));
        Ok(edge_item)
    }
    // TODO: use builder ?
    #[topo::nested]
    #[instrument(skip(self))]
    fn setup_default_edge_in_topo(
        &mut self,
        ei: EdgeIndex<Self::Ix>,
    ) -> Result<EmgEdgeItem<Self::Ix>, String> {
        self.nodes_connect_eix(&ei).ok_or("node insert eix fails")?;

        let source = use_state(ei.source_nix().as_ref().cloned());
        let target = use_state(ei.target_nix().as_ref().cloned());
        let edge_item = EmgEdgeItem::default_in_topo(
            source.watch(),
            target.watch(),
            self.get_raw_edges_watch(),
        );
        self.just_insert_edge(ei, Edge::new(source, target, edge_item.clone()));
        Ok(edge_item)
    }

    #[topo::nested]
    fn handle_root_in_topo(&mut self, tree_layer: &GTreeBuilderElement<'a, Message>)
    //  where
    // Message: Clone + std::fmt::Debug,
    {
        match tree_layer {
            GTreeBuilderElement::Layer(root_id, edge_refreshers, children_list) => {
                let _span = trace_span!("=> handle_root [layer] ",%root_id).entered();
                trace!("{:?}==>{:#?}", &root_id, &children_list);
                // ─────────────────────────────────────────────────────────────────

                let nix = self.insert_node(root_id.clone(), Layer::<Message>::new(root_id).into());

                let width = global_width();
                let height = global_height();
                let mut root_ei = self
                    //TODO: bind browser w h.
                    .setup_edge_in_topo(
                        edge_index_no_source(root_id.clone()),
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

                let path = EPath::<Self::Ix>::new(vector![edge_index_no_source(root_id.clone())]);

                illicit::Layer::new().offer(path.clone()).enter(|| {
                    debug_assert_eq!(*illicit::expect::<EPath<Self::Ix>>(), path);

                    root_ei.refresh_for_use(edge_refreshers);

                    illicit::Layer::new().offer(nix.clone()).enter(|| {
                        assert_eq!(*illicit::expect::<NodeIndex<Self::Ix>>(), nix.clone());
                        trace!("{:?}", *illicit::expect::<NodeIndex<Self::Ix>>());
                        children_list
                            .iter()
                            .for_each(|child_layer| self.handle_children_in_topo(child_layer));
                    });
                });
            }
            _ => {
                panic!("not allow this , first element must layer ");
            }
        };
    }
    #[topo::nested]
    fn handle_children_in_topo(&mut self, tree_layer: &'_ GTreeBuilderElement<'a, Message>) {
        let parent_nix = &*illicit::expect::<NodeIndex<Self::Ix>>();
        match tree_layer {
            //
            GTreeBuilderElement::Layer(id, edge_refreshers, children_list) => {
                let _span =
                    trace_span!("-> handle_children_in_topo [layer] ", ?id, ?parent_nix).entered();

                trace!("{:?}==>{:#?}", &id, &children_list);
                // node index
                let nix = self.insert_node(id.clone(), Layer::<Message>::new(id).into());

                // edge
                let mut ei = self
                    .setup_default_edge_in_topo(EdgeIndex::new(parent_nix.clone(), nix.clone()))
                    .unwrap();

                let path = (&*illicit::expect::<EPath<Self::Ix>>()).add_build(nix.clone());

                illicit::Layer::new().offer(path.clone()).enter(|| {
                    debug_assert_eq!(*illicit::expect::<EPath<Self::Ix>>(), path.clone());
                    ei.refresh_for_use(edge_refreshers);

                    // next
                    #[cfg(debug_assertions)]
                    illicit::Layer::new().offer(nix.clone()).enter(|| {
                        assert_eq!(*illicit::expect::<NodeIndex<Self::Ix>>(), nix.clone());
                        children_list
                            .iter()
                            .for_each(|child_layer| self.handle_children_in_topo(child_layer));
                    });
                    #[cfg(not(debug_assertions))]
                    illicit::Layer::new().offer(nix).enter(|| {
                        children_list
                            .iter()
                            .for_each(|child_layer| self.handle_children_in_topo(child_layer));
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
            GTreeBuilderElement::GElementTree(id, edge_refreshers, gel, refreshers) => {
                let _span =
                    trace_span!("-> handle_children [GElementTree] ", ?id, ?parent_nix).entered();

                //node index
                let nix = self.insert_node(id.clone(), gel.clone());

                //edge
                let mut ei = self
                    .setup_default_edge_in_topo(EdgeIndex::new(parent_nix.clone(), nix.clone()))
                    .unwrap();

                let path = (&*illicit::expect::<EPath<Self::Ix>>()).add_build(nix.clone());

                illicit::Layer::new().offer(path.clone()).enter(|| {
                    debug_assert_eq!(*illicit::expect::<EPath<Self::Ix>>(), path.clone());
                    ei.refresh_for_use(edge_refreshers);

                    //next
                    #[cfg(debug_assertions)]
                    illicit::Layer::new().offer(nix.clone()).enter(|| {
                        assert_eq!(*illicit::expect::<NodeIndex<Self::Ix>>(), nix.clone());
                        refreshers
                            .iter()
                            .for_each(|child_layer| self.handle_children_in_topo(child_layer));
                    });
                    #[cfg(not(debug_assertions))]
                    illicit::Layer::new().offer(nix).enter(|| {
                        refreshers
                            .iter()
                            .for_each(|child_layer| self.handle_children_in_topo(child_layer));
                    });
                });
            }

            GTreeBuilderElement::RefreshUse(id, u) => {
                let _span =
                    trace_span!("-> handle_children_in_topo [RefreshUse] ", ?id, ?parent_nix)
                        .entered();

                //node index
                let nix = self.insert_node(id.clone(), u.clone().into());

                let _ei = self
                    .setup_default_edge_in_topo(EdgeIndex::new(parent_nix.clone(), nix))
                    .unwrap();
            }

            GTreeBuilderElement::Cl(id, dyn_fn) => {
                let _span = trace_span!(
                    "-> handle_children_in_topo [Cl] dyn_fn running",
                    ?id,
                    ?parent_nix
                )
                .entered();

                dyn_fn();
            }

            // TODO make RC remove most clones
            GTreeBuilderElement::Event(id, callback) => {
                let _span =
                    trace_span!("-> handle_children_in_topo [Event] ", ?id, ?parent_nix).entered();

                // TODO: make all into() style?
                // node index
                let nix = self.insert_node(id.clone(), callback.clone().into());

                //edge
                // let e = format!("{} -> {}", parent_nix.index(), nix.index()).into();
                // trace!("{}", &e);
                // self.insert_update_edge(&parent_nix, &nix, e);

                let _ei = self
                    .setup_default_edge_in_topo(EdgeIndex::new(parent_nix.clone(), nix))
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
