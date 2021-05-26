/*
 * @Author: Rais
 * @Date: 2021-05-06 14:13:04
 * @LastEditTime: 2021-05-26 15:08:26
 * @LastEditors: Rais
 * @Description:
 */

use emg::EdgeIndex;
use emg_layout::GenericSizeAnchor;

use crate::GraphType;
use std::hash::Hash;

pub trait GraphMethods<Ix> {
    fn edge_item_set_size(
        &self,
        e: &EdgeIndex<Ix>,
        w: impl Into<GenericSizeAnchor>,
        h: impl Into<GenericSizeAnchor>,
    );
}
impl<'a, Message, Ix> GraphMethods<Ix> for GraphType<'a, Message, Ix>
where
    Ix: Clone + Hash + Eq + std::fmt::Debug + Ord + Default,
    // E: Clone + std::fmt::Debug,
    Message: 'static + Clone,
{
    fn edge_item_set_size(
        &self,
        e: &EdgeIndex<Ix>,
        w: impl Into<GenericSizeAnchor>,
        h: impl Into<GenericSizeAnchor>,
    ) {
        self.edge_item(e).store_set_size(&self.store(), w, h);
    }
}
