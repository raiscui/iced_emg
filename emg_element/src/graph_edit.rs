/*
 * @Author: Rais
 * @Date: 2023-01-19 17:43:32
 * @LastEditTime: 2023-02-21 00:12:13
 * @LastEditors: Rais
 * @Description:
 */

mod impls;
use std::{cell::RefCell, marker::PhantomData, ops::Deref, rc::Rc};

use emg::{Direction, EdgeIndex};
use emg_common::IdStr;
pub use impls::*;

use crate::{error::Error, GraphType};

// ─────────────────────────────────────────────────────────────────────────────

pub trait GraphEdit {
    type Ix;
    fn edit<M: Mode>(&self) -> M::Interface<'_, Self::Ix>;
}
impl<'a, T> GraphEdit for T
where
    T: GraphEditManyMethod,
{
    type Ix = T::Ix;
    fn edit<M: Mode>(&self) -> M::Interface<'_, Self::Ix> {
        M::interface(self)
    }
}

pub trait GraphEditManyMethod {
    type Ix;
    //实例连源枝移动( 某 path edge 原 edge 更改 source node) ,枝上其他node 不动(clone edge?)
    fn edge_plug_edit(
        &self,
        who: &EdgeIndex<Self::Ix>,
        dir: Direction,
        to: Self::Ix,
    ) -> Result<(), Error>;

    //实例嫁接(实例不连源枝移动 , 某 path node 原 edge 断开, xin edge 接上)
    fn edge_path_node_change_edge(&mut self);
}

// Editor ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct GraphEditor<Message, Ix = IdStr>(pub(crate) Rc<RefCell<GraphType<Message, Ix>>>)
where
    Ix: std::hash::Hash + Clone + Ord + Default + 'static,
    Message: 'static; //for Debug derive

impl<Message, Ix> Deref for GraphEditor<Message, Ix>
where
    Ix: std::hash::Hash + Clone + Ord + Default + 'static,
{
    type Target = Rc<RefCell<GraphType<Message, Ix>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// EdgeMode ─────────────────────────────────────────────────────────────────────────────
// I :实例?
pub struct EdgeMode<I = ()>(PhantomData<I>);

pub trait Mode {
    type Interface<'a, Ix>
    where
        Ix: 'a;

    fn interface<'a, Ix, G>(g: &'a G) -> Self::Interface<'a, Ix>
    where
        G: GraphEditManyMethod<Ix = Ix>;
}

impl Mode for EdgeMode {
    type Interface<'a, Ix> = EdittingGraphEdge<'a, Ix, Self> where Ix:'a;

    fn interface<'a, Ix, G>(g: &'a G) -> Self::Interface<'a, Ix>
    where
        G: GraphEditManyMethod<Ix = Ix>,
    {
        let inner = g;
        let phantom_data = PhantomData;
        EdittingGraphEdge {
            inner,
            phantom_data,
        }
    }
}

pub struct EdittingGraphEdge<'a, Ix, M> {
    inner: &'a dyn GraphEditManyMethod<Ix = Ix>,
    phantom_data: PhantomData<M>,
}

impl<'a, Ix, M> EdittingGraphEdge<'a, Ix, M> {
    pub fn moving(
        &self,
        who: impl Into<EdgeIndex<Ix>>,
        dir: Direction,
        to: impl Into<Ix>,
    ) -> Result<(), Error> {
        self.inner.edge_plug_edit(&who.into(), dir, to.into())
    }

    //TODO fn edit to edit other eg. node
}

// ─────────────────────────────────────────────────────────────────────────────
