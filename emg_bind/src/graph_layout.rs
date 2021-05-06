/*
 * @Author: Rais
 * @Date: 2021-05-06 14:13:04
 * @LastEditTime: 2021-05-06 17:42:40
 * @LastEditors: Rais
 * @Description:
 */

use emg::EdgeIndex;
use emg_layout::{GenericSize, GenericWH};

use crate::GraphType;
use std::hash::Hash;

pub trait GraphMethods<Ix> {
    fn edge_item_set_size(
        &self,
        e: &EdgeIndex<Ix>,
        w: impl Into<GenericSize>,
        h: impl Into<GenericSize>,
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
        w: impl Into<GenericSize>,
        h: impl Into<GenericSize>,
    ) {
        self.edge_item(e)
            .store_set_size(&self.store(), GenericWH::new(w, h));
    }
}
