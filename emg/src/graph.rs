/*
 * @Author: Rais
 * @Date: 2020-12-28 16:48:19
 * @LastEditTime: 2023-02-10 18:40:22
 * @LastEditors: Rais
 * @Description:
 */

mod dir;
mod edges;

mod iter_format;
mod neighbors;

use crate::{
    error::Error,
    im::{
        hashmap::{self, Entry},
        HashMap,
    },
};
pub use edges::NodeEdgesIter;
use emg_common::display::DictDisplay;
use emg_state::{
    state_store, topo, use_state, CloneStateAnchor, CloneStateVar, Dict, GStateStore, StateAnchor,
    StateVar,
};
use indented::{indented, indented_with};
use iter_format::{DebugMap, IterFormatExt};
use neighbors::NodeNeighborsIter;
use owning_ref::RcRef;
use std::fmt::Write;
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

pub type NodeCollect<N, Ix> = HashMap<Ix, Node<N, Ix>, BuildHasherDefault<CustomHasher>>;
// type OutGoingEdgeVec<Ix> = SmallVec<[EdgeIndex<Ix>; OUT_EDGES_SIZE]>;
pub type EdgeCollect<Ix> = IndexSet<EdgeIndex<Ix>, BuildHasherDefault<CustomHasher>>;

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
#[derive(Copy, Clone, Default, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct NodeIndex<Ix>(Ix);

impl<Ix> Deref for NodeIndex<Ix> {
    type Target = Ix;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<Ix> NodeIndex<Ix> {
    #[inline]
    pub fn new(x: Ix) -> Self {
        NodeIndex(x)
    }
    // #[inline]
    // pub fn index(self) -> Ix {
    //     self.index_ref().clone()
    // }
    #[inline]
    pub fn index(&self) -> &Ix {
        &self.0
    }

    // NOTE change Ix may out-of-control
    // pub fn set_index(&mut self, ix: Ix) -> &mut Self {
    //     self.0 = ix;
    //     self
    // }
}
// impl<Ix> Clone for NodeIndex<Ix>
// where
//     Ix: Clone,
// {
//     fn clone(&self) -> Self {
//         NodeIndex(self.0.clone())
//     }

//     fn clone_from(&mut self, source: &Self) {
//         self.0.clone_from(&source.0);
//     }
// }

impl<Ix> From<Ix> for NodeIndex<Ix> {
    fn from(ix: Ix) -> Self {
        NodeIndex(ix)
    }
}

impl<Ix> Debug for NodeIndex<Ix>
where
    Ix: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "NodeIndex({:?})", &self.0)
    }
}
impl<Ix> std::fmt::Display for NodeIndex<Ix>
where
    Ix: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "▣ {}", &self.0)
    }
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OptionNodeIndex<Ix>(Option<NodeIndex<Ix>>);

impl<Ix> OptionNodeIndex<Ix> {
    pub fn as_ref(&self) -> Option<&NodeIndex<Ix>> {
        match **self {
            Some(ref x) => Some(x),
            None => None,
        }
    }
}

impl<Ix> From<Ix> for OptionNodeIndex<Ix> {
    fn from(value: Ix) -> Self {
        Self(Some(NodeIndex::new(value)))
    }
}

impl<Ix> From<NodeIndex<Ix>> for OptionNodeIndex<Ix> {
    fn from(value: NodeIndex<Ix>) -> Self {
        Self(Some(value))
    }
}

impl<Ix> From<Option<NodeIndex<Ix>>> for OptionNodeIndex<Ix> {
    fn from(value: Option<NodeIndex<Ix>>) -> Self {
        Self(value)
    }
}
// impl<Ix: Clone> From<Option<&NodeIndex<Ix>>> for OptionNodeIndex<Ix> {
//     fn from(value: Option<&NodeIndex<Ix>>) -> Self {
//         Self(value.cloned())
//     }
// }

impl<Ix> Deref for OptionNodeIndex<Ix> {
    type Target = Option<NodeIndex<Ix>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// @ EdgeIndex ────────────────────────────────────────────────────────────────────────────────
#[derive(Copy, Clone, Default, PartialEq, Eq, Hash)]
//TODO 包裹 Option<NodeIndex<Ix>> 为新 struct 可以更好的 impl into等
pub struct EdgeIndex<Ix>(OptionNodeIndex<Ix>, OptionNodeIndex<Ix>);

impl<Ix> PartialOrd for EdgeIndex<Ix>
where
    Ix: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0
            .partial_cmp(&other.0)
            .and(self.1.partial_cmp(&other.1))
    }
}

impl<Ix> Ord for EdgeIndex<Ix>
where
    Ix: Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0).then(self.1.cmp(&other.1))
    }
}

impl<Ix> Display for EdgeIndex<Ix>
where
    Ix: std::fmt::Display,
{
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

impl<Ix> EdgeIndex<Ix> {
    #[inline]
    pub fn new(s: impl Into<OptionNodeIndex<Ix>>, t: impl Into<OptionNodeIndex<Ix>>) -> Self {
        EdgeIndex(s.into(), t.into())
    }

    // #[inline]
    // pub fn ix_s(&self) -> &(Ix, Ix) {
    //     &self.0
    // }
    #[inline]
    pub fn source_nix(&self) -> &Option<NodeIndex<Ix>> {
        &self.0
    }

    /// Return the target node index.
    #[inline]
    pub fn target_nix(&self) -> &Option<NodeIndex<Ix>> {
        &self.1
    }

    pub fn set_incoming(&mut self, nix: OptionNodeIndex<Ix>) {
        self.0 = nix;
    }
    pub fn set_outgoing(&mut self, nix: OptionNodeIndex<Ix>) {
        self.1 = nix;
    }
    pub fn with_incoming(mut self, nix: OptionNodeIndex<Ix>) -> Self {
        self.0 = nix;
        self
    }
    pub fn with_outgoing(mut self, nix: OptionNodeIndex<Ix>) -> Self {
        self.1 = nix;
        self
    }
    // pub fn set_incoming(&mut self, nix: NodeIndex<Ix>) -> &Self {
    //     self.0 .0 = nix.0;
    //     self
    // }
    // pub fn set_outgoing(&mut self, nix: NodeIndex<Ix>) -> &Self {
    //     self.0 .1 = nix.0;
    //     self
    // }

    pub fn nix_by_dir(&self, dir: Direction) -> &Option<NodeIndex<Ix>> {
        match dir {
            Outgoing => &self.1,
            Incoming => &self.0,
        }
    }

    pub fn get_nix_s(&self) -> (&Option<NodeIndex<Ix>>, &Option<NodeIndex<Ix>>) {
        let Self(s, t) = self;
        (s, t)
    }
    pub fn get_nix_s_unwrap(&self) -> (&NodeIndex<Ix>, &NodeIndex<Ix>) {
        let Self(s, t) = self;
        (s.as_ref().unwrap(), t.as_ref().unwrap())
    }

    // fn _into_node(self) -> NodeIndex<Ix> {
    //     NodeIndex(self.0)
    // }
}

impl<Ix, I, O> From<(I, O)> for EdgeIndex<Ix>
where
    I: Into<OptionNodeIndex<Ix>>,
    O: Into<OptionNodeIndex<Ix>>,
{
    fn from((s, t): (I, O)) -> Self {
        EdgeIndex::new(s, t)
    }
}

impl<Ix> Debug for EdgeIndex<Ix>
where
    Ix: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "EdgeIndex({:?}->{:?})", &self.0, &self.1)
    }
}
// @ make index ────────────────────────────────────────────────────────────────────────────────

/// Short version of `NodeIndex::new`
#[inline]
pub fn node_index<Ix>(index: impl Into<Ix>) -> NodeIndex<Ix> {
    NodeIndex(index.into())
}

/// Short version of `EdgeIndex::new`
#[inline]
pub fn edge_index<Ix>(s: impl Into<Ix>, t: impl Into<Ix>) -> EdgeIndex<Ix> {
    EdgeIndex::new(node_index(s), node_index(t))
}
#[inline]
pub fn edge_index_no_source<Ix>(t: impl Into<Ix>) -> EdgeIndex<Ix> {
    EdgeIndex::new(None::<NodeIndex<Ix>>, node_index(t))
}

// ────────────────────────────────────────────────────────────────────────────────

// const DIRECTIONS: [Direction; 2] = [Outgoing, Incoming];

/// @ Node ────────────────────────────────────────────────────────────────────────────────
#[derive(Eq)]
pub struct Node<N, Ix>
where
    Ix: Clone + std::hash::Hash + std::cmp::Eq,
{
    /// 内容
    pub item: N,
    /// Next edge in outgoing and incoming edge lists.
    //TODO check 要有序
    incoming_eix_set: StateVar<EdgeCollect<Ix>>,
    outgoing_eix_set: StateVar<EdgeCollect<Ix>>, //TODO use smvec
    incoming_len: StateAnchor<usize>,
    outgoing_len: StateAnchor<usize>,
}

impl<N: Display, Ix: Display> Display for Node<N, Ix>
where
    Ix: Clone + std::hash::Hash + std::cmp::Eq + 'static,
{
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

impl<N: PartialEq, Ix: PartialEq> PartialEq for Node<N, Ix>
where
    Ix: Clone + std::hash::Hash + std::cmp::Eq,
{
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
impl<N, Ix> Debug for Node<N, Ix>
where
    N: Debug,
    Ix: Debug + Clone + Eq + Hash + 'static,
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

impl<N, Ix> Node<N, Ix>
where
    Ix: Clone + Hash + Eq + std::fmt::Debug + 'static,
{
    #[topo::nested]
    pub fn new_in_topo(item: N) -> Self {
        let incoming_eix_set: StateVar<EdgeCollect<Ix>> =
            use_state(|| EdgeCollect::<Ix>::default());
        let incoming_len = incoming_eix_set.watch().map(|ins| ins.len());
        let outgoing_eix_set: StateVar<EdgeCollect<Ix>> =
            use_state(|| EdgeCollect::<Ix>::default());
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
        incoming_eix_set: StateVar<EdgeCollect<Ix>>,
        outgoing_eix_set: StateVar<EdgeCollect<Ix>>,
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

    pub fn edge_out_ixs(&self) -> Rc<EdgeCollect<Ix>> {
        self.outgoing_eix_set.get_rc()
    }
    pub fn edge_ixs(&self, dir: Direction) -> EdgeCollect<Ix> {
        match dir {
            // Incoming => self.incoming().clone(),
            Incoming => self.incoming().get(),
            Outgoing => self.outgoing().get(),
        }
    }
    pub fn edge_ixs_sa(&self, dir: Direction) -> &StateVar<EdgeCollect<Ix>> {
        match dir {
            // Incoming => self.incoming().clone(),
            Incoming => self.incoming(),
            Outgoing => self.outgoing(),
        }
    }

    /// Accessor for data structure internals: the first edge in the given direction.

    pub fn incoming(&self) -> &StateVar<EdgeCollect<Ix>> {
        &self.incoming_eix_set
    }
    pub fn outgoing(&self) -> &StateVar<EdgeCollect<Ix>> {
        &self.outgoing_eix_set
    }
    pub fn incoming_mut_with<F: FnOnce(&mut EdgeCollect<Ix>)>(&self, func: F) {
        self.incoming_eix_set.update(func)
    }
    pub fn outgoing_mut_with<F: FnOnce(&mut EdgeCollect<Ix>)>(&self, func: F) {
        self.outgoing_eix_set.update(func)
    }

    pub fn remove_plug(&self, dir: Direction, e_ix: &EdgeIndex<Ix>) -> Option<EdgeIndex<Ix>> {
        let removed = match dir {
            Incoming => {
                if self.incoming_len.get() == 1 {
                    let old = self
                        .incoming_eix_set
                        .get_with(|ins| ins.get(e_ix).cloned())?;
                    self.incoming_eix_set.set(IndexSet::default());
                    old
                } else {
                    let mut old_eix_s = self.incoming_eix_set.get();
                    let old = old_eix_s.shift_take(e_ix)?;
                    self.incoming_eix_set.set(old_eix_s);
                    old
                }
            }
            Outgoing => {
                if self.outgoing_len.get() == 1 {
                    let old = self
                        .outgoing_eix_set
                        .get_with(|outs| outs.get(e_ix).cloned())?;
                    self.outgoing_eix_set.set(IndexSet::default());
                    old
                } else {
                    let mut old_eix_s = self.outgoing_eix_set.get();
                    let old = old_eix_s.shift_take(e_ix)?;

                    // let old = old_eix_s.remove(old_eix_s.index_of(e_ix)?); //TODO 检索性能, 测试 use indexSet 库
                    self.outgoing_eix_set.set(old_eix_s);
                    old
                }
            }
        };
        assert_eq!(&removed, e_ix);

        Some(removed)
    }

    pub fn incoming_len(&self) -> usize {
        self.incoming_len.get()
    }
}

impl<N, Ix> Clone for Node<N, Ix>
where
    N: Clone,
    Ix: Clone + Eq + Hash,
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

// impl<N, Ix> illicit::AsContext for Node<N, Ix>
// where
//     Node<N, Ix>: std::fmt::Debug + Sized + 'static,
// {
//     fn offer<R>(self, op: impl FnOnce() -> R) -> R {
//         todo!()
//     }
// }

/// @ Edge ────────────────────────────────────────────────────────────────────────────────
/// The graph's edge type.
// aef struct aef
type EdgeNodeIxSv<Ix> = StateVar<Option<NodeIndex<Ix>>>;
#[derive(Debug, PartialEq, Eq)]
pub struct Edge<E, Ix>
where
    Ix: Clone + 'static,
{
    /// Associated edge data.
    pub item: E,

    source_nix: EdgeNodeIxSv<Ix>,
    target_nix: EdgeNodeIxSv<Ix>,
}

impl<E: Display, Ix: Display> Display for Edge<E, Ix>
where
    Ix: Clone + 'static,
{
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

// impl<E, Ix> PartialEq for Edge<E, Ix>
// where
//     E: std::cmp::PartialEq,
//     Ix: Clone + 'static + std::cmp::PartialEq,
// {
//     fn eq(&self, other: &Self) -> bool {
//         self.item == other.item
//             && self.source_nix.id() == other.source_nix.id()
//             && self.target_nix.id() == other.target_nix.id()
//             && self.source_nix.get() == other.source_nix.get()
//             && self.target_nix.get() == other.target_nix.get()
//     }
// }
// impl<E, Ix> Eq for Edge<E, Ix>
// where
//     E: std::cmp::PartialEq,
//     Ix: Clone + 'static + std::cmp::PartialEq,
// {
// }

impl<E, Ix> std::ops::Deref for Edge<E, Ix>
where
    Ix: Clone + 'static,
{
    type Target = E;

    fn deref(&self) -> &Self::Target {
        &self.item
    }
}
#[cfg(test)]
#[topo::nested]
pub fn edge_in_topo<E: Clone, Ix: Clone + std::fmt::Debug>(
    s: NodeIndex<Ix>,
    t: NodeIndex<Ix>,
    item: E,
) -> Edge<E, Ix> {
    Edge::new_in_topo(Some(s), Some(t), item)
}

impl<E, Ix> Edge<E, Ix>
where
    E: Clone,
    Ix: Clone + std::fmt::Debug,
{
    #[cfg(test)]
    #[topo::nested]
    pub fn new_in_topo(
        source_nix: Option<NodeIndex<Ix>>,
        target_nix: Option<NodeIndex<Ix>>,
        item: E,
    ) -> Self {
        Self {
            item,
            source_nix: use_state(|| source_nix),
            target_nix: use_state(|| target_nix),
        }
    }
    pub fn new(
        opt_source_nix_sv: EdgeNodeIxSv<Ix>,
        opt_target_nix_sv: EdgeNodeIxSv<Ix>,
        item: E,
    ) -> Self {
        Self {
            item,
            source_nix: opt_source_nix_sv,
            target_nix: opt_target_nix_sv,
        }
    }

    // /// Accessor for data structure internals: the next edge for the given direction.
    // pub fn next_edge(&self, dir: Direction) -> EdgeIndex<Ix> {
    //     self.next[dir.index()]
    // }

    /// Return the source node index.

    pub fn node_ix(&self, dir: Direction) -> &EdgeNodeIxSv<Ix> {
        match dir {
            Outgoing => self.target_nix(),
            Incoming => self.source_nix(),
        }
    }
    pub fn source_nix(&self) -> &EdgeNodeIxSv<Ix> {
        &self.source_nix
    }

    /// Return the target node index.
    pub fn target_nix(&self) -> &EdgeNodeIxSv<Ix> {
        &self.target_nix
    }

    pub fn endpoints(&self) -> (EdgeNodeIxSv<Ix>, EdgeNodeIxSv<Ix>) {
        (*self.source_nix(), *self.target_nix())
    }

    /// Get a reference to the edge's item.
    pub fn item(&self) -> &E {
        &self.item
    }
}

impl<E, Ix> Clone for Edge<E, Ix>
where
    Ix: Clone,
    E: Clone,
{
    clone_fields!(Edge, item, source_nix, target_nix);
}

// @ Graph ────────────────────────────────────────────────────────────────────────────────

pub struct Graph<N, E, Ix = String>
where
    Ix: Eq + Hash + Clone + PartialOrd + Ord + 'static,
    N: Clone,
    E: Clone,
{
    store: Rc<RefCell<GStateStore>>,
    nodes: NodeCollect<N, Ix>,
    pub edges: StateVar<Dict<EdgeIndex<Ix>, Edge<E, Ix>>>,
}

impl<N, E, Ix> Display for Graph<N, E, Ix>
where
    Ix: Eq + Hash + Clone + PartialOrd + Ord + 'static,
    N: Clone,
    E: Clone + 'static,
    N: Display,
    E: Display,
    Ix: Display,
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

// impl<N, E, Ix> PartialEq for Graph<N, E, Ix>
// where
//     N: Clone + PartialEq,
//     E: Clone + PartialEq,
//     Ix: std::hash::Hash + std::clone::Clone + std::cmp::Ord + Eq,
// {
//     fn eq(&self, other: &Self) -> bool {
//         self.nodes == other.nodes && self.edges == other.edges
//     }
// }

impl<N, E, Ix> Clone for Graph<N, E, Ix>
where
    Ix: Clone + Eq + Eq + Hash + PartialOrd + Ord,
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

impl<N, E, Ix> fmt::Debug for Graph<N, E, Ix>
where
    Ix: fmt::Debug + Clone + Hash + Eq + PartialOrd + Ord + 'static,
    N: fmt::Debug + Clone,
    E: fmt::Debug + Clone + 'static,
    // Edge<E, Ix>: fmt::Debug + Clone,
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
                        .collect::<Vec<(&Ix, &Node<N, Ix>)>>()
                }),
            );
        }
        if self.edges_count() > 0 {
            let es = self.edges.store_get(&self.store());
            fmt_struct.field("edges", &es.iter().map(|(eix, _e)| eix).format(",\n"));

            if size_of::<E>() != 0 {
                fmt_struct.field(
                    "edge item",
                    &DebugMap(|| (&es).iter().map(|(eix, e)| (eix, e)).collect::<Vec<_>>()),
                );
            }
        }

        fmt_struct.finish()
    }
}

impl<N, E, Ix> Default for Graph<N, E, Ix>
where
    Ix: Clone + Hash + Eq + PartialOrd + Ord + 'static + std::fmt::Debug,
    E: Clone + 'static + std::fmt::Debug,
    N: Clone,
{
    #[topo::nested]
    fn default() -> Self {
        Self {
            store: state_store(),
            nodes: HashMap::default(),
            edges: use_state(|| Dict::new()),
        }
    }
}

type EdgeRef<R, E, Ix> = RcRef<Dict<EdgeIndex<Ix>, Edge<E, Ix>>, R>;

impl<N, E, Ix> Graph<N, E, Ix>
where
    Ix: Clone + Hash + Eq + PartialOrd + Ord + 'static + std::fmt::Debug,
    E: Clone + 'static + std::fmt::Debug,
    N: Clone,
    EdgeIndex<Ix>: Clone,
{
    pub fn eq_sloppy(&self, other: &Self) -> bool
    where
        N: Clone + PartialEq,
        E: Clone + PartialEq,
    {
        self.nodes == other.nodes && self.edges == other.edges
    }

    #[topo::nested]
    pub fn new_with_in_topo(
        nodes: NodeCollect<N, Ix>,
        edges: Dict<EdgeIndex<Ix>, Edge<E, Ix>>,
    ) -> Self {
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

    // pub fn deep_eq(&self, other: &Self) -> bool
    // where
    //     Ix: Eq,
    //     N: Eq,
    //     E: Eq,
    // {
    //     if self.nodes.len() != other.nodes.len() {
    //         debug!("deep_eq nodes.len() not same");

    //         return false;
    //     }
    //     let nodes_eq = self.nodes == other.nodes;

    //     let edges_eq = self.edges.store_get(&self.store()) == other.edges.store_get(&other.store());
    //     debug!("deep_eq nodes:{} edges:{}", nodes_eq, edges_eq);

    //     nodes_eq && edges_eq
    //     // && format!("{:?}", self.edges.get()) == format!("{:?}", other.edges.get())
    // }
    pub fn deep_eq_use_format_str(&self, other: &Self) -> bool
    where
        // E: Eq,
        Ix: Eq + fmt::Debug,
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

    pub fn get_node_item(&self, a: &NodeIndex<Ix>) -> Option<&N> {
        self.nodes.get(a.index()).map(|n| &n.item)
    }
    pub fn get_node_use_ix(&self, ix: &Ix) -> Option<&Node<N, Ix>> {
        self.nodes.get(ix)
    }
    pub fn get_node_item_use_ix(&self, ix: &Ix) -> Option<&N> {
        self.nodes.get(ix).map(|n| &n.item)
    }
    pub fn get_mut_node_item_use_ix(&mut self, ix: &Ix) -> Option<&mut N> {
        self.nodes.get_mut(ix).map(|n| &mut n.item)
    }
    /// Access the item for node `a`, mutably.
    ///
    /// Also available with indexing syntax: `&mut graph[a]`.
    pub fn get_mut_node_item(&mut self, a: &NodeIndex<Ix>) -> Option<&mut N> {
        self.nodes.get_mut(a.index()).map(|n| &mut n.item)
    }

    #[topo::nested]
    pub fn insert_node_in_topo(&mut self, key: Ix, item: N) -> NodeIndex<Ix> {
        let node = Node::new_in_topo(item);
        let node_idx = node_index(key.clone());
        self.nodes.insert(key, node);

        node_idx
    }
    //TODO use topo key? 同id 不同key ,实现shadow-nodeItem(not shadow-tree(node))
    pub fn or_insert_node_with_plugs(
        &mut self,
        key: Ix,
        item: N,
        incoming_eix_set: StateVar<EdgeCollect<Ix>>,
        outgoing_eix_set: StateVar<EdgeCollect<Ix>>,
    ) -> NodeIndex<Ix> {
        let node = Node::new(item, incoming_eix_set, outgoing_eix_set);
        let node_idx = node_index(key.clone());
        // 直接 ─────────────────────────────────────────────────────────────
        // self.nodes.insert(key, node);
        // ─────────────────────────────────────────────────────────────
        //TODO remove clone for if no need warn!
        let ent = self.nodes.entry(key.clone());

        if let Entry::Vacant(entry) = ent {
            entry.insert(node);
        } else {
            warn!("id:{:?} already exists", &key);
        }
        // ─────────────────────────────────────────────────────────────

        node_idx
    }

    pub fn nodes_contains_key(&self, key: &Ix) -> bool {
        self.nodes.contains_key(key)
    }
    // pub fn insert_root(&mut self, key: Ix, item: N, edge_item: E) -> NodeIndex<Ix> {
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
        s_nix: &NodeIndex<Ix>,
        t_nix: &NodeIndex<Ix>,
        item: E,
    ) -> Option<EdgeIndex<Ix>>
// where
        //     Ix: Eq,
        //     E: Eq,
    {
        // if self.nodes.contains_key(&a.index()) {
        //     return None;
        // }
        // if self.nodes.contains_key(&b.index()) {
        //     return None;
        // }

        let edge_idx = EdgeIndex::new(s_nix.clone(), t_nix.clone());

        self.nodes_connect_eix(&edge_idx)?;

        let edge = Edge::new_in_topo(Some(s_nix.clone()), Some(t_nix.clone()), item);

        self.just_insert_edge(edge_idx.clone(), edge);

        Some(edge_idx)
    }

    //TODO return Result
    pub fn nodes_connect_eix(&self, edge_index: &EdgeIndex<Ix>) -> Option<()> {
        if let Some(s_nix) = edge_index.source_nix() {
            self.nodes.get(s_nix.index())?.outgoing_mut_with(|outs| {
                outs.insert(edge_index.clone());
            });
        }

        if let Some(t_nix) = edge_index.target_nix() {
            self.nodes.get(t_nix.index())?.incoming_mut_with(|in_s| {
                in_s.insert(edge_index.clone());
            });
        }

        Some(())
    }

    pub fn just_insert_edge(&mut self, edge_idx: EdgeIndex<Ix>, edge: Edge<E, Ix>)
    // where
    //     Ix: Eq,
    //     E: Eq,
    {
        self.edges.store_update(&self.store(), |es| {
            trace!(
                "has edge?-- {:?} --{}",
                &edge_idx,
                es.contains_key(&edge_idx)
            );
            // if es.contains_key(&edge_idx) {
            //     let e = es.get(&edge_idx).unwrap();
            //     assert_eq!(e, &edge);
            // }
            es.insert(edge_idx, edge);
        })
    }

    #[must_use]
    pub fn edge(&self, e: &EdgeIndex<Ix>) -> EdgeRef<Edge<E, Ix>, E, Ix> {
        RcRef::new(self.edges.store_get_rc(&self.store())).map(|f| &f[e])
    }

    #[must_use]
    pub fn edge_item(&self, e: &EdgeIndex<Ix>) -> EdgeRef<E, E, Ix> {
        RcRef::new(self.edges.store_get_rc(&self.store())).map(|f| &f[e].item)
    }

    #[must_use]
    pub fn edge_source(&self, e: &EdgeIndex<Ix>) -> EdgeRef<EdgeNodeIxSv<Ix>, E, Ix> {
        RcRef::new(self.edges.store_get_rc(&self.store())).map(|f| f[e].source_nix())
    }

    pub fn edge_plug_edit(&self, eix: &EdgeIndex<Ix>, dir: Direction, change_to: impl Into<Ix>) {
        match dir {
            Outgoing => {
                //TODO finish this like incoming
                // edge.target_nix.set(Some(node_index(change_to)))
                todo!()
            }
            Incoming => {
                let new_incoming_nix = Some(node_index(change_to));
                let new_eix = eix.clone().with_incoming(new_incoming_nix.clone().into());

                self.just_remove_plug_in_nodes(eix);

                // add in nodes ─────────────────────────────────────────────
                self.nodes_connect_eix(&new_eix);

                //edges ─────────────────────────────────────────────────────────────────────────────

                self.edges.store_update(&self.store(), |edges| {
                    //TODO 当前在 FnOnce中 无法 返回 result , 暂时只能 unwrap
                    let edge = edges.remove(eix).ok_or(Error::CanNotGetEdge).unwrap();
                    edge.source_nix.set(new_incoming_nix);
                    edges.insert(new_eix, edge);
                });
                // ─────────────────────────────────────────────
                // //old source node remove eix a->c change to  a!=>c
                // self.disconnect_plug_in_node_with_dir(eix.source_nix(), Outgoing, eix);
                // // old target node's incoming eix change to new eix   ,c的 incoming eix a->c 改成 b->c (删除 a->c 添加 b->c)
                // self.update_plug_in_node_with_dir(eix.target_nix(), Incoming, eix, ||panic!("在node修改源过程中,类似 a->c 变成 b->c, c肯定要有一个已有的edge a->c, 但这里没有"),|eix| {
                //     eix.set_incoming(new_incoming_nix.clone().into())
                // });
                // //new incoming node set target eix, b的 outgoing eix 添加 b->c
                // let mut new_incoming_eix = eix.clone();
                // new_incoming_eix.set_incoming(new_incoming_nix.into());
                // self.nodes_connect_eix(&new_incoming_eix);
                // remove edge in node ─────────────────────────────────────────────
            }
        }
    }

    fn just_remove_plug_in_nodes<'a>(&self, e_ix: &'a EdgeIndex<Ix>) -> Option<&'a EdgeIndex<Ix>> {
        let (source_n, target_n) = e_ix.get_nix_s();

        self.disconnect_plug_in_node_with_dir(source_n, Outgoing, e_ix);
        self.disconnect_plug_in_node_with_dir(target_n, Incoming, e_ix);

        Some(e_ix)
    }

    //移除 opt_n_ix (如果有) 的 dir 方向 的 edgeindex记录- e_ix
    fn disconnect_plug_in_node_with_dir(
        &self,
        opt_n_ix: &Option<NodeIndex<Ix>>,
        dir: Direction,
        e_ix: &EdgeIndex<Ix>,
    ) -> Option<EdgeIndex<Ix>> {
        opt_n_ix
            .as_ref()
            .and_then(|n_ix| self.nodes.get(n_ix.index())?.remove_plug(dir, e_ix))
    }

    fn update_plug_in_node_with_dir<F, Fdef>(
        &self,
        opt_n_ix: &Option<NodeIndex<Ix>>,
        dir: Direction,
        e_ix: &EdgeIndex<Ix>,
        not_has_new_fn: Fdef,
        update_fn: F,
    ) -> Option<()>
    where
        F: FnOnce(&mut EdgeIndex<Ix>),
        Fdef: FnOnce() -> EdgeIndex<Ix>,
    {
        if let Some(n_ix) = opt_n_ix {
            let n = self.nodes.get(n_ix.index())?;
            let edge_ixs_sa = n.edge_ixs_sa(dir);
            let mut e_sets = edge_ixs_sa.get();

            if let Some((i, mut the_eix)) = e_sets.swap_remove_full(e_ix) {
                //a-c -> b->c
                update_fn(&mut the_eix);
                let (end_i, is_inserted) = e_sets.insert_full(the_eix);
                debug_assert!(is_inserted);
                e_sets.swap_indices(i, end_i);
            } else {
                // not has , add new
                let is_inserted = e_sets.insert(not_has_new_fn());
                debug_assert!(is_inserted);
            }

            edge_ixs_sa.set(e_sets);

            Some(())
        } else {
            None
        }
    }

    /// Remove an edge and return its edge item, or `None` if it didn't exist.
    ///
    /// Apart from `e`, this invalidates the last edge index in the graph
    /// (that edge will adopt the removed edge index).
    ///
    /// Computes in **O(e')** time, where **e'** is the size of four particular edge lists, for
    /// the vertices of `e` and the vertices of another affected edge.
    pub fn remove_edge<'a>(&mut self, e_ix: &'a EdgeIndex<Ix>) -> Option<&'a EdgeIndex<Ix>> {
        // remove edge
        self.just_remove_edge_in_edges(e_ix);

        // remove edge in node
        self.just_remove_plug_in_nodes(e_ix);

        Some(e_ix)
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
    pub fn remove_node(&mut self, n: NodeIndex<Ix>) -> Option<N> {
        let Node {
            incoming_eix_set: incoming,
            outgoing_eix_set: outgoing,
            item,
            ..
        } = self.nodes.remove(&n)?;

        // 断开 node 进出 连接
        // TODO: Rc - ENGINE in graph
        for n_in_e_ix in incoming.get_rc().iter() {
            self.disconnect_plug_in_node_with_dir(n_in_e_ix.source_nix(), Outgoing, n_in_e_ix);
            self.just_remove_edge_in_edges(n_in_e_ix);
        }
        // TODO: Rc - ENGINE in graph
        for n_out_e_ix in outgoing.get_rc().iter() {
            self.disconnect_plug_in_node_with_dir(n_out_e_ix.target_nix(), Incoming, n_out_e_ix);
            self.just_remove_edge_in_edges(n_out_e_ix);
        }

        Some(item)
    }

    fn just_remove_edge_in_edges(&self, eix: &EdgeIndex<Ix>) {
        self.edges.store_update(&self.store(), |es| {
            es.remove(eix)
                .expect("edges can't remove not find EdgeIndex.");
        })
    }

    /// ## iter NodeIndex 的 边 , 从 edgeIndex 中 取出边衔接的另一头 NodeIndex,
    // 迭代 与 A node dir 方向相连的 NodeIndex
    /// * return: NodeIndex,  not edge  /  以及另头的Node
    pub fn neighbors_consuming_iter(
        &self,
        nix: &NodeIndex<Ix>,
        dir: Direction,
    ) -> NodeNeighborsIter<Ix, NodeEdgesConsumingIter<Ix>> {
        NodeNeighborsIter::new(self.deprecated_edges_consuming_iter(nix, dir))
    }
    // pub fn neighbors_iter(&self, nix: &NodeIndex<Ix>, dir: Direction) -> NodeNeighborsIter<Ix> {
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
    pub fn deprecated_edges_consuming_iter(
        &self,
        nix: &Ix,
        dir: Direction,
    ) -> NodeEdgesConsumingIter<Ix> {
        let node = self
            .nodes
            .get(nix)
            .unwrap_or_else(|| panic!(":: not find node for id:{:?}!", nix));

        NodeEdgesConsumingIter::new(dir, node.edge_ixs(dir).into_iter())
    }
    // pub fn edges_iter_use_ix(&self, ix: &Ix, dir: Direction) -> NodeEdgesIter<Ix> {
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
    // /// Iterator element type is `EdgeReference<E, Ix>`.
    // pub fn edges_connecting(
    //     &self,
    //     a: NodeIndex<Ix>,
    //     b: NodeIndex<Ix>,
    // ) -> EdgesConnecting<E, Ty, Ix> {
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
    // pub fn contains_edge(&self, a: NodeIndex<Ix>, b: NodeIndex<Ix>) -> bool {
    //     self.find_edge(a, b).is_some()
    // }

    // * 找到 A B 直之间一个边
    // /// Lookup an edge from `a` to `b`.
    // ///
    // /// Computes in **O(e')** time, where **e'** is the number of edges
    // /// connected to `a` (and `b`, if the graph edges are undirected).
    // pub fn find_edge(&self, a: NodeIndex<Ix>, b: NodeIndex<Ix>) -> Option<EdgeIndex<Ix>> {
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
    //     node: &Node<N, Ix>,
    //     b: NodeIndex<Ix>,
    // ) -> Option<EdgeIndex<Ix>> {
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
    // pub fn externals(&self, dir: Direction) -> Externals<N, Ty, Ix> {
    //     Externals {
    //         iter: self.nodes.iter().enumerate(),
    //         dir,
    //         ty: PhantomData,
    //     }
    // }

    /// Access the internal node array.
    pub fn raw_nodes(&self) -> &NodeCollect<N, Ix> {
        &self.nodes
    }

    /// Access the internal edge array.
    pub fn raw_edges(&self) -> &StateVar<Dict<EdgeIndex<Ix>, Edge<E, Ix>>> {
        &self.edges
    }
    // /// Access the internal edge array.
    // pub fn get_raw_edges(&self) -> StateVar<Dict<EdgeIndex<Ix>, Edge<E, Ix>>> {
    //     self.edges
    // }
    pub fn get_raw_edges_watch(&self) -> StateAnchor<Dict<EdgeIndex<Ix>, Edge<E, Ix>>> {
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
            pub fn retain_nodes<F: FnMut(&Ix, &Node<N, Ix>) -> bool>(&mut self, f:F);

        }
        // to self.edges {

        //     #[call(retain)]
        //     pub fn retain_edges<F: FnMut(&(Ix,Ix), &Edge<E, Ix>) -> bool>(&mut self, f:F);

        // }
    }

    #[cfg(test)]
    #[topo::nested]
    pub fn from_nodes_and_edges_in_topo(
        nodes: &[(Ix, N)],
        edges: &[((Ix, Ix), E)],
    ) -> Graph<N, E, Ix> {
        let handled_nodes = nodes
            .iter()
            .cloned()
            .map(|(k, w)| (k, Node::new_in_topo(w)));
        // let mut g_nodes: HashMap<Ix, Node<N, Ix>> = HashMap::from_iter(handled_nodes);
        let mut g_nodes: NodeCollect<N, Ix> = handled_nodes.collect();
        // let handled_edges = edges.iter().cloned().map(|(k, w)| (k, w.into()));
        let mut g_edges: Dict<EdgeIndex<Ix>, Edge<E, Ix>> = Dict::new();
        for ((s, t), ew) in edges {
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
impl<N, E, Ix> Index<NodeIndex<Ix>> for Graph<N, E, Ix>
where
    N: Clone,
    Ix: Eq + Hash + Clone + PartialOrd + Ord,
    E: Clone,
{
    type Output = N;
    fn index(&self, index: NodeIndex<Ix>) -> &N {
        &self.nodes[index.index()].item
    }
}
impl<N, E, Ix> IndexMut<NodeIndex<Ix>> for Graph<N, E, Ix>
where
    N: Clone,
    Ix: Eq + Hash + Clone + PartialOrd + Ord,
    E: Clone,
{
    fn index_mut(&mut self, index: NodeIndex<Ix>) -> &mut N {
        &mut self.nodes[index.index()].item
    }
}

// impl<N, E, Ix> IndexMut<EdgeIndex<Ix>> for Graph<N, E, Ix>
// where
//     E: Clone,
//     Ix: Eq + Hash + Clone + PartialOrd + Ord,
//     N: Clone,
// {
//     fn index_mut(&mut self, index: EdgeIndex<Ix>) -> &mut E {
//         &mut self.edges.get()[index].item
//     }
// }

// @ test ────────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(unused)]
mod graph_test_mod {
    use crate::graph::Graph;
    use crate::im::Vector;
    use emg_state::use_state;
    use indexmap::IndexSet;
    use std::{iter::FromIterator, path::Path};
    use tracing::debug;

    use std::clone::Clone;

    use crate::graph::{edge_index, node_index, Node};
    // ────────────────────────────────────────────────────────────────────────────────

    use std::cell::RefCell;
    use std::collections::HashMap;
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
                    // && !metadata.target().contains("winit event")
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
        //     .with_filter(EnvFilter::new("[event_matching...]=debug"));

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
    fn test() {
        let ff: RefCell<_> = RefCell::new(HashMap::new());

        ff.borrow_mut().insert("africa", 92388);
        ff.borrow_mut().insert("kyoto", 11837);
        ff.borrow_mut().insert("piccadilly", 11826);
        ff.borrow_mut().insert("marbles", 38);
        let _xx = ff.clone();
        // println!("{:?}", ff);

        let shared_map: Rc<RefCell<_>> = Rc::new(RefCell::new(HashMap::new()));
        let cloned = Rc::clone(&shared_map);
        let own = cloned.to_owned();
        shared_map.borrow_mut().insert("africa", 92388);
        shared_map.borrow_mut().insert("kyoto", 11837);
        shared_map.borrow_mut().insert("piccadilly", 11826);
        shared_map.borrow_mut().insert("marbles", 38);
        assert_eq!(cloned, shared_map);
        assert_eq!(own, shared_map);
        println!("{:?}", shared_map);
        println!("{:?}", cloned);
        println!("{:?}", own);
        println!("{:?}", own);
    }

    #[test]
    fn node_create() {
        let a_node: Node<String, String> = Node::new(
            String::from("is node item"),
            use_state(|| {
                [edge_index(String::from("3"), String::from("1"))]
                    .into_iter()
                    .collect()
            }),
            use_state(|| {
                [edge_index(String::from("1"), String::from("2"))]
                    .into_iter()
                    .collect()
            }),
        );
        let str_node: Node<String, String> = Node::new(
            String::from("is node item"),
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
                String::from("ww_item"),
                use_state(||
                    [edge_index(String::from("xx"), String::from("ww"))]
                        .into_iter()
                        .collect(),
                ),
                use_state(||
                    [edge_index(String::from("ww"), String::from("xx"))]
                        .into_iter()
                        .collect(),
                ),
            );
            // @ graph ─────────────────────────────────────────────────────────────────

            let mut g1: Graph<String, &'static str, String> = Graph::empty();

            // @ add node ─────────────────────────────────────────────────────────────────

            // let ww_nix = g1.insert_node_in_topo(String::from("ww"), String::from("ww_item"));
            g1.nodes.insert(String::from("ww"), ww_node.clone());
            let ww_nix = node_index("ww");
            assert_eq!(
                String::from("ww_item"),
                g1.get_node_item(&ww_nix).unwrap().clone()
            );

            let xx_nix = g1.insert_node_in_topo(String::from("xx"), String::from("xx_item"));

            // @ add edge ─────────────────────────────────────────────────────────────────

            let op_eix1 = g1
                .unused_insert_update_edge_in_topo(&ww_nix, &xx_nix, "ww->xx:item")
                .unwrap();
            assert_eq!(op_eix1, edge_index(String::from("ww"), String::from("xx")));
            let op_eix2 = g1
                .unused_insert_update_edge_in_topo(&xx_nix, &ww_nix, "xx->ww:item")
                .unwrap();
            assert_eq!(op_eix2, edge_index(String::from("xx"), String::from("ww")));
            // @ test ─────────────────────────────────────────────────────────────────
            println!("{:#?}", g1);

            // * match node item

            assert_eq!(
                ww_node.item,
                g1.nodes.get(&String::from("ww")).unwrap().item
            );
            assert_eq!(
                ww_node.incoming_len,
                g1.nodes.get(&String::from("ww")).unwrap().incoming_len
            );
            assert_eq!(
                ww_node.outgoing_len,
                g1.nodes.get(&String::from("ww")).unwrap().outgoing_len
            );
            // * match node
            assert_eq!(&ww_node, g1.nodes.get(&String::from("ww")).unwrap());
            assert_eq!(2, g1.node_count());
            assert_eq!(2, g1.edges_count());
            assert_eq!("ww->xx:item", *g1.edge_item(&op_eix1));
            assert_eq!("ww->xx:item", *g1.edge_item(&op_eix1));
            assert_eq!("xx->ww:item", *g1.edge_item(&op_eix2));
            assert_eq!((&ww_nix, &xx_nix), op_eix1.get_nix_s_unwrap());

            // @ remove edge ─────────────────────────────────────────────────────────────────
            g1.remove_edge(&op_eix1);


            assert_eq!(1, g1.edges_count());

            let ww_node_rm_edge = Node::new(
                String::from("ww_item"),
                use_state(||
                    [edge_index(String::from("xx"), String::from("ww"))]
                        .into_iter()
                        .collect(),
                ),
                use_state(||IndexSet::default()),
            );

            let xx_ww_edge = ww_node_rm_edge
                .incoming()
                .get_with(|x| x.first().unwrap().clone());
            let g1_xx_ww_edge = g1
                .nodes
                .get(&String::from("ww"))
                .unwrap()
                .incoming()
                .get_with(|x| x.first().unwrap().clone());
            // 依然保有 xx->ww 边
            assert_eq!(xx_ww_edge, g1_xx_ww_edge);

            // @ remove node ─────────────────────────────────────────────────────────────────
            insta::assert_display_snapshot!("graph_a",g1);
            g1.remove_node(node_index(String::from("ww")));
            insta::assert_display_snapshot!("graph_a_removed",g1);


            assert_eq!(1, g1.node_count());

            let mut g2: Graph<String, &'static str, String> = Graph::empty();

            g2.insert_node_in_topo(String::from("xx"), String::from("xx_item"));

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
        let mut g: Graph<&str, &str, &str> = Graph::empty();
        let l1_nix = g.insert_node_in_topo("1", "1");

        let n = &mut g[l1_nix];

        println!("===================");
        *n = "edited";
        println!("{:?}", n);

        println!("{:#?}", &g);
        assert_eq!("edited", g[l1_nix]);
    }
    #[test]
    #[allow(unused)]
    fn from_nodes_and_edges() {
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
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
        let ff = "caaa";
        let xx = &ff;
        let cc = *xx;
        let dd = ff;
        println!("clone: {:?}", ff.as_ptr());
        println!("clone: {:?}", ff.clone().as_ptr());
        println!("clone: {:?}", xx.as_ptr());
        println!("clone: {:?}", cc.as_ptr());
        println!("clone: {:?}", dd.as_ptr());

        assert_eq!(cc.as_ptr(), ff.as_ptr());
        assert_eq!(dd.as_ptr(), ff.as_ptr());

        let mut my_speed = String::from("99old");
        let my_speed_ptr = &mut my_speed;
        *my_speed_ptr = String::from("99xx");
        println!("{:?}", my_speed_ptr);
        println!("{:?} ", my_speed);
        // ─────────────────────────────────────────────────────────────────
        let mut input = Vector::from_iter(0..10);
        let vec = input.clone();
        input.push_back(999);

        println!("{:?}", input);
        println!("{:?}", vec);
    }
}
