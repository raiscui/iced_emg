/*
 * @Author: Rais
 * @Date: 2023-01-19 17:43:32
 * @LastEditTime: 2023-02-23 12:35:32
 * @LastEditors: Rais
 * @Description:
 */

mod impls;
mod mode;

// ─────────────────────────────────────────────────────────────────────────────

use std::{cell::RefCell, ops::Deref, rc::Rc};

use crate::{error::Error, GTreeBuilderElement, GraphType};
use emg::{Direction, EdgeIndex};
use emg_common::IdStr;
pub use impls::*;
pub use mode::*;

// ─────────────────────────────────────────────────────────────────────────────
//NOTE: not object safe
pub trait GraphEdit {
    type Ix;
    type Message;

    fn edit<M: Mode<Self::Message, Ix = Self::Ix>>(&self, mode: M) -> M::Interface<'_>;
}
impl<T> GraphEdit for T
where
    T: GraphEditManyMethod,
{
    type Ix = T::Ix;
    type Message = T::Message;
    fn edit<M: Mode<Self::Message, Ix = Self::Ix>>(&self, mode: M) -> M::Interface<'_> {
        mode.interface(self)
    }
}

pub trait GraphEditManyMethod {
    type Ix;
    type Message;

    //实例连源枝移动( 某 path edge 原 edge 更改 source node) ,枝上其他node 不动(clone edge?)
    fn edge_plug_edit(
        &self,
        who: &EdgeIndex<Self::Ix>,
        dir: Direction,
        to: Self::Ix,
    ) -> Result<(), Error>;

    //实例嫁接(实例不连源枝移动 , 某 path node 原 edge 断开, xin edge 接上)
    fn edge_path_node_change_edge(&mut self);
    fn insert_node_in_topo(&self, tree_element: &'_ GTreeBuilderElement<Self::Message>);
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

pub trait Mode<Message> {
    type Ix;

    type Interface<'a>
    where
        Self: 'a,
        Message: 'a;

    fn interface(
        self,
        g: &dyn GraphEditManyMethod<Message = Message, Ix = Self::Ix>,
    ) -> Self::Interface<'_>;
}

// ─────────────────────────────────────────────────────────────────────────────
