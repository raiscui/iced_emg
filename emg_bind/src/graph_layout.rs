/*
 * @Author: Rais
 * @Date: 2021-05-06 14:13:04
 * @LastEditTime: 2022-06-15 21:22:48
 * @LastEditors: Rais
 * @Description:
 */

use emg::EdgeIndex;
use emg_layout::GenericSizeAnchor;

// use crate::GraphType;
use std::hash::Hash;

pub trait GraphMethods {
    type Ix;
    fn edge_item_set_size(
        &self,
        e: &EdgeIndex<Self::Ix>,
        w: impl Into<GenericSizeAnchor>,
        h: impl Into<GenericSizeAnchor>,
    );
}
impl<Message, Ix> GraphMethods for crate::GraphType<Message, Ix>
where
    Ix: Clone + Hash + Eq + std::fmt::Debug + Ord + Default + Send,
    // E: Clone + std::fmt::Debug,
    Message: 'static + Clone + std::cmp::PartialEq,
{
    type Ix = Ix;
    fn edge_item_set_size(
        &self,
        e: &EdgeIndex<Self::Ix>,
        w: impl Into<GenericSizeAnchor>,
        h: impl Into<GenericSizeAnchor>,
    ) {
        self.edge_item(e).store_set_size(&self.store(), w, h);
    }
}

impl<Message, Ix> GraphMethods for crate::g_node::node_item_rc::GraphType<Message, Ix>
where
    Ix: Clone + Hash + Eq + std::fmt::Debug + Ord + Default + Send,
    // E: Clone + std::fmt::Debug,
    Message: 'static + Clone + std::cmp::PartialEq,
{
    type Ix = Ix;
    fn edge_item_set_size(
        &self,
        e: &EdgeIndex<Self::Ix>,
        w: impl Into<GenericSizeAnchor>,
        h: impl Into<GenericSizeAnchor>,
    ) {
        self.edge_item(e).store_set_size(&self.store(), w, h);
    }
}
