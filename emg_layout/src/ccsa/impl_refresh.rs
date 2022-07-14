/*
 * @Author: Rais
 * @Date: 2022-07-12 18:16:47
 * @LastEditTime: 2022-07-13 15:00:28
 * @LastEditors: Rais
 * @Description:
 */

use emg_core::Vector;
use emg_refresh::RefreshFor;
use emg_state::CloneStateVar;

use crate::EmgEdgeItem;

use super::{ScopeViewVariable, CCSS};

impl<Ix> RefreshFor<EmgEdgeItem<Ix>> for (Vector<CCSS>, Vector<ScopeViewVariable>)
where
    Ix: Clone + std::hash::Hash + Eq + Ord + 'static + Default,
{
    #[track_caller]
    fn refresh_for(&self, who: &mut EmgEdgeItem<Ix>) {
        let (added_vec_ccss, added_vec_selector) = self.clone();
        let vec_ccss = who.layout.cassowary_constants.get_inner_anchor();
        let new_vec_ccss = vec_ccss.map(move |old| {
            let mut new = old.clone();
            new.append(added_vec_ccss.clone());
            new
        });

        who.layout.cassowary_constants.set(new_vec_ccss);

        who.layout
            .cassowary_selectors
            .update(|selectors| selectors.append(added_vec_selector))
    }
}
