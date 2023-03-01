use std::{clone::Clone, cmp::Eq, hash::Hash, iter::Iterator, marker::PhantomData};

use crate::graph::{dir::HasDir, edges::NodeEdgesIter, Direction, NodeIndex};

use super::edges::NodeEdgesConsumingIter;

pub struct NodeNeighborsIter<EdgeIter>
where
    EdgeIter: Iterator + HasDir,
{
    edge_iter: EdgeIter,
}

impl<EdgeIter> NodeNeighborsIter<EdgeIter>
where
    EdgeIter: Iterator + HasDir,
{
    pub fn new(edge_iter: EdgeIter) -> Self {
        Self { edge_iter }
    }

    pub fn edges(self) -> EdgeIter {
        self.edge_iter
    }
}

// ────────────────────────────────────────────────────────────────────────────────
// ────────────────────────────────────────────────────────────────────────────────

// pub trait IntoNeighborsNode<'a, IterItem> {
//     type NeighborsIter: Iterator<Item = IterItem>;
//     fn neighbors_consuming_iter2(self, a: &'a IterItem, dir: Direction) -> Self::NeighborsIter;
// }

// impl<'a, N, E, > IntoNeighborsNode<'a, NodeIndex> for &'a Graph<N, E>
// where
//     E: Clone,
//     N: Clone,
// {
//     type NeighborsIter = NeighborsNodeIter<'a, >;
//     fn neighbors_consuming_iter2(self, n: &'a NodeIndex, dir: Direction) -> Self::NeighborsIter {
//         Graph::neighbors_node_iter(self, n, dir)
//     }
// }

// ────────────────────────────────────────────────────────────────────────────────
// ────────────────────────────────────────────────────────────────────────────────
impl<EdgeIter> HasDir for NodeNeighborsIter<EdgeIter>
where
    EdgeIter: Iterator + HasDir,
{
    fn dir(&self) -> Direction {
        self.edge_iter.dir()
    }
}

impl<'a> Iterator for NodeNeighborsIter<NodeEdgesIter<'a>> {
    type Item = &'a NodeIndex;

    fn next(&mut self) -> Option<Self::Item> {
        self.edge_iter
            .next()
            .and_then(|e| e.nix_by_dir(self.dir()).clone())
    }
}

impl Iterator for NodeNeighborsIter<NodeEdgesConsumingIter> {
    type Item = NodeIndex;

    fn next(&mut self) -> Option<Self::Item> {
        self.edge_iter.next().and_then(|e| match self.dir() {
            Direction::Incoming => e.0 .0,
            Direction::Outgoing => e.1 .0,
        })
    }
}
impl From<NodeEdgesConsumingIter> for NodeNeighborsIter<NodeEdgesConsumingIter> {
    fn from(edges: NodeEdgesConsumingIter) -> Self {
        Self { edge_iter: edges }
    }
}

impl<'a> From<NodeEdgesIter<'a>> for NodeNeighborsIter<NodeEdgesIter<'a>> {
    fn from(edges: NodeEdgesIter<'a>) -> Self {
        Self { edge_iter: edges }
    }
}

#[cfg(test)]
mod neighbors_test {

    use emg_common::IdStr;

    use crate::graph::{edges::NodeEdgesConsumingIter, Graph, Incoming, NodeIndex, Outgoing};

    use super::NodeNeighborsIter;
    #[test]
    #[allow(unused)]
    fn neighbors() {
        let mut g1: Graph<IdStr, &'static str> = Graph::empty();
        let ww_nix = g1.insert_node_in_topo_only(IdStr::from("ww"), IdStr::from("ww_item"));
        let xx_nix = g1.insert_node_in_topo_only(IdStr::from("xx"), IdStr::from("xx_item"));
        let xx_nix2 = g1.insert_node_in_topo_only(IdStr::from("xx2"), IdStr::from("xx_item2"));
        let xx_nix3 = g1.insert_node_in_topo_only(IdStr::from("xx3"), IdStr::from("xx_item3"));
        // ─────────────────────────────────────────────────────────────────
        // ────────────────────────────────────────────────────────────────────────────────

        let op_eix1 = g1
            .unused_insert_update_edge_in_topo(&ww_nix, &xx_nix, "ww->xx:item")
            .unwrap();
        let op_eix1 = g1
            .unused_insert_update_edge_in_topo(&ww_nix, &xx_nix2, "ww->xx:item")
            .unwrap();
        let op_eix1 = g1
            .unused_insert_update_edge_in_topo(&ww_nix, &xx_nix3, "ww->xx:item")
            .unwrap();

        let op_eix2 = g1
            .unused_insert_update_edge_in_topo(&xx_nix, &ww_nix, "xx->ww:item")
            .unwrap();

        // ─────────────────────────────────────────────────────────────────
        // ────────────────────────────────────────────────────────────────────────────────
        println!("edges: {:#?}", g1.raw_edges());
        for nix in g1.neighbors_consuming_iter(&ww_nix, Outgoing) {
            println!("{:?}", nix)
        }
        println!("======================");
        let mut it = g1.neighbors_consuming_iter(&ww_nix, Outgoing);

        println!("1:{:?}", &it.next());
        println!("2:{:?}", &it.next());
        println!("3:{:?}", &it.next());
        println!("4:{:?}", &it.next());

        println!("======================");

        println!("======================");
        let mut edge_it = g1.edges_consuming_iter(&ww_nix, Outgoing);

        println!("edge 1     {:?}", &edge_it.next());
        let e1node = &edge_it.node().unwrap();
        println!("edge 1 node() :{:?}", e1node);
        let mut n_iter: NodeNeighborsIter<NodeEdgesConsumingIter> = edge_it.into();
        println!("edge iter -> n_iter      :{:?}", &n_iter.next());
        let mut e2edge_iter: NodeEdgesConsumingIter = n_iter.into();
        println!("edge back 3     :{:?}", &e2edge_iter.next());
        println!("edge back 4     :{:?}", &e2edge_iter.next());

        println!("======================");

        let ff = &g1;
        for nix in ff.neighbors_consuming_iter(&xx_nix, Incoming) {
            println!("{:?}", nix)
        }
        let err_nix = NodeIndex(IdStr::from("123"));
        // for nix in g1.neighbors_consuming_iter(&err_nix, Incoming) {
        //     println!("err never show :{:?}", nix)
        // }
    }
    #[test]
    #[allow(unused)]
    fn neighbors2() {
        let mut g = Graph::empty();
        let l1_nix = g.insert_node_in_topo_only("1", "1");
        let l1_1_nix = g.insert_node_in_topo_only("1.1", "1.1");
        let l1_2_nix = g.insert_node_in_topo_only("1.2", "1.2");
        let l1_3_nix = g.insert_node_in_topo_only("1.3", "1.3");

        let l1_1_1_nix = g.insert_node_in_topo_only("1.1.1", "1.1.1");
        let l1_1_2_nix = g.insert_node_in_topo_only("1.1.2", "1.1.2");
        let l1_1_3_nix = g.insert_node_in_topo_only("1.1.3", "1.1.3");

        let l1_2_1_nix = g.insert_node_in_topo_only("1.2.1", "1.2.1");
        let l1_2_2_nix = g.insert_node_in_topo_only("1.2.2", "1.2.2");
        let l1_2_3_nix = g.insert_node_in_topo_only("1.2.3", "1.2.3");

        let l1_3_1_nix = g.insert_node_in_topo_only("1.3.1", "1.3.1");
        let l1_3_2_nix = g.insert_node_in_topo_only("1.3.2", "1.3.2");
        let l1_3_3_nix = g.insert_node_in_topo_only("1.3.3", "1.3.3");
        // ─────────────────────────────────────────────────────────────────
        // ────────────────────────────────────────────────────────────────────────────────

        let op_eix1 = g
            .unused_insert_update_edge_in_topo(&l1_nix, &l1_1_nix, "1->1.1 :edge_item")
            .unwrap();
        let op_eix1 = g
            .unused_insert_update_edge_in_topo(&l1_nix, &l1_2_nix, "1->1.2 :edge_item")
            .unwrap();
        let op_eix1 = g
            .unused_insert_update_edge_in_topo(&l1_nix, &l1_3_nix, "1->1.3 :edge_item")
            .unwrap();

        let op_eix1 = g
            .unused_insert_update_edge_in_topo(&l1_1_nix, &l1_1_1_nix, "1.1->1.1.1 :edge_item")
            .unwrap();
        let op_eix1 = g
            .unused_insert_update_edge_in_topo(&l1_1_nix, &l1_1_2_nix, "1.1->1.1.2 :edge_item")
            .unwrap();
        let op_eix1 = g
            .unused_insert_update_edge_in_topo(&l1_1_nix, &l1_1_3_nix, "1.1->1.1.3 :edge_item")
            .unwrap();

        let op_eix1 = g
            .unused_insert_update_edge_in_topo(&l1_2_nix, &l1_2_1_nix, "1.2->1.2.1 :edge_item")
            .unwrap();
        let op_eix1 = g
            .unused_insert_update_edge_in_topo(&l1_2_nix, &l1_2_2_nix, "1.2->1.2.2 :edge_item")
            .unwrap();
        let op_eix1 = g
            .unused_insert_update_edge_in_topo(&l1_2_nix, &l1_2_3_nix, "1.2->1.2.3 :edge_item")
            .unwrap();

        let op_eix1 = g
            .unused_insert_update_edge_in_topo(&l1_3_nix, &l1_3_1_nix, "1.3->1.3.1 :edge_item")
            .unwrap();
        let op_eix1 = g
            .unused_insert_update_edge_in_topo(&l1_3_nix, &l1_3_2_nix, "1.3->1.3.2 :edge_item")
            .unwrap();
        let op_eix1 = g
            .unused_insert_update_edge_in_topo(&l1_3_nix, &l1_3_3_nix, "1.3->1.3.3 :edge_item")
            .unwrap();

        // ─────────────────────────────────────────────────────────────────
        // ────────────────────────────────────────────────────────────────────────────────

        for nix in g.neighbors_consuming_iter(&l1_nix, Outgoing) {
            println!("{:?}", nix)
        }
        println!("======================");
        let mut it = g.neighbors_consuming_iter(&l1_nix, Outgoing);
        println!("1:{:?}", &it.next());
        let n2 = &it.next().unwrap();
        println!("2:{:?}", n2);
        let mut it2 = g.neighbors_consuming_iter(n2, Outgoing);

        for nix in it2 {
            println!("it2   :   {:?}", nix)
        }

        println!("3:{:?}", &it.next());
        println!("4:{:?}", &it.next());

        println!("======================");

        println!("======================");
        let mut edge_it = g.edges_consuming_iter(&l1_nix, Outgoing);

        println!("edge 1     {:?}", &edge_it.next());
        let e1node = &edge_it.node().unwrap();
        println!("edge 1 node() :{:?}", e1node);

        for nix in g.neighbors_consuming_iter(e1node, Outgoing) {
            println!("edge1node iter   :   {:?}", nix)
        }

        let mut n_iter: NodeNeighborsIter<NodeEdgesConsumingIter> = edge_it.into();
        println!("edge iter -> n_iter      :{:?}", &n_iter.next());
        let mut e2edge_iter: NodeEdgesConsumingIter = n_iter.into();
        println!("edge back 3     :{:?}", &e2edge_iter.next());
        println!("edge back 4     :{:?}", &e2edge_iter.next());

        println!("======================");
    }
}
