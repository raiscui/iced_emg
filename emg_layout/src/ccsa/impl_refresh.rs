/*
 * @Author: Rais
 * @Date: 2022-07-12 18:16:47
 * @LastEditTime: 2023-01-14 01:02:39
 * @LastEditors: Rais
 * @Description:
 */

use emg_common::Vector;
use emg_shaping::{Shaping, ShapingUseNoWarper, ShapingWhoNoWarper};
use emg_state::CloneStateVar;

use crate::EmgEdgeItem;

use super::{CassowaryVar, GeneralVar, NameChars, ScopeViewVariable, Virtual, CCSS};

impl<Ix> Shaping<EmgEdgeItem<Ix>> for (Vector<CCSS>, Vector<ScopeViewVariable>)
where
    Ix: Clone + std::hash::Hash + Eq + Ord + 'static + Default,
    EmgEdgeItem<Ix>: ShapingWhoNoWarper,
{
    #[track_caller]
    fn shaping(&self, who: &mut EmgEdgeItem<Ix>) {
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
            .update(|selectors| selectors.append(added_vec_selector));
    }
}

impl ShapingUseNoWarper for GeneralVar {}

impl<Ix> Shaping<EmgEdgeItem<Ix>> for GeneralVar
where
    Ix: Clone + std::hash::Hash + Eq + Ord + 'static + Default,
    EmgEdgeItem<Ix>: ShapingWhoNoWarper,
    Self: ShapingUseNoWarper,
{
    #[allow(clippy::match_same_arms)]
    #[track_caller]
    fn shaping(&self, who: &mut EmgEdgeItem<Ix>) {
        let Self(
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
            (None, Some(view_), None) => match view_ {
                NameChars::Id(_) => todo!(),
                NameChars::Class(_) => todo!(),
                NameChars::Element(_) => todo!(),
                NameChars::Virtual(_) => todo!(),
                NameChars::Number(n) => who.layout.cassowary_generals.update(|x| {
                    x.insert(name.clone(), n.into_inner());
                    // warn!("cassowary_generals update \n{:?}", &x);
                }),
                NameChars::Next(_) => todo!(),
                NameChars::Last(_) => todo!(),
                NameChars::First(_) => todo!(),
            },
            (None, Some(_), Some(_)) => todo!(),
            (Some(_), None, None) => todo!(),
            (Some(_), None, Some(_)) => todo!(),
            (Some(_), Some(_), None) => todo!(),
            (Some(_), Some(_), Some(_)) => todo!(),
        };
    }
}

impl ShapingUseNoWarper for Virtual {}

impl<Ix> Shaping<EmgEdgeItem<Ix>> for Virtual
where
    Ix: Clone + std::hash::Hash + Eq + Ord + 'static + Default,
    EmgEdgeItem<Ix>: ShapingWhoNoWarper,
    Self: ShapingUseNoWarper,
{
    #[allow(clippy::match_same_arms)]
    #[track_caller]
    fn shaping(&self, who: &mut EmgEdgeItem<Ix>) {
        let virtual_name = self.name();
        let (gvs_match_props, (top_constants, constants), not_match) = self.process();

        //TODO 更高效的 一次全更新.

        who.layout
            .cassowary_generals
            .update(|x| x.insert_constants(virtual_name.clone(), top_constants, constants));

        for (_, opt_gv) in not_match {
            opt_gv.unwrap().shaping(who);
        }

        for (prop, (top_var, var, opt_gv)) in gvs_match_props {
            if let Some(GeneralVar(
                _,
                ScopeViewVariable {
                    scope,
                    view,
                    variable,
                },
            )) = opt_gv
            {
                match (scope, view, variable) {
                    (None, None, None) => todo!(),
                    (None, None, Some(_)) => todo!(),
                    (None, Some(view_), None) => match view_ {
                        NameChars::Id(_) => todo!(),
                        NameChars::Class(_) => todo!(),
                        NameChars::Element(_) => todo!(),
                        NameChars::Virtual(_) => todo!(),
                        NameChars::Number(n) => {
                            who.layout.cassowary_generals.update(|x| {
                                x.insert_with_var(
                                    virtual_name.clone() + "." + prop,
                                    top_var,
                                    var,
                                    n.into_inner(),
                                );
                            });
                        }
                        NameChars::Next(_) => todo!(),
                        NameChars::Last(_) => todo!(),
                        NameChars::First(_) => todo!(),
                    },
                    (None, Some(_), Some(_)) => todo!(),
                    (Some(_), None, None) => todo!(),
                    (Some(_), None, Some(_)) => todo!(),
                    (Some(_), Some(_), None) => todo!(),
                    (Some(_), Some(_), Some(_)) => todo!(),
                };
            } else {
                who.layout.cassowary_generals.update(|x| {
                    x.insert_only_var(virtual_name.clone() + "." + prop, top_var, var);
                });
            }
        }
    }
}

impl ShapingUseNoWarper for CassowaryVar {}

impl<Ix> Shaping<EmgEdgeItem<Ix>> for CassowaryVar
where
    Ix: Clone + std::hash::Hash + Eq + Ord + 'static + Default,
    EmgEdgeItem<Ix>: ShapingWhoNoWarper,
    Self: ShapingUseNoWarper,
{
    #[allow(clippy::match_same_arms)]
    #[track_caller]
    fn shaping(&self, who: &mut EmgEdgeItem<Ix>) {
        match self {
            Self::General(gv) => gv.shaping(who),
            Self::Virtual(vv) => vv.shaping(who),
        };
    }
}
