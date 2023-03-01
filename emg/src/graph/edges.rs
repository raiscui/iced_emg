use crate::graph::{dir::HasDir, neighbors::NodeNeighborsIter, Direction, EdgeIndex, NodeIndex};
use crate::im::vector;
use std::clone::Clone;

// type ConsumingIter = vector::ConsumingIter<EdgeIndex>;
type ConsumingIter = indexmap::set::IntoIter<EdgeIndex>;

pub struct NodeEdgesIter<'a> {
    dir: Direction,
    edge_iter: vector::Iter<'a, EdgeIndex>,
    current_next: Option<&'a EdgeIndex>,
}

impl HasDir for NodeEdgesIter<'_> {
    fn dir(&self) -> Direction {
        self.dir
    }
}

impl<'a> NodeEdgesIter<'a> {
    pub fn new(dir: Direction, edge_iter: vector::Iter<'a, EdgeIndex>) -> Self {
        Self {
            dir,
            edge_iter,
            current_next: None,
        }
    }

    pub fn node(&self) -> Option<&NodeIndex> {
        self.current_next.and_then(|e| e.nix_by_dir(self.dir()))
    }
}

impl<'a> From<NodeNeighborsIter<NodeEdgesIter<'a>>> for NodeEdgesIter<'a> {
    fn from(nn: NodeNeighborsIter<NodeEdgesIter<'a>>) -> Self {
        nn.edges()
    }
}

impl<'a> Iterator for NodeEdgesIter<'a> {
    type Item = &'a EdgeIndex;

    fn next(&mut self) -> Option<Self::Item> {
        self.current_next = self.edge_iter.next();
        self.current_next
    }
}

// ────────────────────────────────────────────────────────────────────────────────
pub struct NodeEdgesConsumingIter {
    dir: Direction,
    edge_iter: ConsumingIter,
    current_next: Option<EdgeIndex>,
}

impl HasDir for NodeEdgesConsumingIter {
    fn dir(&self) -> Direction {
        self.dir
    }
}

impl NodeEdgesConsumingIter {
    pub fn new(dir: Direction, edge_iter: ConsumingIter) -> Self {
        Self {
            dir,
            edge_iter,
            current_next: None,
        }
    }

    pub fn node(&self) -> Option<NodeIndex> {
        self.current_next
            .as_ref()
            .and_then(|e| e.nix_by_dir(self.dir()).cloned())
    }
}

impl From<NodeNeighborsIter<NodeEdgesConsumingIter>> for NodeEdgesConsumingIter {
    fn from(nn: NodeNeighborsIter<NodeEdgesConsumingIter>) -> Self {
        nn.edges()
    }
}

impl Iterator for NodeEdgesConsumingIter {
    type Item = EdgeIndex;

    fn next(&mut self) -> Option<Self::Item> {
        self.current_next = self.edge_iter.next();
        self.current_next.clone()
    }
}
