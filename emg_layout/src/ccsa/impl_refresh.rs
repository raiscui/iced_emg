/*
 * @Author: Rais
 * @Date: 2022-07-12 18:16:47
 * @LastEditTime: 2022-07-12 18:33:11
 * @LastEditors: Rais
 * @Description:
 */

use emg_core::{vector, Vector};
use emg_refresh::RefreshFor;
use emg_state::CloneStateVar;

use crate::EmgEdgeItem;

use super::CCSS;

impl<Ix> RefreshFor<EmgEdgeItem<Ix>> for Vector<CCSS>
where
    Ix: Clone + std::hash::Hash + Eq + Ord + 'static + Default,
{
    #[track_caller]
    fn refresh_for(&self, who: &mut EmgEdgeItem<Ix>) {
        let added_vec_ccss = self.clone();
        let vec_ccss = who.layout.cassowary.get_inner_anchor();
        let new_vec_ccss = vec_ccss.map(move |old| {
            let mut new = old.clone();
            new.append(added_vec_ccss.clone());
            new
        });

        who.layout.cassowary.set(new_vec_ccss);
    }
}
