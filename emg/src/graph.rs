/*
 * @Author: Rais
 * @Date: 2020-12-28 16:48:19
 * @LastEditTime: 2023-03-29 16:40:44
 * @LastEditors: Rais
 * @Description:
 */

mod dir;
mod edges;

mod iter_format;
mod neighbors;

use crate::error::Error;
pub use edges::NodeEdgesIter;
use emg_common::im::{
    hashmap::{self, Entry},
    HashMap,
};
use emg_common::{display::DictDisplay, IdStr};
use emg_state::{
    state_store, topo, use_state, CloneState, CloneStateAnchor, Dict, GStateStore, StateAnchor,
    StateVar,
};
use indented::{indented, indented_with};
use iter_format::{DebugMap, IterFormatExt};
use neighbors::NodeNeighborsIter;
use owning_ref::RcRef;
use std::{borrow::Cow, fmt::Write};
// use smallvec::{smallvec, SmallVec};
use std::{
    cell::{Ref, RefCell},
    clone::Clone,
    cmp::{Eq, Ordering},
    fmt::{self, Debug, Display},
    hash::{BuildHasherDefault, Hash},
    mem::size_of,
    ops::{Deref, Index, IndexMut},
    rc::Rc,
};
use tracing::{debug, debug_span, trace, warn};

use delegate::delegate;
// use fxhash::FxBuildHasher;
// use rustc_hash::FxHasher as Custom_hasher;
use emg_hasher::CustomHasher;
use indexmap::IndexSet;

// ────────────────────────────────────────────────────────────────────────────────
// ────────────────────────────────────────────────────────────────────────────────

pub type NodeCollect<N> = HashMap<IdStr, Node<N>, BuildHasherDefault<CustomHasher>>;
// type OutGoingEdgeVec = SmallVec<[EdgeIndex; OUT_EDGES_SIZE]>;
pub type EdgePlugsCollect = IndexSet<EdgeIndex, BuildHasherDefault<CustomHasher>>;

// const OUT_EDGES_SIZE: usize = 2;
pub use Direction::{Incoming, Outgoing};

use crate::{clone_fields, copyclone};

// Index into the NodeIndex and EdgeIndex arrays
/// Edge direction.
#[derive(Copy, Debug, PartialEq, PartialOrd, Ord, Eq, Hash)]
#[repr(usize)]
pub enum Direction {
    /// An `Outgoing` edge is an outward edge *from* the current node.
    Outgoing = 0,
    /// An `Incoming` edge is an inbound edge *to* the current node.
    Incoming = 1,
}

copyclone!(Direction);

impl Direction {
    /// Return the opposite `Direction`.
    #[inline]
    pub fn opposite(self) -> Direction {
        match self {
            Outgoing => Incoming,
            Incoming => Outgoing,
        }
    }
    /// Return `0` for `Outgoing` and `1` for `Incoming`.
    #[inline]
    pub fn index(self) -> usize {
        (self as usize) & 0x1
    }
}

/// @ NodeIndex ──────────────────────────────────────────────────────────────────────────────────
#[derive(Clone, Default, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct NodeIndex(IdStr);

impl std::borrow::Borrow<IdStr> for NodeIndex {
    fn borrow(&self) -> &IdStr {
        &self.0
    }
}

impl Deref for NodeIndex {
    type Target = IdStr;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl NodeIndex {
    #[inline]
    pub fn new(x: IdStr) -> Self {
        NodeIndex(x)
    }
    // #[inline]
    // pub fn index(self) -> IdStr {
    //     self.index_ref().clone()
    // }
    #[inline]
    pub fn index(&self) -> &IdStr {
        &self.0
    }

    // NOTE change IdStr may out-of-control
    // pub fn set_index(&mut self, ix: IdStr) -> &mut Self {
    //     self.0 = ix;
    //     self
    // }
}
// impl Clone for NodeIndex
// where
//     IdStr: Clone,
// {
//     fn clone(&self) -> Self {
//         NodeIndex(self.0.clone())
//     }

//     fn clone_from(&mut self, source: &Self) {
//         self.0.clone_from(&source.0);
//     }
// }

impl From<IdStr> for NodeIndex {
    fn from(ix: IdStr) -> Self {
        NodeIndex(ix)
    }
}

impl Debug for NodeIndex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "NodeIndex({:?})", &self.0)
    }
}
impl std::fmt::Display for NodeIndex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "▣ {}", &self.0)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OptionNodeIndex(Option<NodeIndex>);

impl OptionNodeIndex {
    pub fn as_ref(&self) -> Option<&NodeIndex> {
        (**self).as_ref()
    }
}

impl From<IdStr> for OptionNodeIndex {
    fn from(value: IdStr) -> Self {
        Self(Some(NodeIndex::new(value)))
    }
}

impl From<NodeIndex> for OptionNodeIndex {
    fn from(value: NodeIndex) -> Self {
        Self(Some(value))
    }
}

impl From<Option<NodeIndex>> for OptionNodeIndex {
    fn from(value: Option<NodeIndex>) -> Self {
        Self(value)
    }
}
// impl<IdStr: Clone> From<Option<&NodeIndex>> for OptionNodeIndex {
//     fn from(value: Option<&NodeIndex>) -> Self {
//         Self(value.cloned())
//     }
// }

impl Deref for OptionNodeIndex {
    type Target = Option<NodeIndex>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// @ EdgeIndex ────────────────────────────────────────────────────────────────────────────────
#[derive(Clone, Default, PartialEq, Eq, Hash)]
pub struct EdgeIndex(OptionNodeIndex, OptionNodeIndex);

impl PartialOrd for EdgeIndex {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0
            .partial_cmp(&other.0)
            .and(self.1.partial_cmp(&other.1))
    }
}

impl Ord for EdgeIndex {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0).then(self.1.cmp(&other.1))
    }
}

impl Display for EdgeIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "├─ {} => {} ",
            self.0
                .as_ref()
                .map_or("▢".to_string(), |nix| format!("{nix}")),
            self.1
                .as_ref()
                .map_or("▢".to_string(), |nix| format!("{nix}")),
        )
    }
}

impl EdgeIndex {
    #[inline]
    pub fn new(s: impl Into<OptionNodeIndex>, t: impl Into<OptionNodeIndex>) -> Self {
        EdgeIndex(s.into(), t.into())
    }

    // #[inline]
    // pub fn ix_s(&self) -> &(IdStr, IdStr) {
    //     &self.0
    // }
    #[inline]
    pub fn source_nix(&self) -> Option<&NodeIndex> {
        self.0.as_ref()
    }

    /// Return the target node index.
    #[inline]
    pub fn target_nix(&self) -> Option<&NodeIndex> {
        self.1.as_ref()
    }

    pub fn set_incoming(&mut self, nix: OptionNodeIndex) {
        self.0 = nix;
    }
    pub fn set_outgoing(&mut self, nix: OptionNodeIndex) {
        self.1 = nix;
    }
    pub fn with_incoming(mut self, nix: OptionNodeIndex) -> Self {
        self.0 = nix;
        self
    }
    pub fn with_outgoing(mut self, nix: OptionNodeIndex) -> Self {
        self.1 = nix;
        self
    }
    // pub fn set_incoming(&mut self, nix: NodeIndex) -> &Self {
    //     self.0 .0 = nix.0;
    //     self
    // }
    // pub fn set_outgoing(&mut self, nix: NodeIndex) -> &Self {
    //     self.0 .1 = nix.0;
    //     self
    // }

    pub fn nix_by_dir(&self, dir: Direction) -> Option<&NodeIndex> {
        match dir {
            Outgoing => &self.1,
            Incoming => &self.0,
        }
        .as_ref()
    }

    pub fn get_nix_s(&self) -> (Option<&NodeIndex>, Option<&NodeIndex>) {
        let Self(s, t) = self;
        (s.as_ref(), t.as_ref())
    }
    pub fn get_nix_s_unwrap(&self) -> (&NodeIndex, &NodeIndex) {
        let Self(s, t) = self;
        (s.as_ref().unwrap(), t.as_ref().unwrap())
    }

    // fn _into_node(self) -> NodeIndex {
    //     NodeIndex(self.0)
    // }
}

impl<I, O> From<(I, O)> for EdgeIndex
where
    I: Into<OptionNodeIndex>,
    O: Into<OptionNodeIndex>,
{
    fn from((s, t): (I, O)) -> Self {
        EdgeIndex::new(s, t)
    }
}

impl Debug for EdgeIndex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "EdgeIndex({:?}->{:?})", &self.0, &self.1)
    }
}
// @ make index ────────────────────────────────────────────────────────────────────────────────

/// Short version of `NodeIndex::new`
#[inline]
pub fn node_index(index: impl Into<IdStr>) -> NodeIndex {
    NodeIndex(index.into())
}

/// Short version of `EdgeIndex::new`
#[inline]
pub fn edge_index(s: impl Into<IdStr>, t: impl Into<IdStr>) -> EdgeIndex {
    EdgeIndex::new(node_index(s), node_index(t))
}
#[inline]
pub fn edge_index_no_source(t: impl Into<IdStr>) -> EdgeIndex {
    EdgeIndex::new(None::<NodeIndex>, node_index(t))
}

// ────────────────────────────────────────────────────────────────────────────────

// const DIRECTIONS: [Direction; 2] = [Outgoing, Incoming];

/// @ Node ────────────────────────────────────────────────────────────────────────────────
#[derive(Eq)]
pub struct Node<N> {
    /// 内容
    pub item: N,
    /// Next edge in outgoing and incoming edge lists.
    //TODO check 要有序
    incoming_eix_set: StateVar<EdgePlugsCollect>,
    outgoing_eix_set: StateVar<EdgePlugsCollect>, //TODO use smvec
    incoming_len: StateAnchor<usize>,
    outgoing_len: StateAnchor<usize>,
}

impl<N: Display> Display for Node<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut numbers = String::new();
        writeln!(numbers, "item: {}", &self.item)?;
        // ─────────────────────────────────────────────────────────────────────────────

        let mut incomings = String::new();
        for eix in self.incoming_eix_set.get_rc().iter() {
            writeln!(incomings, "{eix}")?;
        }
        // ─────────────────────────────────────────────────────────────

        let mut outgoings = String::new();
        for eix in self.outgoing_eix_set.get_rc().iter() {
            writeln!(outgoings, "{eix}")?;
        }

        writeln!(
            numbers,
            "incoming:[\n{}]",
            indented_with(incomings, " ".repeat("incoming:[".len()).as_str())
        )?;
        writeln!(
            numbers,
            "outgoing:[\n{}]",
            indented_with(outgoings, " ".repeat("outgoing:[".len()).as_str())
        )?;
        write!(f, "Node {{\n{}\n}}\n", indented(numbers))
    }
}

impl<N: PartialEq> PartialEq for Node<N> {
    fn eq(&self, other: &Self) -> bool {
        let _span = debug_span!("PartialEq for Node").entered();
        #[cfg(debug_assertions)]
        {
            debug!("item is:{}", self.item == other.item);
            debug!(
                "incoming_eix_set is:{}",
                self.incoming_eix_set == other.incoming_eix_set
            );
            debug!(
                "outgoing_eix_set is:{}",
                self.outgoing_eix_set == other.outgoing_eix_set
            );
            debug!(
                "incoming_len is:{}",
                self.incoming_len == other.incoming_len
            );
            debug!(
                "outgoing_len is:{}",
                self.outgoing_len == other.outgoing_len
            );
        }
        self.item == other.item
            && self.incoming_eix_set == other.incoming_eix_set
            && self.outgoing_eix_set == other.outgoing_eix_set
            && self.incoming_len == other.incoming_len
            && self.outgoing_len == other.outgoing_len
    }
}

//TODO:  clean where dep , current working at here...
impl<N> Debug for Node<N>
where
    N: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let outs_len = self.outgoing_len.get();
        let ins_len = self.incoming_len.get();
        let mut fmt_struct = f.debug_struct("Node");
        fmt_struct.field("item", &self.item);
        fmt_struct.field("incoming_count", &ins_len);
        fmt_struct.field("outgoing_count", &outs_len);
        if ins_len != 0 {
            fmt_struct.field("incoming", &self.incoming());
        }
        if outs_len != 0 {
            fmt_struct.field("outgoing", &self.outgoing_eix_set.get());
        }
        fmt_struct.finish()
    }
}

impl<N> Node<N> {
    #[topo::nested]
    pub fn new_in_topo(item: N) -> Self {
        let incoming_eix_set: StateVar<EdgePlugsCollect> = use_state(EdgePlugsCollect::default);
        let incoming_len = incoming_eix_set.watch().map(|ins| ins.len());
        let outgoing_eix_set: StateVar<EdgePlugsCollect> = use_state(EdgePlugsCollect::default);
        let outgoing_len = outgoing_eix_set.watch().map(|outs| outs.len());
        Self {
            item,
            incoming_eix_set,
            outgoing_eix_set,
            incoming_len,
            outgoing_len,
        }
    }
    pub fn new(
        item: N,
        incoming_eix_set: StateVar<EdgePlugsCollect>,
        outgoing_eix_set: StateVar<EdgePlugsCollect>,
    ) -> Self {
        let incoming_len = incoming_eix_set.watch().map(|ins| ins.len());
        let outgoing_len = outgoing_eix_set.watch().map(|outs| outs.len());
        Self {
            item,
            incoming_eix_set,
            outgoing_eix_set,
            incoming_len,
            outgoing_len,
        }
    }

    pub fn edge_out_ixs(&self) -> Rc<EdgePlugsCollect> {
        self.outgoing_eix_set.get_rc()
    }
    pub fn edge_ixs(&self, dir: Direction) -> EdgePlugsCollect {
        match dir {
            // Incoming => self.incoming().clone(),
            Incoming => self.incoming().get(),
            Outgoing => self.outgoing().get(),
        }
    }
    pub fn edge_ixs_sa(&self, dir: Direction) -> &StateVar<EdgePlugsCollect> {
        match dir {
            // Incoming => self.incoming().clone(),
            Incoming => self.incoming(),
            Outgoing => self.outgoing(),
        }
    }

    /// Accessor for data structure internals: the first edge in the given direction.

    pub fn incoming(&self) -> &StateVar<EdgePlugsCollect> {
        &self.incoming_eix_set
    }
    pub fn outgoing(&self) -> &StateVar<EdgePlugsCollect> {
        &self.outgoing_eix_set
    }
    pub fn incoming_mut_with<F: FnOnce(&mut EdgePlugsCollect)>(&self, func: F) {
        self.incoming_eix_set.update(func)
    }
    pub fn outgoing_mut_with<F: FnOnce(&mut EdgePlugsCollect)>(&self, func: F) {
        self.outgoing_eix_set.update(func)
    }

    pub fn remove_plug(&self, dir: Direction, e_ix: &EdgeIndex) -> Result<EdgeIndex, Error> {
        let removed = match dir {
            Incoming => {
                if self.incoming_len.get() == 1 {
                    self.incoming_eix_set
                        .update(|ins| ins.take(e_ix))
                        .ok_or(Error::CanNotGetEdge)?
                } else {
                    self.incoming_eix_set
                        .update(|ins| ins.shift_take(e_ix))
                        .ok_or(Error::CanNotGetEdge)?
                }
            }
            Outgoing => {
                if self.outgoing_len.get() == 1 {
                    self.outgoing_eix_set
                        .update(|x| x.take(e_ix))
                        .ok_or(Error::CanNotGetEdge)?
                } else {
                    self.outgoing_eix_set
                        .update(|x| x.shift_take(e_ix))
                        .ok_or(Error::CanNotGetEdge)?
                }
            }
        };
        assert_eq!(&removed, e_ix);

        Ok(removed)
    }

    pub fn incoming_len(&self) -> usize {
        self.incoming_len.get()
    }
}

impl<N> Clone for Node<N>
where
    N: Clone,
{
    clone_fields!(
        Node,
        item,
        incoming_eix_set,
        outgoing_eix_set,
        incoming_len,
        outgoing_len
    );
}

/// @ Edge ────────────────────────────────────────────────────────────────────────────────
/// The graph's edge type.
// aef struct aef
type EdgeNodeIxSv = StateVar<Option<NodeIndex>>;
#[derive(Debug, PartialEq, Eq)]
pub struct Edge<E> {
    /// Associated edge data.
    pub item: E,

    source_nix: EdgeNodeIxSv,
    target_nix: EdgeNodeIxSv,
}

impl<E: Display> Display for Edge<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut members = String::new();
        writeln!(members, "item: {} ,", self.item)?;

        if let Some(source_nix) = self.source_nix.get() {
            writeln!(members, "source_nix: {} ,", source_nix)?;
        } else {
            writeln!(members, "source_nix: None ,")?;
        }

        if let Some(target_nix) = self.target_nix.get() {
            writeln!(members, "target_nix: {} ,", target_nix)?;
        } else {
            writeln!(members, "target_nix: None ,")?;
        }

        write!(
            f,
            "Edge {{\n{}\n}}",
            indented_with(members, " ".repeat("Edge {".len()).as_str())
        )
    }
}

impl<E> std::ops::Deref for Edge<E> {
    type Target = E;

    fn deref(&self) -> &Self::Target {
        &self.item
    }
}
#[cfg(test)]
#[topo::nested]
pub fn edge_in_topo<E: Clone>(s: NodeIndex, t: NodeIndex, item: E) -> Edge<E> {
    Edge::new_in_topo(Some(s), Some(t), item)
}

impl<E> Edge<E>
where
    E: Clone,
{
    #[cfg(test)]
    #[topo::nested]
    pub fn new_in_topo(
        source_nix: Option<NodeIndex>,
        target_nix: Option<NodeIndex>,
        item: E,
    ) -> Self {
        Self {
            item,
            source_nix: use_state(|| source_nix),
            target_nix: use_state(|| target_nix),
        }
    }
    pub fn new(opt_source_nix_sv: EdgeNodeIxSv, opt_target_nix_sv: EdgeNodeIxSv, item: E) -> Self {
        Self {
            item,
            source_nix: opt_source_nix_sv,
            target_nix: opt_target_nix_sv,
        }
    }

    // /// Accessor for data structure internals: the next edge for the given direction.
    // pub fn next_edge(&self, dir: Direction) -> EdgeIndex {
    //     self.next[dir.index()]
    // }

    /// Return the source node index.

    pub fn node_ix(&self, dir: Direction) -> &EdgeNodeIxSv {
        match dir {
            Outgoing => self.target_nix(),
            Incoming => self.source_nix(),
        }
    }
    pub fn source_nix(&self) -> &EdgeNodeIxSv {
        &self.source_nix
    }

    /// Return the target node index.
    pub fn target_nix(&self) -> &EdgeNodeIxSv {
        &self.target_nix
    }

    pub fn endpoints(&self) -> (EdgeNodeIxSv, EdgeNodeIxSv) {
        (*self.source_nix(), *self.target_nix())
    }

    /// Get a reference to the edge's item.
    pub fn item(&self) -> &E {
        &self.item
    }
}

impl<E> Clone for Edge<E>
where
    E: Clone,
{
    clone_fields!(Edge, item, source_nix, target_nix);
}

// @ Graph ────────────────────────────────────────────────────────────────────────────────

pub struct Graph<N, E>
where
    N: Clone,
    E: Clone,
{
    store: Rc<RefCell<GStateStore>>,
    nodes: NodeCollect<N>,
    pub edges: StateVar<Dict<EdgeIndex, Edge<E>>>,
}

impl<N, E> Display for Graph<N, E>
where
    N: Clone,
    E: Clone + 'static,
    N: Display,
    E: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut nodes = String::new();
        self.nodes.iter().for_each(|(ix, node)| {
            writeln!(nodes, "{}:\n{}\n,", ix, indented(node)).unwrap();
        });

        let mut members = format!("nodes:{{\n{}\n}}\n", indented_with(nodes, "       "));
        writeln!(
            members,
            "edges:{{\n{}}}\n",
            indented_with(DictDisplay("", self.edges.get()), "       ")
        )?;

        write!(f, "Graph {{\n{}\n}}\n", indented(members))
    }
}

impl<N, E> Clone for Graph<N, E>
where
    N: Clone,
    E: Clone,
{
    fn clone(&self) -> Self {
        Graph {
            store: self.store.clone(),
            nodes: self.nodes.clone(),
            edges: self.edges,
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.store.clone_from(&source.store);
        self.nodes.clone_from(&source.nodes);
        self.edges = source.edges;
    }
}

impl<N, E> fmt::Debug for Graph<N, E>
where
    N: fmt::Debug + Clone,
    E: fmt::Debug + Clone + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut fmt_struct = f.debug_struct("Graph");
        fmt_struct.field("node_count", &self.node_count());
        fmt_struct.field("edges_count", &self.edges_count());
        if size_of::<N>() != 0 {
            // {k:v} like
            fmt_struct.field(
                "node item",
                &DebugMap(|| {
                    self.nodes
                        .iter()
                        .map(|n| (n.0, n.1))
                        .collect::<Vec<(&IdStr, &Node<N>)>>()
                }),
            );
        }
        if self.edges_count() > 0 {
            let es = self.edges.store_get(&self.store());
            fmt_struct.field("edges", &es.iter().map(|(eix, _e)| eix).format(",\n"));

            if size_of::<E>() != 0 {
                fmt_struct.field(
                    "edge item",
                    &DebugMap(|| es.iter().map(|(eix, e)| (eix, e)).collect::<Vec<_>>()),
                );
            }
        }

        fmt_struct.finish()
    }
}

impl<N, E> Default for Graph<N, E>
where
    E: Clone + 'static + std::fmt::Debug,
    N: Clone,
{
    #[topo::nested]
    fn default() -> Self {
        Self {
            store: state_store(),
            nodes: HashMap::default(),
            edges: use_state(Dict::new),
        }
    }
}

type EdgeRef<R, E> = RcRef<Dict<EdgeIndex, Edge<E>>, R>;

impl<N, E> Graph<N, E>
where
    E: Clone + 'static + std::fmt::Debug,
    N: Clone,
    EdgeIndex: Clone,
{
    pub fn eq_sloppy(&self, other: &Self) -> bool
    where
        N: Clone + PartialEq,
        E: Clone + PartialEq,
    {
        self.nodes == other.nodes && self.edges == other.edges
    }

    #[topo::nested]
    pub fn new_with_in_topo(nodes: NodeCollect<N>, edges: Dict<EdgeIndex, Edge<E>>) -> Self {
        Self {
            store: state_store(),
            nodes,
            edges: use_state(|| edges),
        }
    }

    #[topo::nested]
    pub fn empty() -> Self {
        Graph::default()
    }

    pub fn deep_eq_use_format_str(&self, other: &Self) -> bool
    where
        N: Eq + fmt::Debug,
        E: Eq + fmt::Debug,
    {
        // self.nodes == other.nodes
        // && format!("{:?}", self.edges.store_get(&self.store())) == format!("{:?}", other.edges.store_get(&other.store()))
        format!("{self:?}") == format!("{other:?}")
    }
    // /// Create a new `Graph` with estimated capacity.
    // pub fn with_capacity(nodes: usize, edges: usize) -> Self {
    //     Graph {
    //         nodes: Vec::with_capacity(nodes),
    //         edges: Vec::with_capacity(edges),
    //     }
    // }

    /// Return the number of nodes (vertices) in the graph.
    ///
    /// Computes in **O(1)** time.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Return the number of edges in the graph.
    ///
    /// Computes in **O(1)** time.
    pub fn edges_count(&self) -> usize {
        self.edges.store_get_rc(&self.store()).len()
    }

    /// Access the item for node `a`.
    ///
    /// Also available with indexing syntax: `&graph[a]`.

    pub fn get_node_item(&self, a: &NodeIndex) -> Option<&N> {
        self.nodes.get(a.index()).map(|n| &n.item)
    }
    pub fn get_node_use_ix(&self, ix: &IdStr) -> Option<&Node<N>> {
        self.nodes.get(ix)
    }
    pub fn get_node_item_use_ix(&self, ix: &IdStr) -> Option<&N> {
        self.nodes.get(ix).map(|n| &n.item)
    }
    pub fn get_mut_node_item_use_ix(&mut self, ix: &IdStr) -> Option<&mut N> {
        self.nodes.get_mut(ix).map(|n| &mut n.item)
    }
    /// Access the item for node `a`, mutably.
    ///
    /// Also available with indexing syntax: `&mut graph[a]`.
    pub fn get_mut_node_item(&mut self, a: &NodeIndex) -> Option<&mut N> {
        self.nodes.get_mut(a.index()).map(|n| &mut n.item)
    }

    #[cfg(test)]
    #[topo::nested]
    pub fn insert_node_in_topo_only(&mut self, key: impl Into<IdStr>, item: N) -> NodeIndex {
        let id = key.into();
        let node = Node::new_in_topo(item);
        let node_idx = node_index(id.clone());
        self.nodes.insert(id, node);

        node_idx
    }
    //TODO use topo key? 同id 不同key ,实现shadow-nodeItem(not shadow-tree(node))
    pub fn or_insert_node_with_plugs(
        &mut self,
        key: IdStr,
        item: impl FnOnce(IdStr) -> N,
        incoming_eix_set: StateVar<EdgePlugsCollect>,
        outgoing_eix_set: StateVar<EdgePlugsCollect>,
    ) {
        self.nodes
            .entry(key.clone())
            .or_insert_with(|| Node::new(item(key), incoming_eix_set, outgoing_eix_set));
    }

    pub fn nodes_contains_key(&self, key: &IdStr) -> bool {
        self.nodes.contains_key(key)
    }
    // pub fn insert_root(&mut self, key: IdStr, item: N, edge_item: E) -> NodeIndex {
    //     let node = Node::from(item);
    //     let node_idx = node_index(key.clone());
    //     self.nodes.insert(key, node);
    //     self.root_edge_ix = Edge::new(node_idx.clone(), node_idx.clone(), edge_item).into();
    //     node_idx
    // }

    /// Add an edge from `a` to `b` to the graph, with its associated
    /// data `item`.
    ///
    /// Return the index of the new edge.
    ///
    /// Computes in **O(1)** time.
    ///
    /// **Panics** if any of the nodes don't exist.<br>
    /// **Panics** if the Graph is at the maximum number of edges for its index
    /// type (N/A if usize).
    ///
    /// **Note:** `Graph` allows adding parallel (“duplicate”) edges. If you want
    /// to avoid this, use [`.update_edge(a, b, item)`](#method.update_edge) instead.
    #[cfg(test)]
    #[topo::nested]
    pub fn unused_insert_update_edge_in_topo(
        &mut self,
        s_nix: &NodeIndex,
        t_nix: &NodeIndex,
        item: E,
    ) -> Option<EdgeIndex> {
        // if self.nodes.contains_key(&a.index()) {
        //     return None;
        // }
        // if self.nodes.contains_key(&b.index()) {
        //     return None;
        // }

        let edge_idx = EdgeIndex::new(s_nix.clone(), t_nix.clone());

        self.nodes_connect(&edge_idx).unwrap();

        let edge = || Edge::new_in_topo(Some(s_nix.clone()), Some(t_nix.clone()), item);

        self.or_insert_edge_only(edge_idx.clone(), edge);
        Some(edge_idx)
    }

    ///will update [`Node`]`outgoing_eix_set` ([`StateVar`])
    pub fn nodes_connect(&self, e_ix: &EdgeIndex) -> Result<(), Error> {
        if let Some(s_nix) = e_ix.source_nix() {
            self.nodes
                .get(s_nix.index())
                .ok_or_else(|| Error::CanNotGetNode {
                    nix: format!("{s_nix:?}"),
                })?
                .outgoing_mut_with(|outs| {
                    outs.insert(e_ix.clone());
                });
        }

        if let Some(t_nix) = e_ix.target_nix() {
            self.nodes
                .get(t_nix.index())
                .ok_or_else(|| Error::CanNotGetNode {
                    nix: format!("{t_nix:?}"),
                })?
                .incoming_mut_with(|in_s| {
                    in_s.insert(e_ix.clone());
                });
        }

        Ok(())
    }

    pub fn or_insert_edge_only(&self, edge_idx: EdgeIndex, edge: impl FnOnce() -> Edge<E>) -> E {
        let es = self.edges.get_rc();

        if es.contains_key(&edge_idx) {
            es.get(&edge_idx).unwrap().item.clone()
        } else {
            let new_e = edge();
            let ei = new_e.item.clone();

            self.edges.set(es.update(edge_idx, new_e));
            ei
        }
        // self.edges.store_update(&self.store(), |es| {
        //     trace!(
        //         "has edge?-- {:?} --{}",
        //         &edge_idx,
        //         es.contains_key(&edge_idx)
        //     );
        //     // if es.contains_key(&edge_idx) {
        //     //     let e = es.get(&edge_idx).unwrap();
        //     //     assert_eq!(e, &edge);
        //     // }
        //     es.entry(edge_idx)
        //         .and_modify(|_| drop_fn())
        //         .or_insert_with(edge)
        //         .item
        //         .clone()
        // })
    }

    #[must_use]
    pub fn edge(&self, e: &EdgeIndex) -> EdgeRef<Edge<E>, E> {
        RcRef::new(self.edges.store_get_rc(&self.store())).map(|f| &f[e])
    }

    #[must_use]
    pub fn edge_item(&self, e: &EdgeIndex) -> EdgeRef<E, E> {
        RcRef::new(self.edges.store_get_rc(&self.store())).map(|f| &f[e].item)
    }

    #[must_use]
    pub fn edge_source(&self, e: &EdgeIndex) -> EdgeRef<EdgeNodeIxSv, E> {
        RcRef::new(self.edges.store_get_rc(&self.store())).map(|f| f[e].source_nix())
    }

    pub fn edge_plug_edit(
        &self,
        eix: &EdgeIndex,
        dir: Direction,
        change_to: impl Into<IdStr>,
    ) -> Result<(), Error> {
        match dir {
            Outgoing => {
                //TODO finish this like incoming
                // edge.target_nix.set(Some(node_index(change_to)))
                todo!()
            }
            Incoming => {
                let new_incoming_nix = Some(node_index(change_to));

                let old_eix = self.nodes_disconnect_only(eix)?;
                let new_eix = old_eix
                    .into_owned()
                    .with_incoming(new_incoming_nix.clone().into());

                // add in nodes ─────────────────────────────────────────────
                self.nodes_connect(&new_eix)?;

                //edges ─────────────────────────────────────────────────────────────────────────────

                self.edges
                    .store_update_result_check(&self.store(), |edges| {
                        let edge = edges.remove(eix).ok_or(Error::CanNotGetEdge)?;
                        edge.source_nix.set(new_incoming_nix);
                        edges.insert(new_eix, edge);
                        Ok(())
                    })
            }
        }
    }

    fn nodes_disconnect_only<'a>(&self, e_ix: &'a EdgeIndex) -> Result<Cow<'a, EdgeIndex>, Error> {
        let (source_n, target_n) = e_ix.get_nix_s();

        match (
            self.disconnect_plug_in_node_with_dir(source_n, Outgoing, e_ix),
            self.disconnect_plug_in_node_with_dir(target_n, Incoming, e_ix),
        ) {
            (Ok(a), Ok(b)) => match (a, b) {
                (aa @ Cow::Owned(_), _) => Ok(aa),

                (_, bb) => Ok(bb),
            },
            (a, b) => a.and(b),
        }
    }

    fn disconnect_plug_in_node_with_dir<'a>(
        &self,
        opt_n_ix: Option<&NodeIndex>,
        dir: Direction,
        e_ix: &'a EdgeIndex,
    ) -> Result<Cow<'a, EdgeIndex>, Error> {
        if let Some(n_ix) = opt_n_ix {
            self.nodes
                .get(n_ix.index())
                .ok_or_else(|| Error::CanNotGetNode {
                    nix: format!("{n_ix:?}"),
                })
                .and_then(|n| n.remove_plug(dir, e_ix))
                .map(Cow::Owned)
        } else {
            Ok(Cow::Borrowed(e_ix))
        }
    }

    /// Remove an edge and return its edge item, or `None` if it didn't exist.
    ///
    /// Apart from `e`, this invalidates the last edge index in the graph
    /// (that edge will adopt the removed edge index).
    ///
    /// Computes in **O(e')** time, where **e'** is the size of four particular edge lists, for
    /// the vertices of `e` and the vertices of another affected edge.
    #[allow(clippy::type_complexity)]
    pub fn remove_edge_and_disconnect<'a>(
        &mut self,
        e_ix: &'a EdgeIndex,
    ) -> Result<(Edge<E>, Cow<'a, EdgeIndex>), Error> {
        // remove edge
        Ok((
            self.remove_edge_only(e_ix)?,
            self.nodes_disconnect_only(e_ix)?,
        ))
    }

    /// Remove `a` from the graph if it exists, and return its item.
    /// If it doesn't exist in the graph, return `None`.
    ///
    /// Apart from `a`, this invalidates the last node index in the graph
    /// (that node will adopt the removed node index). Edge indices are
    /// invalidated as they would be following the removal of each edge
    /// with an endpoint in `a`.
    ///
    /// Computes in **O(e')** time, where **e'** is the number of affected
    /// edges, including *n* calls to `.remove_edge()` where *n* is the number
    /// of edges with an endpoint in `a`, and including the edges with an
    /// endpoint in the displaced node.
    // TODO: iter version for array remove
    pub fn remove_node_and_edge_and_disconnect(&mut self, n: NodeIndex) -> Option<N> {
        let Node {
            incoming_eix_set: incoming,
            outgoing_eix_set: outgoing,
            item,
            ..
        } = self.nodes.remove(n.index())?;

        // 断开 node 进出 连接,删除 edge
        for n_in_e_ix in incoming.get_rc().iter() {
            self.disconnect_plug_in_node_with_dir(n_in_e_ix.source_nix(), Outgoing, n_in_e_ix)
                .unwrap();
            self.remove_edge_only(n_in_e_ix).unwrap();
        }
        for n_out_e_ix in outgoing.get_rc().iter() {
            self.disconnect_plug_in_node_with_dir(n_out_e_ix.target_nix(), Incoming, n_out_e_ix)
                .unwrap();
            self.remove_edge_only(n_out_e_ix).unwrap();
        }

        Some(item)
    }

    fn remove_edge_only(&self, eix: &EdgeIndex) -> Result<Edge<E>, Error> {
        self.edges.store_update_result_check(&self.store(), |es| {
            es.remove(eix).ok_or(Error::CanNotGetEdge)
        })
    }

    /// ## iter NodeIndex 的 边 , 从 edgeIndex 中 取出边衔接的另一头 NodeIndex,
    // 迭代 与 A node dir 方向相连的 NodeIndex
    /// * return: NodeIndex,  not edge  /  以及另头的Node
    pub fn neighbors_consuming_iter(
        &self,
        nix: &NodeIndex,
        dir: Direction,
    ) -> NodeNeighborsIter<NodeEdgesConsumingIter> {
        NodeNeighborsIter::new(self.edges_consuming_iter(nix, dir))
    }
    // pub fn neighbors_iter(&self, nix: &NodeIndex, dir: Direction) -> NodeNeighborsIter {
    //     let node = self
    //         .nodes
    //         .get(nix.index())
    //         .expect(format!(":: not find node for id:{:?}!", &nix).as_str());
    //     NodeNeighborsIter {
    //         dir,
    //         node_plug_iter: node.edge_ix_s(dir).iter(),
    //     }
    // }

    /// ## 迭代 NodeIndex 的 edge
    /// * return: edgeIndex
    pub fn edges_consuming_iter(&self, nix: &IdStr, dir: Direction) -> NodeEdgesConsumingIter {
        let node = self
            .nodes
            .get(nix)
            .unwrap_or_else(|| panic!(":: not find node for id:{:?}!", nix));

        NodeEdgesConsumingIter::new(dir, node.edge_ixs(dir).into_iter())
    }
    // pub fn edges_iter_use_ix(&self, ix: &IdStr, dir: Direction) -> NodeEdgesIter {
    //     let node = self
    //         .nodes
    //         .get(ix)
    //         .unwrap_or_else(|| panic!(":: not find node for id:{:?}!", ix));

    //     NodeEdgesIter::new(dir, node.edge_ix_s(dir).iter())
    // }

    // * 迭代 A B 之间所有的 edge
    // /// Return an iterator over all the edges connecting `a` and `b`.
    // ///
    // /// - `Directed`: Outgoing edges from `a`.
    // /// - `Undirected`: All edges connected to `a`.
    // ///
    // /// Iterator element type is `EdgeReference<E, >
    // pub fn edges_connecting(
    //     &self,
    //     a: NodeIndex,
    //     b: NodeIndex,
    // ) -> EdgesConnecting<E, Ty, > {
    //     EdgesConnecting {
    //         target_node: b,
    //         edges: self.edges_directed(a, Direction::Outgoing),
    //         ty: PhantomData,
    //     }
    // }

    // * 确认 A B 是否有连接
    // /// Lookup if there is an edge from `a` to `b`.
    // ///
    // /// Computes in **O(e')** time, where **e'** is the number of edges
    // /// connected to `a` (and `b`, if the graph edges are undirected).
    // pub fn contains_edge(&self, a: NodeIndex, b: NodeIndex) -> bool {
    //     self.find_edge(a, b).is_some()
    // }

    // * 找到 A B 直之间一个边
    // /// Lookup an edge from `a` to `b`.
    // ///
    // /// Computes in **O(e')** time, where **e'** is the number of edges
    // /// connected to `a` (and `b`, if the graph edges are undirected).
    // pub fn find_edge(&self, a: NodeIndex, b: NodeIndex) -> Option<EdgeIndex> {
    //     if !self.is_directed() {
    //         self.find_edge_undirected(a, b).map(|(ix, _)| ix)
    //     } else {
    //         match self.nodes.get(a.index()) {
    //             None => None,
    //             Some(node) => self.find_edge_directed_from_node(node, b),
    //         }
    //     }
    // }

    // * 找到 A B 直之间一个边
    // fn find_edge_directed_from_node(
    //     &self,
    //     node: &Node<N, >,
    //     b: NodeIndex,
    // ) -> Option<EdgeIndex> {
    //     let mut edix = node.next[0];
    //     while let Some(edge) = self.edges.get(edix.index()) {
    //         if edge.node[1] == b {
    //             return Some(edix);
    //         }
    //         edix = edge.next[0];
    //     }
    //     None
    // }

    // * 起点 或 终点
    // /// Return an iterator over either the nodes without edges to them
    // /// (`Incoming`) or from them (`Outgoing`).
    // ///
    // /// An *internal* node has both incoming and outgoing edges.
    // /// The nodes in `.externals(Incoming)` are the source nodes and
    // /// `.externals(Outgoing)` are the sinks of the graph.
    // ///
    // /// For a graph with undirected edges, both the sinks and the sources are
    // /// just the nodes without edges.
    // ///
    // /// The whole iteration computes in **O(|V|)** time.
    // pub fn externals(&self, dir: Direction) -> Externals<N, Ty, > {
    //     Externals {
    //         iter: self.nodes.iter().enumerate(),
    //         dir,
    //         ty: PhantomData,
    //     }
    // }

    /// Access the internal node array.
    pub fn raw_nodes(&self) -> &NodeCollect<N> {
        &self.nodes
    }

    /// Access the internal edge array.
    pub fn raw_edges(&self) -> &StateVar<Dict<EdgeIndex, Edge<E>>> {
        &self.edges
    }

    pub fn get_raw_edges_watch(&self) -> StateAnchor<Dict<EdgeIndex, Edge<E>>> {
        self.edges.store_watch(&self.store())
    }

    // pub fn get_children_s(&self) -> Vec<N> {
    //     self.edges_iter_use_ix(cix, Outgoing)
    //         .map(|eix| {
    //             let child_ix = eix.ix_dir(Outgoing);
    //             // let a_child = self.get_node_weight_use_ix(child_ix).unwrap();
    //             self.gelement_to_el(child_ix)
    //         })
    //         .collect()
    // }

    // TODO: use retain
    delegate! {
        to self.nodes {

            #[call(retain)]
            pub fn retain_nodes<F: FnMut(&IdStr, &Node<N>) -> bool>(&mut self, f:F);

        }

    }

    #[cfg(test)]
    #[topo::nested]
    pub fn from_nodes_and_edges_in_topo<I>(nodes: &[(I, N)], edges: &[((I, I), E)]) -> Graph<N, E>
    where
        I: Into<IdStr> + Clone,
    {
        let handled_nodes = nodes
            .iter()
            .cloned()
            .map(|(k, w)| (k.into(), Node::new_in_topo(w)));
        let mut g_nodes: NodeCollect<N> = handled_nodes.collect();
        // let handled_edges = edges.iter().cloned().map(|(k, w)| (k, w.into()));
        let mut g_edges: Dict<EdgeIndex, Edge<E>> = Dict::new();
        for ((s, t), ew) in edges {
            let s = s.clone().into();
            let t = t.clone().into();
            let edge_idx = edge_index(s.clone(), t.clone());

            let node_with_edge_build_res = g_nodes
                .entry(s.clone())
                .and_modify(|n| {
                    n.outgoing_mut_with(|outs| {
                        outs.insert(edge_idx.clone());
                    });
                })
                .turn_to_result()
                // • • • • •
                .and(
                    // • • • • •
                    g_nodes
                        .entry(t.clone())
                        .and_modify(|n| {
                            // n.incoming_mut().push_back(edge_idx.clone());
                            n.incoming_mut_with(|ins| {
                                ins.insert(edge_idx.clone());
                            })
                        })
                        .turn_to_result(),
                    // • • • • •
                );

            match node_with_edge_build_res {
                Ok(_) => {}
                Err(e) => {
                    panic!("{}", e);
                }
            }

            let edge = edge_in_topo(node_index(s.clone()), node_index(t.clone()), ew.to_owned());

            g_edges.insert((s.to_owned(), t.to_owned()).into(), edge);
        }

        Graph::new_with_in_topo(g_nodes, g_edges)
    }

    /// Get a reference to the graph's store.
    pub fn store(&self) -> Ref<GStateStore> {
        self.store.as_ref().borrow()
    }
}

// ────────────────────────────────────────────────────────────────────────────────
// ────────────────────────────────────────────────────────────────────────────────

use std::hash::BuildHasher;

use self::edges::NodeEdgesConsumingIter;

trait ImEntryDebugOutput {
    fn turn_to_result(self) -> Result<(), String>;
}

impl<'a, K, V, S> ImEntryDebugOutput for hashmap::Entry<'a, K, V, S>
where
    K: 'a + Hash + Eq + Clone + Debug,
    V: 'a + Clone,
    S: 'a + BuildHasher,
{
    fn turn_to_result(self) -> Result<(), String> {
        match self {
            hashmap::Entry::Occupied(_) => Ok(()),
            hashmap::Entry::Vacant(entry) => {
                Err(format!("not have this index in hashmap:{:?}", &entry.key()))
            }
        }
    }
}
// trait ImEntry {
//     fn turn_to_option(self) -> Result<(), String>;
// }
// impl<'a, K, V, S> ImEntry for hashmap::Entry<'a, K, V, S>
// where
//     K: 'a + Hash + Eq + Clone,
//     V: 'a + Clone,
//     S: 'a + BuildHasher,
// {
//     fn turn_to_option(self) -> Result<(), String> {
//         match self {
//             hashmap::Entry::Occupied(_) => Ok(()),
//             hashmap::Entry::Vacant(_) => Err(format!("not have this index in hashmap!")),
//         }
//     }
// }

// @ graph index ────────────────────────────────────────────────────────────────────────────────
/// Index the `Graph` by `NodeIndex` to access node weights.
///
/// **Panics** if the node doesn't exist.
impl<'a, N, E> Index<&'a NodeIndex> for Graph<N, E>
where
    N: Clone,
    E: Clone,
{
    type Output = N;
    fn index(&self, index: &NodeIndex) -> &N {
        &self.nodes[index.index()].item
    }
}
impl<'a, N, E> IndexMut<&'a NodeIndex> for Graph<N, E>
where
    N: Clone,
    E: Clone,
{
    fn index_mut(&mut self, index: &NodeIndex) -> &mut N {
        &mut self.nodes[index.index()].item
    }
}

// @ test ────────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(unused)]
mod graph_test_mod {
    use crate::graph::Graph;
    use crate::im::Vector;
    use emg_common::IdStr;
    use emg_state::use_state;
    use indexmap::IndexSet;
    use std::{iter::FromIterator, path::Path};
    use tracing::debug;

    use std::clone::Clone;

    use crate::graph::{edge_index, node_index, Node};
    // ────────────────────────────────────────────────────────────────────────────────

    use std::cell::RefCell;
    use std::mem;
    use std::rc::Rc;

    use color_eyre::eyre::Report;
    use tracing_subscriber::EnvFilter;

    fn tracing_init(level: tracing::Level) -> Result<(), Report> {
        // use tracing_error::ErrorLayer;
        use tracing_subscriber::prelude::*;

        let out_layer = tracing_tree::HierarchicalLayer::new(2)
            .with_indent_lines(true)
            .with_indent_amount(4)
            .with_targets(true)
            .with_filter(tracing_subscriber::filter::dynamic_filter_fn(
                move |metadata, _cx| {
                    // if metadata.level() <= &tracing::Level::DEBUG{
                    //     // If this *is* "interesting_span", make sure to enable it.
                    //     if metadata.is_span() && metadata.name() == "LayoutOverride" {
                    //         return true;
                    //     }

                    //     // Otherwise, are we in an interesting span?
                    //     if let Some(current_span) = cx.lookup_current()  {
                    //         return current_span.name() == "LayoutOverride";
                    //     }
                    // }
                    // ─────────────────────────────────────────────────────

                    // #[cfg(feature = "debug")]
                    // return false;

                    // !metadata.target().contains("anchors")
                    // &&
                    !metadata.target().contains("emg_layout")
                        && !metadata.target().contains("emg_state")
                        && !metadata.target().contains("cassowary")
                        && !metadata.target().contains("wgpu")
                        && metadata.level() <= &level
                    // && !metadata.target().contains("winit_event")
                    // && !metadata.fields().field("event").map(|x|x.to_string())
                    // && !metadata.target().contains("winit event: DeviceEvent")
                },
            ));

        // #[cfg(feature = "debug")]
        // let layout_override_layer = tracing_tree::HierarchicalLayer::new(2)
        //     .with_indent_lines(true)
        //     .with_indent_amount(4)
        //     .with_targets(true)
        //     .with_filter(EnvFilter::new("[LayoutOverride]=debug"));

        // #[cfg(feature = "debug")]
        // let event_matching_layer = tracing_tree::HierarchicalLayer::new(2)
        //     .with_indent_lines(true)
        //     .with_indent_amount(4)
        //     .with_targets(true)
        //     .with_filter(EnvFilter::new("[event_matching]=debug"));

        // #[cfg(feature = "debug")]
        // let touch_layer = tracing_tree::HierarchicalLayer::new(2)
        //     .with_indent_lines(true)
        //     .with_indent_amount(4)
        //     .with_targets(true)
        //     .with_filter(EnvFilter::new("[Touch]=debug"));

        //NOTE emg_layout
        // let emg_layout_layer = tracing_tree::HierarchicalLayer::new(2)
        //     .with_indent_lines(true)
        //     .with_indent_amount(4)
        //     .with_targets(true)
        //     .with_filter(EnvFilter::new(
        //         // "emg_layout=debug,emg_layout[build inherited cassowary_generals_map],emg_layout[LayoutOverride]=error",
        //         "emg_layout[build inherited cassowary_generals_map]",
        //     ));
        // ─────────────────────────────────────────────────────────────────────────────

        // tracing_subscriber::registry()
        //     // .with(layout_override_layer)
        //     // .with(event_matching_layer)
        //     // .with(touch_layer)
        //     .with(emg_layout_layer)
        //     // .with(out_layer)
        //     .init();

        tracing_subscriber::registry().with(out_layer).init();
        // ─────────────────────────────────────────────────────────────────────────────

        color_eyre::install()
    }

    #[test]
    fn test3() {
        enum Foo {
            A(&'static str),
            B(i32),
            C(i32),
        }

        assert_eq!(
            mem::discriminant(&Foo::A("bar")),
            mem::discriminant(&Foo::A("baz"))
        );
        let ff = mem::discriminant(&Foo::A("ff"));
        println!("{:?}", ff);
        let ff = mem::discriminant(&Foo::B(1));
        println!("{:?}", ff);
        assert_eq!(mem::discriminant(&Foo::B(1)), mem::discriminant(&Foo::B(2)));
        assert_ne!(mem::discriminant(&Foo::B(3)), mem::discriminant(&Foo::C(3)));
    }

    #[test]
    fn test2() {
        use std::rc::Rc;

        let mut data = Rc::new(5);

        *Rc::make_mut(&mut data) += 1; // 不会克隆
        let mut other_data = Rc::clone(&data); //此时还未复制
        *Rc::make_mut(&mut data) += 1; // 复制内部数据
        *Rc::make_mut(&mut data) += 1; // 复制后再次调用原指针将不会触发克隆
        *Rc::make_mut(&mut other_data) *= 2;

        // 现在 `data` 和 `other_data` 指向不同值
        assert_eq!(*data, 8);
        assert_eq!(*other_data, 12);
    }

    #[test]
    fn node_create() {
        let a_node: Node<IdStr> = Node::new(
            IdStr::from("is node item"),
            use_state(|| {
                [edge_index(IdStr::from("3"), IdStr::from("1"))]
                    .into_iter()
                    .collect()
            }),
            use_state(|| {
                [edge_index(IdStr::from("1"), IdStr::from("2"))]
                    .into_iter()
                    .collect()
            }),
        );
        let str_node: Node<IdStr> = Node::new(
            IdStr::from("is node item"),
            use_state(|| [edge_index("3", "1")].into_iter().collect()),
            use_state(|| [edge_index("1", "2")].into_iter().collect()),
        );
        println!("{:?}", a_node);
        println!("{:?}", str_node);
    }

    #[test]
    fn mut_graph_create() {
        insta::with_settings!({snapshot_path => Path::new("./insta")},{
            tracing_init(tracing::Level::DEBUG).unwrap();
            debug!("run mut_graph_create");
            let ww_node = Node::new(
                IdStr::from("ww_item"),
                use_state(||
                    [edge_index(IdStr::from("xx"), IdStr::from("ww"))]
                        .into_iter()
                        .collect(),
                ),
                use_state(||
                    [edge_index(IdStr::from("ww"), IdStr::from("xx"))]
                        .into_iter()
                        .collect(),
                ),
            );
            // @ graph ─────────────────────────────────────────────────────────────────

            let mut g1: Graph<IdStr, &'static str> = Graph::empty();

            // @ add node ─────────────────────────────────────────────────────────────────

            // let ww_nix = g1.insert_node_in_topo(String::from("ww"), String::from("ww_item"));
            g1.nodes.insert(IdStr::from("ww"), ww_node.clone());
            let ww_nix = node_index("ww");
            assert_eq!(
                IdStr::from("ww_item"),
                g1.get_node_item(&ww_nix).unwrap().clone()
            );

            let xx_nix = g1.insert_node_in_topo_only(IdStr::from("xx"), IdStr::from("xx_item"));

            // @ add edge ─────────────────────────────────────────────────────────────────

            let op_eix1 = g1
                .unused_insert_update_edge_in_topo(&ww_nix, &xx_nix, "ww->xx:item")
                .unwrap();
            assert_eq!(op_eix1, edge_index(IdStr::from("ww"), IdStr::from("xx")));
            let op_eix2 = g1
                .unused_insert_update_edge_in_topo(&xx_nix, &ww_nix, "xx->ww:item")
                .unwrap();
            assert_eq!(op_eix2, edge_index(IdStr::from("xx"), IdStr::from("ww")));
            // @ test ─────────────────────────────────────────────────────────────────
            println!("{:#?}", g1);

            // * match node item

            assert_eq!(
                ww_node.item,
                g1.nodes.get(&IdStr::from("ww")).unwrap().item
            );
            assert_eq!(
                ww_node.incoming_len,
                g1.nodes.get(&IdStr::from("ww")).unwrap().incoming_len
            );
            assert_eq!(
                ww_node.outgoing_len,
                g1.nodes.get(&IdStr::from("ww")).unwrap().outgoing_len
            );
            // * match node
            assert_eq!(&ww_node, g1.nodes.get(&IdStr::from("ww")).unwrap());
            assert_eq!(2, g1.node_count());
            assert_eq!(2, g1.edges_count());
            assert_eq!("ww->xx:item", *g1.edge_item(&op_eix1));
            assert_eq!("ww->xx:item", *g1.edge_item(&op_eix1));
            assert_eq!("xx->ww:item", *g1.edge_item(&op_eix2));
            assert_eq!((&ww_nix, &xx_nix), op_eix1.get_nix_s_unwrap());

            // @ remove edge ─────────────────────────────────────────────────────────────────
            g1.remove_edge_and_disconnect(&op_eix1);


            assert_eq!(1, g1.edges_count());

            let ww_node_rm_edge = Node::new(
                IdStr::from("ww_item"),
                use_state(||
                    [edge_index(IdStr::from("xx"), IdStr::from("ww"))]
                        .into_iter()
                        .collect(),
                ),
                use_state(IndexSet::default),
            );

            let xx_ww_edge = ww_node_rm_edge
                .incoming()
                .get_with(|x| x.first().unwrap().clone());
            let g1_xx_ww_edge = g1
                .nodes
                .get(&IdStr::from("ww"))
                .unwrap()
                .incoming()
                .get_with(|x| x.first().unwrap().clone());
            // 依然保有 xx->ww 边
            assert_eq!(xx_ww_edge, g1_xx_ww_edge);

            // @ remove node ─────────────────────────────────────────────────────────────────
            #[cfg(feature="insta")]
            insta::assert_display_snapshot!("graph_a",g1);
            g1.remove_node_and_edge_and_disconnect(node_index(IdStr::from("ww")));
            #[cfg(feature="insta")]
            insta::assert_display_snapshot!("graph_a_removed",g1);


            assert_eq!(1, g1.node_count());

            let mut g2: Graph<IdStr, &'static str> = Graph::empty();

            g2.insert_node_in_topo_only(IdStr::from("xx"), IdStr::from("xx_item"));

            debug!("=======================================================");
            debug!("g1{:?}", &g1);
            debug!("=======================================================");
            debug!("g2{:?}", &g2);
            debug!("=======================================================");
            assert!(g2.deep_eq_use_format_str(&g1));
            assert!(!g2.eq_sloppy(&g1));

            let g2clone = g2.clone();
            assert!(g2.eq_sloppy(&g2clone));
        });
    }

    #[test]
    fn hashmap_index() {
        let mut g: Graph<&str, &str> = Graph::empty();
        let l1_nix = g.insert_node_in_topo_only("1", "1");

        let n = &mut g[&l1_nix];

        println!("===================");
        *n = "edited";
        println!("{:?}", n);

        println!("{:#?}", &g);
        assert_eq!("edited", g[&l1_nix]);
    }
    #[test]
    #[allow(unused)]
    fn from_nodes_and_edges() {
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

        let mut g2 = Graph::from_nodes_and_edges_in_topo(
            [
                ("1", "1"),
                // • • • • •
                ("1.1", "1.1"),
                ("1.2", "1.2"),
                ("1.3", "1.3"),
                // • • • • •
                ("1.1.1", "1.1.1"),
                ("1.1.2", "1.1.2"),
                ("1.1.3", "1.1.3"),
                // • • • • •
                ("1.2.1", "1.2.1"),
                ("1.2.2", "1.2.2"),
                ("1.2.3", "1.2.3"),
                // • • • • •
                ("1.3.1", "1.3.1"),
                ("1.3.2", "1.3.2"),
                ("1.3.3", "1.3.3"),
            ]
            .as_ref(),
            [
                (("1", "1.1"), "1->1.1 :edge_item"),
                (("1", "1.2"), "1->1.2 :edge_item"),
                (("1", "1.3"), "1->1.3 :edge_item"),
                // • • • • •
                (("1.1", "1.1.1"), "1.1->1.1.1 :edge_item"),
                (("1.1", "1.1.2"), "1.1->1.1.2 :edge_item"),
                (("1.1", "1.1.3"), "1.1->1.1.3 :edge_item"),
                // • • • • •
                (("1.2", "1.2.1"), "1.2->1.2.1 :edge_item"),
                (("1.2", "1.2.2"), "1.2->1.2.2 :edge_item"),
                (("1.2", "1.2.3"), "1.2->1.2.3 :edge_item"),
                // • • • • •
                (("1.3", "1.3.1"), "1.3->1.3.1 :edge_item"),
                (("1.3", "1.3.2"), "1.3->1.3.2 :edge_item"),
                (("1.3", "1.3.3"), "1.3->1.3.3 :edge_item"),
            ]
            .as_ref(),
        );

        println!("{:#?}", &g);
        println!("================================");
        println!("{:#?}", &g2);
        println!("================================");
        assert!(g.deep_eq_use_format_str(&g2));
    }
}
