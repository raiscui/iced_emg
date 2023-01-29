use crate::graph::{dir::HasDir, neighbors::NodeNeighborsIter, Direction, EdgeIndex, NodeIndex};
use crate::im::vector;
use std::{clone::Clone, cmp::Eq, hash::Hash};

// type ConsumingIter<Ix> = vector::ConsumingIter<EdgeIndex<Ix>>;
type ConsumingIter<Ix> = indexmap::set::IntoIter<EdgeIndex<Ix>>;

pub struct NodeEdgesIter<'a, Ix>
where
    Ix: 'a + Clone + Hash + Eq,
{
    dir: Direction,
    edge_iter: vector::Iter<'a, EdgeIndex<Ix>>,
    current_next: Option<&'a EdgeIndex<Ix>>,
}

impl<Ix> HasDir for NodeEdgesIter<'_, Ix>
where
    Ix: Clone + Hash + Eq,
{
    fn dir(&self) -> Direction {
        self.dir
    }
}

impl<'a, Ix> NodeEdgesIter<'a, Ix>
where
    Ix: 'a + Clone + Hash + Eq,
{
    pub fn new(dir: Direction, edge_iter: vector::Iter<'a, EdgeIndex<Ix>>) -> Self {
        Self {
            dir,
            edge_iter,
            current_next: None,
        }
    }

    pub fn node(&self) -> Option<NodeIndex<Ix>> {
        self.current_next
            .and_then(|e| e.nix_by_dir(self.dir()).clone())
    }
}

impl<'a, Ix> From<NodeNeighborsIter<Ix, NodeEdgesIter<'a, Ix>>> for NodeEdgesIter<'a, Ix>
where
    Ix: Clone + Hash + Eq,
{
    fn from(nn: NodeNeighborsIter<Ix, NodeEdgesIter<'a, Ix>>) -> Self {
        nn.edges()
    }
}

impl<'a, Ix> Iterator for NodeEdgesIter<'a, Ix>
where
    Ix: 'a + Clone + Hash + Eq,
{
    type Item = &'a EdgeIndex<Ix>;

    fn next(&mut self) -> Option<Self::Item> {
        self.current_next = self.edge_iter.next();
        self.current_next
    }
}

// ────────────────────────────────────────────────────────────────────────────────
pub struct NodeEdgesConsumingIter<Ix>
where
    Ix: Clone + Hash + Eq,
{
    dir: Direction,
    edge_iter: ConsumingIter<Ix>,
    current_next: Option<EdgeIndex<Ix>>,
}

impl<Ix> HasDir for NodeEdgesConsumingIter<Ix>
where
    Ix: Clone + Hash + Eq,
{
    fn dir(&self) -> Direction {
        self.dir
    }
}

impl<Ix> NodeEdgesConsumingIter<Ix>
where
    Ix: Clone + Hash + Eq,
{
    pub fn new(dir: Direction, edge_iter: ConsumingIter<Ix>) -> Self {
        Self {
            dir,
            edge_iter,
            current_next: None,
        }
    }

    pub fn node(&self) -> Option<NodeIndex<Ix>> {
        self.current_next
            .as_ref()
            .and_then(|e| e.nix_by_dir(self.dir()).clone())
    }
}

impl<Ix> From<NodeNeighborsIter<Ix, NodeEdgesConsumingIter<Ix>>> for NodeEdgesConsumingIter<Ix>
where
    Ix: Clone + Hash + Eq,
{
    fn from(nn: NodeNeighborsIter<Ix, NodeEdgesConsumingIter<Ix>>) -> Self {
        nn.edges()
    }
}

impl<Ix> Iterator for NodeEdgesConsumingIter<Ix>
where
    Ix: Clone + Hash + Eq,
{
    type Item = EdgeIndex<Ix>;

    fn next(&mut self) -> Option<Self::Item> {
        self.current_next = self.edge_iter.next();
        self.current_next.clone()
    }
}

// ────────────────────────────────────────────────────────────────────────────────

// pub trait IntoNodeEdges<'a, IterItem> {
//     type EdgesIter: Iterator<Item = IterItem>;
//     fn into_edges(self, n: IterItem, dir: Direction) -> Self::EdgesIter;
// }

// impl<'a, N, E, Ix> IntoNodeEdges<'a, NodeIndex<Ix>> for &'a Graph<N, E, Ix>
// where
//     E: Clone,
//     Ix: Eq + Clone + Hash + Debug,
//     N: Clone,
// {
//     type EdgesIter = NodeEdgesIter<'a, Ix>;
//     fn into_edges(self, n: &'a NodeIndex<Ix>, dir: Direction) -> Self::EdgesIter {

//     }
// }
