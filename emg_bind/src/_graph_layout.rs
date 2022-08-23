/*
 * @Author: Rais
 * @Date: 2021-05-06 14:13:04
 * @LastEditTime: 2022-06-18 23:48:17
 * @LastEditors: Rais
 * @Description:
 */

use emg::{EdgeIndex, Graph};
use emg_layout::{EmgEdgeItem, GenericSizeAnchor};

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
impl<N, Ix> GraphMethods for Graph<N, EmgEdgeItem<Ix>, Ix>
where
    N: Clone,
    Ix: PartialOrd + 'static,
    Ix: Clone + Hash + Eq + std::fmt::Debug + Ord + Default,
    // E: Clone + std::fmt::Debug,
    // Message: 'static + Clone + std::cmp::PartialEq,
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
