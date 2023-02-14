/*
 * @Author: Rais
 * @Date: 2023-01-20 00:02:37
 * @LastEditTime: 2023-02-14 17:53:15
 * @LastEditors: Rais
 * @Description:
 */

use emg::{Direction, EdgeIndex};

use super::{GraphEditManyMethod, GraphEditor};

// impl<Message, Ix> GraphEditManyMethod for GraphType<Message, Ix>
// where
//     Ix: std::hash::Hash
//         + std::clone::Clone
//         + std::cmp::Ord
//         + std::default::Default
//         + std::fmt::Debug,
// {
//     type Ix = Ix;
//     fn edge_plug_edit(&self, who: &EdgeIndex<Ix>, dir: Direction, to: Ix) {
//         self.edge_plug_edit(who, dir, to);
//     }

//     fn edge_path_node_change_edge(&mut self) {
//         todo!("edge_path_node_change_edge")
//     }
// }

impl<Message, Ix> GraphEditManyMethod for GraphEditor<Message, Ix>
where
    Ix: std::hash::Hash + Clone + Ord + Default + std::fmt::Debug,
{
    type Ix = Ix;
    fn edge_plug_edit(&self, who: &EdgeIndex<Ix>, dir: Direction, to: Ix) {
        self.borrow().edge_plug_edit(who, dir, to)
    }

    fn edge_path_node_change_edge(&mut self) {
        todo!("edge_path_node_change_edge")
    }
}

//test

#[cfg(test)]
// #[allow(unused)]
mod test {
    use std::{cell::RefCell, path::Path, rc::Rc};

    use emg::{edge_index, edge_index_no_source, Direction::Incoming};
    use emg_common::{px, IdStr};

    use emg_state::{use_state, StateAnchor};

    use crate::{
        g_tree_builder::{GraphEdgeBuilder, GraphNodeBuilder},
        graph_edit::{EdgeMode, GraphEdit},
        widget::Layer,
        GTreeBuilderFn, GraphType,
    };

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    enum Message {}

    #[ignore = "reason"]
    #[test]

    fn test_edge_path_change_source() {
        insta::with_settings!({snapshot_path => Path::new("./insta")},{
         let emg_graph = GraphType::<Message>::default();
         let emg_graph_rc_refcell = Rc::new(RefCell::new(emg_graph));

         // ────────────────────────────────────────────────────────────────────────────────
         let root_id = IdStr::new_inline("root");
         let root_edge_ix = edge_index_no_source(root_id.clone());
         // node ────────────────────────────────────────────────────────────────────────────────

         GraphNodeBuilder::new(root_id.clone())
             .with_gel_sa(use_state(||StateAnchor::constant(Rc::new(
                 Layer::<Message>::new(root_id.clone()).into(),
             ))))
             .with_incoming_eix_set([root_edge_ix.clone()].into_iter().collect())
             .with_outgoing_eix_set_with_default_capacity(5)
             .build_in_topo(&emg_graph_rc_refcell);
         // edge ─────────────────────────────────────────────────────

         let  _root_ei = GraphEdgeBuilder::new(root_edge_ix.clone())
             .with_size((px(1000), px(1000)))
             .build_in_topo(&emg_graph_rc_refcell)
             .unwrap();

         // =======================================================
         // ────────────────────────────────────────────────────────────────────────────────
         let id = IdStr::new_inline("a");
         let edge_ix = edge_index("root", "a");
         // node ────────────────────────────────────────────────────────────────────────────────

         GraphNodeBuilder::new(id.clone())
             .with_gel_sa(use_state(||StateAnchor::constant(Rc::new(
                 Layer::<Message>::new(id.clone()).into(),
             ))))
             .with_incoming_eix_set([edge_ix.clone()].into_iter().collect())
             .with_outgoing_eix_set_with_default_capacity(5)
             .build_in_topo(&emg_graph_rc_refcell);
         // edge ─────────────────────────────────────────────────────

         let _e_item = GraphEdgeBuilder::new(edge_ix.clone())
         .with_size((px(100), px(100)))
             .build_in_topo(&emg_graph_rc_refcell)
             .unwrap();
         // ─────────────────────────────────────────────────────────────────────────────

         // =======================================================
         // ────────────────────────────────────────────────────────────────────────────────
         let id = IdStr::new_inline("b");
         let edge_ix = edge_index("root", "b");
         // node ────────────────────────────────────────────────────────────────────────────────

         GraphNodeBuilder::new(id.clone())
             .with_gel_sa(use_state(||StateAnchor::constant(Rc::new(
                 Layer::<Message>::new(id.clone()).into(),
             ))))
             .with_incoming_eix_set([edge_ix.clone()].into_iter().collect())
             .with_outgoing_eix_set_with_default_capacity(5)
             .build_in_topo(&emg_graph_rc_refcell);
         // edge ─────────────────────────────────────────────────────

         let _e_item = GraphEdgeBuilder::new(edge_ix.clone())
         .with_size((px(200), px(200)))
             .build_in_topo(&emg_graph_rc_refcell)
             .unwrap();
         // ─────────────────────────────────────────────────────────────────────────────

         // =======================================================
         // ────────────────────────────────────────────────────────────────────────────────
         let id = IdStr::new_inline("c");
         let edge_ix = edge_index("a", "c");
         // node ────────────────────────────────────────────────────────────────────────────────

         GraphNodeBuilder::new(id.clone())
             .with_gel_sa(use_state(||StateAnchor::constant(Rc::new(
                 Layer::<Message>::new(id.clone()).into(),
             ))))
             .with_incoming_eix_set([edge_ix.clone()].into_iter().collect())
             .with_outgoing_eix_set_with_default_capacity(5)
             .build_in_topo(&emg_graph_rc_refcell);
         // edge ─────────────────────────────────────────────────────

         let _e_item = GraphEdgeBuilder::new(edge_ix.clone())
         .with_size((px(30), px(30)))
             .build_in_topo(&emg_graph_rc_refcell)
             .unwrap();
         // ─────────────────────────────────────────────────────────────────────────────
        { let x = &*emg_graph_rc_refcell.borrow();
         insta::assert_display_snapshot!("graph_new",x);

        }
         {
            //NOTE a->c to b->c

            emg_graph_rc_refcell.editor()
             .edit::<EdgeMode>()
             .moving(edge_index("a", "c"),Incoming, "b");
         }
         // ─────────────────────────────────────────────────────

         // println!("{:#?}", &emg_graph_rc_refcell);
         let x = &*emg_graph_rc_refcell.borrow();
        //  println!("{}", x);
             insta::assert_display_snapshot!("graph_moved",x);
         });
    }
}
