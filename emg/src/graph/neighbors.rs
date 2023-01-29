use std::{clone::Clone, cmp::Eq, hash::Hash, iter::Iterator, marker::PhantomData};

use crate::graph::{dir::HasDir, edges::NodeEdgesIter, Direction, NodeIndex};

use super::edges::NodeEdgesConsumingIter;

pub struct NodeNeighborsIter<Ix, EdgeIter>
where
    EdgeIter: Iterator + HasDir,
{
    edge_iter: EdgeIter,
    ix: PhantomData<Ix>,
}

impl<Ix, EdgeIter> NodeNeighborsIter<Ix, EdgeIter>
where
    EdgeIter: Iterator + HasDir,
{
    pub fn new(edge_iter: EdgeIter) -> Self {
        Self {
            edge_iter,
            ix: PhantomData,
        }
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

// impl<'a, N, E, Ix> IntoNeighborsNode<'a, NodeIndex<Ix>> for &'a Graph<N, E, Ix>
// where
//     E: Clone,
//     Ix: Eq + Clone + Hash + Debug,
//     N: Clone,
// {
//     type NeighborsIter = NeighborsNodeIter<'a, Ix>;
//     fn neighbors_consuming_iter2(self, n: &'a NodeIndex<Ix>, dir: Direction) -> Self::NeighborsIter {
//         Graph::neighbors_node_iter(self, n, dir)
//     }
// }

// ────────────────────────────────────────────────────────────────────────────────
// ────────────────────────────────────────────────────────────────────────────────
impl<Ix, EdgeIter> HasDir for NodeNeighborsIter<Ix, EdgeIter>
where
    Ix: Clone + Hash + Eq,
    EdgeIter: Iterator + HasDir,
{
    fn dir(&self) -> Direction {
        self.edge_iter.dir()
    }
}

impl<Ix> Iterator for NodeNeighborsIter<Ix, NodeEdgesIter<'_, Ix>>
where
    Ix: Clone + Hash + Eq,
{
    type Item = NodeIndex<Ix>;

    fn next(&mut self) -> Option<Self::Item> {
        self.edge_iter
            .next()
            .and_then(|e| e.nix_by_dir(self.dir()).clone())
    }
}

impl<Ix> Iterator for NodeNeighborsIter<Ix, NodeEdgesConsumingIter<Ix>>
where
    Ix: Clone + Hash + Eq,
{
    type Item = NodeIndex<Ix>;

    fn next(&mut self) -> Option<Self::Item> {
        self.edge_iter.next().and_then(|e| match self.dir() {
            Direction::Incoming => e.0 .0,
            Direction::Outgoing => e.1 .0,
        })
    }
}
impl<Ix> From<NodeEdgesConsumingIter<Ix>> for NodeNeighborsIter<Ix, NodeEdgesConsumingIter<Ix>>
where
    Ix: Clone + Hash + Eq,
{
    fn from(edges: NodeEdgesConsumingIter<Ix>) -> Self {
        Self {
            edge_iter: edges,
            ix: PhantomData,
        }
    }
}

impl<'a, Ix> From<NodeEdgesIter<'a, Ix>> for NodeNeighborsIter<Ix, NodeEdgesIter<'a, Ix>>
where
    Ix: Clone + Hash + Eq,
{
    fn from(edges: NodeEdgesIter<'a, Ix>) -> Self {
        Self {
            edge_iter: edges,
            ix: PhantomData,
        }
    }
}

#[cfg(test)]
mod neighbors_test {

    use crate::graph::{edges::NodeEdgesConsumingIter, Graph, Incoming, NodeIndex, Outgoing};

    use super::NodeNeighborsIter;
    #[test]
    #[allow(unused)]
    fn neighbors() {
        let mut g1: Graph<String, &'static str, String> = Graph::empty();
        let ww_nix = g1.insert_node_in_topo(String::from("ww"), String::from("ww_item"));
        let xx_nix = g1.insert_node_in_topo(String::from("xx"), String::from("xx_item"));
        let xx_nix2 = g1.insert_node_in_topo(String::from("xx2"), String::from("xx_item2"));
        let xx_nix3 = g1.insert_node_in_topo(String::from("xx3"), String::from("xx_item3"));
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
        let mut edge_it = g1.deprecated_edges_consuming_iter(&ww_nix, Outgoing);

        println!("edge 1     {:?}", &edge_it.next());
        let e1node = &edge_it.node().unwrap();
        println!("edge 1 node() :{:?}", e1node);
        let mut n_iter: NodeNeighborsIter<String, NodeEdgesConsumingIter<String>> = edge_it.into();
        println!("edge iter -> n_iter      :{:?}", &n_iter.next());
        let mut e2edge_iter: NodeEdgesConsumingIter<String> = n_iter.into();
        println!("edge back 3     :{:?}", &e2edge_iter.next());
        println!("edge back 4     :{:?}", &e2edge_iter.next());

        println!("======================");

        let ff = &g1;
        for nix in ff.neighbors_consuming_iter(&xx_nix, Incoming) {
            println!("{:?}", nix)
        }
        let err_nix = NodeIndex(String::from("123"));
        // for nix in g1.neighbors_consuming_iter(&err_nix, Incoming) {
        //     println!("err never show :{:?}", nix)
        // }
    }
    #[test]
    #[allow(unused)]
    fn neighbors2() {
        let mut g = Graph::empty();
        let l1_nix = g.insert_node_in_topo("1", "1");
        let l1_1_nix = g.insert_node_in_topo("1.1", "1.1");
        let l1_2_nix = g.insert_node_in_topo("1.2", "1.2");
        let l1_3_nix = g.insert_node_in_topo("1.3", "1.3");

        let l1_1_1_nix = g.insert_node_in_topo("1.1.1", "1.1.1");
        let l1_1_2_nix = g.insert_node_in_topo("1.1.2", "1.1.2");
        let l1_1_3_nix = g.insert_node_in_topo("1.1.3", "1.1.3");

        let l1_2_1_nix = g.insert_node_in_topo("1.2.1", "1.2.1");
        let l1_2_2_nix = g.insert_node_in_topo("1.2.2", "1.2.2");
        let l1_2_3_nix = g.insert_node_in_topo("1.2.3", "1.2.3");

        let l1_3_1_nix = g.insert_node_in_topo("1.3.1", "1.3.1");
        let l1_3_2_nix = g.insert_node_in_topo("1.3.2", "1.3.2");
        let l1_3_3_nix = g.insert_node_in_topo("1.3.3", "1.3.3");
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
        let mut edge_it = g.deprecated_edges_consuming_iter(&l1_nix, Outgoing);

        println!("edge 1     {:?}", &edge_it.next());
        let e1node = &edge_it.node().unwrap();
        println!("edge 1 node() :{:?}", e1node);

        for nix in g.neighbors_consuming_iter(e1node, Outgoing) {
            println!("edge1node iter   :   {:?}", nix)
        }

        let mut n_iter: NodeNeighborsIter<&str, NodeEdgesConsumingIter<&str>> = edge_it.into();
        println!("edge iter -> n_iter      :{:?}", &n_iter.next());
        let mut e2edge_iter: NodeEdgesConsumingIter<&str> = n_iter.into();
        println!("edge back 3     :{:?}", &e2edge_iter.next());
        println!("edge back 4     :{:?}", &e2edge_iter.next());

        println!("======================");
    }
}

// impl<'a, E, Ix> Neighbors<'a, E, Ix>
// where
//     Ix: Clone + Hash + Eq,
// {
//     /// Return a “walker” object that can be used to step through the
//     /// neighbors and edges from the origin node.
//     ///
//     /// Note: The walker does not borrow from the graph, this is to allow mixing
//     /// edge walking with mutating the graph's weights.
//     pub fn detach(&self) -> WalkNeighbors<Ix> {
//         WalkNeighbors {
//             skip_start: self.skip_start,
//             next: self.next,
//         }
//     }
// }

// pub struct WalkNeighbors<Ix>
// where
//     Ix: Clone + Hash + Eq,
// {
//     skip_start: NodeIndex<Ix>,
//     next: [EdgeIndex<Ix>; 2],
// }

// impl<Ix> WalkNeighbors<Ix>
// where
//     Ix: Hash + Clone + Eq,
// {
//     /// Step to the next edge and its endpoint node in the walk for graph `g`.
//     ///
//     /// The next node indices are always the others than the starting point
//     /// where the `WalkNeighbors` value was created.
//     /// For an `Outgoing` walk, the target nodes,
//     /// for an `Incoming` walk, the source nodes of the edge.
//     pub fn next<N, E>(&mut self, g: &Graph<N, E, Ix>) -> Option<(EdgeIndex<Ix>, NodeIndex<Ix>)>
//     where
//         N: Clone,
//     {
//         // First any outgoing edges
//         match g.edges.get(self.next[0].index()) {
//             None => {}
//             Some(edge) => {
//                 let ed = self.next[0];
//                 self.next[0] = edge.next[0];
//                 return Some((ed, edge.node[1]));
//             }
//         }
//         // Then incoming edges
//         // For an "undirected" iterator (traverse both incoming
//         // and outgoing edge lists), make sure we don't double
//         // count selfloops by skipping them in the incoming list.
//         while let Some(edge) = g.edges.get(self.next[1].index()) {
//             let ed = self.next[1];
//             self.next[1] = edge.next[1];
//             if edge.node[0] != self.skip_start {
//                 return Some((ed, edge.node[0]));
//             }
//         }
//         None
//     }

//     pub fn next_node<N, E>(&mut self, g: &Graph<N, E, Ix>) -> Option<NodeIndex<Ix>>
//     where
//         N: Clone,
//     {
//         self.next(g).map(|t| t.1)
//     }

//     pub fn next_edge<N, E>(&mut self, g: &Graph<N, E, Ix>) -> Option<EdgeIndex<Ix>>
//     where
//         N: Clone,
//     {
//         self.next(g).map(|t| t.0)
//     }
// }
