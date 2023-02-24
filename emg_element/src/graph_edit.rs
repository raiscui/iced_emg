/*
 * @Author: Rais
 * @Date: 2023-01-19 17:43:32
 * @LastEditTime: 2023-02-23 23:33:20
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
    type Message;

    fn edit<M: Mode<Self::Message>>(&self, mode: M) -> M::Interface<'_>;
}
impl<T> GraphEdit for T
where
    T: GraphEditManyMethod,
{
    type Message = T::Message;
    fn edit<M: Mode<Self::Message>>(&self, mode: M) -> M::Interface<'_> {
        mode.interface(self)
    }
}

pub trait GraphEditManyMethod {
    type Message;

    //实例连源枝移动( 某 path edge 原 edge 更改 source node) ,枝上其他node 不动(clone edge?)
    fn edge_plug_edit(&self, who: &EdgeIndex, dir: Direction, to: IdStr) -> Result<(), Error>;

    //实例嫁接(实例不连源枝移动 , 某 path node 原 edge 断开, xin edge 接上)
    fn edge_path_node_change_edge(&mut self);
    fn insert_node_in_topo(&self, tree_element: &'_ GTreeBuilderElement<Self::Message>, to: IdStr);
}

// Editor ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct GraphEditor<Message>(pub(crate) Rc<RefCell<GraphType<Message>>>)
where
    Message: 'static;

impl<Message> GraphEditor<Message>
where
    Message: 'static,
{
    pub fn graph(&self) -> std::cell::Ref<GraphType<Message>> {
        self.0.borrow()
    }
} //for Debug derive

// impl<Message> Deref for GraphEditor<Message> {
//     type Target = Rc<RefCell<GraphType<Message>>>;

//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

pub trait Mode<Message> {
    type Interface<'a>
    where
        Self: 'a,
        Message: 'a;

    fn interface(self, g: &dyn GraphEditManyMethod<Message = Message>) -> Self::Interface<'_>;
}

// ─────────────────────────────────────────────────────────────────────────────
