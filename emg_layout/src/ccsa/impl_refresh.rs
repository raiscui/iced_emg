/*
 * @Author: Rais
 * @Date: 2022-07-12 18:16:47
 * @LastEditTime: 2022-07-20 17:40:26
 * @LastEditors: Rais
 * @Description:
 */

use emg_core::Vector;
use emg_refresh::{RefreshFor, RefreshUseNoWarper, RefreshWhoNoWarper};
use emg_state::CloneStateVar;
use tracing::warn;

use crate::EmgEdgeItem;

use super::{GeneralVar, ScopeViewVariable, CCSS};

impl<Ix> RefreshFor<EmgEdgeItem<Ix>> for (Vector<CCSS>, Vector<ScopeViewVariable>)
where
    Ix: Clone + std::hash::Hash + Eq + Ord + 'static + Default,
    EmgEdgeItem<Ix>: RefreshWhoNoWarper,
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

impl RefreshUseNoWarper for GeneralVar {}

impl<Ix> RefreshFor<EmgEdgeItem<Ix>> for GeneralVar
where
    Ix: Clone + std::hash::Hash + Eq + Ord + 'static + Default,
    EmgEdgeItem<Ix>: RefreshWhoNoWarper,
    GeneralVar: RefreshUseNoWarper,
{
    #[track_caller]
    fn refresh_for(&self, who: &mut EmgEdgeItem<Ix>) {
        let GeneralVar(
            name,
            ScopeViewVariable {
                scope,
                view,
                variable,
            },
        ) = self;
        match (scope, view, variable) {
            (None, None, None) => todo!(),
            (None, None, Some(_)) => todo!(),
            (None, Some(view), None) => match view {
                super::NameChars::Id(_) => todo!(),
                super::NameChars::Class(_) => todo!(),
                super::NameChars::Element(_) => todo!(),
                super::NameChars::Virtual(_) => todo!(),
                super::NameChars::Number(n) => who.layout.cassowary_generals.update(|x| {
                    x.insert(name.clone(), n.into_inner());
                    // warn!("cassowary_generals update \n{:?}", &x);
                }),
                super::NameChars::Next(_) => todo!(),
                super::NameChars::Last(_) => todo!(),
                super::NameChars::First(_) => todo!(),
            },
            (None, Some(_), Some(_)) => todo!(),
            (Some(_), None, None) => todo!(),
            (Some(_), None, Some(_)) => todo!(),
            (Some(_), Some(_), None) => todo!(),
            (Some(_), Some(_), Some(_)) => todo!(),
        };
    }
}
