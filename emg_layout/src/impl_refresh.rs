/*
 * @Author: Rais
 * @Date: 2021-03-29 19:22:19
 * @LastEditTime: 2022-01-07 13:08:30
 * @LastEditors: Rais
 * @Description:
 */

use emg_refresh::{RefreshFor, RefreshForUse, RefreshUseNoWarper, RefreshWhoNoWarper};
use std::{any::Any, panic::Location, rc::Rc};

use emg_state::{CloneStateVar, StateAnchor, StateVar};
pub use seed_styles as styles;
use styles::{CssHeight, CssValueTrait, CssWidth, UpdateStyle};
use tracing::{debug, trace, trace_span, warn};

use crate::{
    add_values::{AlignX, AlignY, OriginX, OriginY},
    animation::AnimationE,
    Css, EPath, EmgEdgeItem,
};

// ────────────────────────────────────────────────────────────────────────────────

//TODO lifetime
impl<Ix> RefreshWhoNoWarper for EmgEdgeItem<Ix> where
    Ix: Clone + std::hash::Hash + Eq + Ord + 'static + Default
{
}

impl<T> RefreshUseNoWarper for Css<T> where T: CssValueTrait + Clone + 'static {}

impl<Ix> RefreshFor<EmgEdgeItem<Ix>> for Box<dyn RefreshFor<EmgEdgeItem<Ix>>>
where
    Ix: Clone + std::hash::Hash + Eq + Ord + 'static + Default,
    EmgEdgeItem<Ix>: RefreshWhoNoWarper,
{
    #[track_caller]
    fn refresh_for(&self, who: &mut EmgEdgeItem<Ix>) {
        let _g = trace_span!(
            "!!!!!!!!!!!!!!-> RefreshFor<EdgeItem> for Box<(dyn RefreshFor<EdgeItem> + 'static)>"
        )
        .entered();
        // let ii = i.as_ref();
        who.refresh_for_use(self.as_ref());
    }
}
impl<Ix> RefreshFor<EmgEdgeItem<Ix>> for Rc<dyn RefreshFor<EmgEdgeItem<Ix>>>
where
    Ix: Clone + std::hash::Hash + Eq + Ord + 'static + Default,
    EmgEdgeItem<Ix>: RefreshWhoNoWarper,
{
    #[track_caller]
    fn refresh_for(&self, who: &mut EmgEdgeItem<Ix>) {
        let _g = trace_span!(
            "!!!!!!!!!!!!!!-> RefreshFor<EdgeItem> for Box<(dyn RefreshFor<EdgeItem> + 'static)>"
        )
        .entered();
        // let ii = i.as_ref();
        who.refresh_for_use(self.as_ref());
    }
}

impl<Ix, Use> RefreshFor<EmgEdgeItem<Ix>> for StateVar<Use>
where
    Ix: Clone + std::hash::Hash + Eq + Ord + 'static + Default,
    EmgEdgeItem<Ix>: RefreshWhoNoWarper,
    Use: RefreshUseNoWarper + RefreshFor<EmgEdgeItem<Ix>> + Clone + 'static,
{
    // #[allow(clippy::redundant_closure_for_method_calls)]
    default fn refresh_for(&self, who: &mut EmgEdgeItem<Ix>) {
        let rc_v = self.get_var_with(emg_state::Var::get);
        warn!("Edge  Refresh use StateVar current value");
        who.refresh_for_use(&*rc_v);
    }
}
// ────────────────────────────────────────────────────────────────────────────────
// ────────────────────────────────────────────────────────────────────────────────
impl<Ix> RefreshFor<EmgEdgeItem<Ix>> for StateVar<CssWidth>
where
    Ix: Clone + std::hash::Hash + Eq + Ord + 'static + Default,
    EmgEdgeItem<Ix>: RefreshWhoNoWarper,
{
    #[allow(clippy::redundant_closure_for_method_calls)]
    fn refresh_for(&self, who: &mut EmgEdgeItem<Ix>) {
        warn!("Edge  Refresh use StateVar<CssWidth>");

        who.layout.w.set(self.watch().into());

        // who.refresh_use(&*rc_var);
    }
}
impl<Ix> RefreshFor<EmgEdgeItem<Ix>> for StateAnchor<CssWidth>
where
    Ix: Clone + std::hash::Hash + Eq + Ord + 'static + Default,
    EmgEdgeItem<Ix>: RefreshWhoNoWarper,
    // Use: RefreshUseNoWarper + RefreshFor<EmgEdgeItem<Ix>> + Clone + 'static,
{
    #[allow(clippy::redundant_closure_for_method_calls)]
    fn refresh_for(&self, who: &mut EmgEdgeItem<Ix>) {
        warn!("Edge  Refresh use StateAnchor<CssWidth>");

        who.layout.w.set(self.clone().into());

        // who.refresh_use(&*rc_var);
    }
}
impl<Ix> RefreshFor<EmgEdgeItem<Ix>> for StateVar<CssHeight>
where
    Ix: Clone + std::hash::Hash + Eq + Ord + 'static + Default,
    EmgEdgeItem<Ix>: RefreshWhoNoWarper,
    // Use: RefreshUseNoWarper + RefreshFor<EmgEdgeItem<Ix>> + Clone + 'static,
{
    #[allow(clippy::redundant_closure_for_method_calls)]
    fn refresh_for(&self, who: &mut EmgEdgeItem<Ix>) {
        warn!("Edge  Refresh use StateVar<CssHeight>");

        who.layout.h.set(self.watch().into());

        // who.refresh_use(&*rc_var);
    }
}
impl<Ix> RefreshFor<EmgEdgeItem<Ix>> for StateAnchor<CssHeight>
where
    Ix: Clone + std::hash::Hash + Eq + Ord + 'static + Default,
    EmgEdgeItem<Ix>: RefreshWhoNoWarper,
    // Use: RefreshUseNoWarper + RefreshFor<EmgEdgeItem<Ix>> + Clone + 'static,
{
    #[allow(clippy::redundant_closure_for_method_calls)]
    fn refresh_for(&self, who: &mut EmgEdgeItem<Ix>) {
        warn!("Edge  Refresh use StateAnchor<CssHeight>");

        who.layout.h.set(self.clone().into());

        // who.refresh_use(&*rc_var);
    }
}

// ────────────────────────────────────────────────────────────────────────────────
// ────────────────────────────────────────────────────────────────────────────────

// impl<Ix> RefreshFor<EmgEdgeItem<Ix>> for Vec<Box<(dyn RefreshFor<EmgEdgeItem<Ix>> + 'static)>>
// where
//     Ix: Clone + std::hash::Hash + Eq + Ord + 'static + Default,
// {
//     #[track_caller]
//     fn refresh_for(&self, who: &mut EmgEdgeItem<Ix>) {
//         for i in self {
//             let _g = trace_span!(
//                 "-> RefreshFor<EdgeItem> for Vec<Box<(dyn RefreshFor<EdgeItem> + 'static)>>"
//             )
//             .entered();
//             // let ii = i.as_ref();
//             who.refresh_use(i.as_ref());
//         }
//     }
// }
// impl RefreshFor<EdgeData> for Vec<Box<(dyn RefreshFor<EdgeData> + 'static)>> {
//     #[track_caller]
//     fn refresh_for(&self, _who: &mut EdgeData) {
//         panic!("!!!!!!");
//         // for i in self {
//         //     // let ii = i.as_ref();
//         //     who.refresh_use(i.as_ref());
//         // }
//     }
// }
//TODO 做 不是refresh 版本的
#[track_caller]
fn css_refresh_edgedata<Use, Ix>(css: &Css<Use>, ed: &mut EmgEdgeItem<Ix>)
where
    Use: CssValueTrait + std::clone::Clone,
    Ix: Clone + std::hash::Hash + Eq + Ord + 'static + Default,
{
    let _g = trace_span!("-> css_refresh_edgedata").entered();

    let any = &css.0 as &dyn Any;
    if let Some(css_width) = any.downcast_ref::<CssWidth>() {
        debug!("dyn match CssWidth {}", &css_width);
        ed.refresh_for_use(css_width);
        return;
    }

    if let Some(css_height) = any.downcast_ref::<CssHeight>() {
        debug!("dyn match CssHeight {}", &css_height);
        ed.refresh_for_use(css_height);
        return;
    }

    {
        // @ 不唯一, 多次会重复 ─────────────────────────────────────────────────────────────────

        ed.other_styles.set_with(|s| {
            let mut tmp_s = s.clone();
            let t = css.0.clone();

            tmp_s
                .updated_at
                .push_back(format!("{}", Location::caller()));

            t.update_style(&mut tmp_s);
            tmp_s
        });
    }
}
// ────────────────────────────────────────────────────────────────────────────────

impl<Use, Ix> RefreshFor<EmgEdgeItem<Ix>> for Css<Use>
where
    Use: CssValueTrait + std::clone::Clone,
    Ix: Clone + std::hash::Hash + Eq + Ord + 'static + Default,
{
    #[track_caller]
    fn refresh_for(&self, who: &mut EmgEdgeItem<Ix>) {
        let _g = trace_span!("-> RefreshFor<EdgeItem> for Css<Use>").entered();

        css_refresh_edgedata(self, who);
    }
}

// ────────────────────────────────────────────────────────────────────────────────

impl<Ix> RefreshFor<EmgEdgeItem<Ix>> for CssWidth
where
    Ix: Clone + std::hash::Hash + Eq + Ord + 'static + Default,
{
    #[track_caller]
    fn refresh_for(&self, who: &mut EmgEdgeItem<Ix>) {
        let _g = trace_span!("-> RefreshFor<EmgEdgeItem> for CssWidth").entered();

        who.layout.w.set(self.clone().into());
    }
}

impl<Ix> RefreshFor<EmgEdgeItem<Ix>> for CssHeight
where
    Ix: Clone + std::hash::Hash + Eq + Ord + 'static + Default,
{
    #[track_caller]
    fn refresh_for(&self, who: &mut EmgEdgeItem<Ix>) {
        let _g = trace_span!("-> RefreshFor<EmgEdgeItem> for CssHeight").entered();

        who.layout.h.set(self.clone().into());
    }
}
impl<Ix> RefreshFor<EmgEdgeItem<Ix>> for OriginX
where
    Ix: Clone + std::hash::Hash + Eq + Ord + 'static + Default,
{
    #[track_caller]
    fn refresh_for(&self, who: &mut EmgEdgeItem<Ix>) {
        let _g = trace_span!("-> RefreshFor<EmgEdgeItem> for OriginX").entered();

        who.layout.origin_x.set(self.clone().into());
    }
}
impl<Ix> RefreshFor<EmgEdgeItem<Ix>> for OriginY
where
    Ix: Clone + std::hash::Hash + Eq + Ord + 'static + Default,
{
    #[track_caller]
    fn refresh_for(&self, who: &mut EmgEdgeItem<Ix>) {
        let _g = trace_span!("-> RefreshFor<EmgEdgeItem> for OriginY").entered();

        who.layout.origin_y.set(self.clone().into());
    }
}

impl<Ix> RefreshFor<EmgEdgeItem<Ix>> for AlignX
where
    Ix: Clone + std::hash::Hash + Eq + Ord + 'static + Default,
{
    #[track_caller]
    fn refresh_for(&self, who: &mut EmgEdgeItem<Ix>) {
        let _g = trace_span!("-> RefreshFor<EmgEdgeItem> for AlignX").entered();

        who.layout.align_x.set(self.clone().into());
    }
}
impl<Ix> RefreshFor<EmgEdgeItem<Ix>> for AlignY
where
    Ix: Clone + std::hash::Hash + Eq + Ord + 'static + Default,
{
    #[track_caller]
    fn refresh_for(&self, who: &mut EmgEdgeItem<Ix>) {
        let _g = trace_span!("-> RefreshFor<EmgEdgeItem> for AlignY").entered();

        who.layout.align_y.set(self.clone().into());
    }
}

// ────────────────────────────────────────────────────────────────────────────────

// #[derive(Clone)]
// pub struct EffectingPath<Ix, T>(T, PhantomData<Ix>)
// where
//     T: RefreshFor<EmgEdgeItem<Ix>> + Clone + 'static,
//     Ix: Clone
//         + std::hash::Hash
//         + Eq
//         + Ord
//         + 'static
//         + Default
//         + std::fmt::Debug
//         + std::fmt::Display;

// impl<Ix, T> std::ops::Deref for EffectingPath<Ix, T>
// where
//     T: RefreshFor<EmgEdgeItem<Ix>> + Clone + 'static,
//     Ix: Clone
//         + std::hash::Hash
//         + Eq
//         + Ord
//         + 'static
//         + Default
//         + std::fmt::Debug
//         + std::fmt::Display,
// {
//     type Target = T;

//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

// impl<T, Ix> From<T> for EffectingPath<Ix, T>
// where
//     T: RefreshFor<EmgEdgeItem<Ix>> + Clone + 'static,
//     Ix: Clone
//         + std::hash::Hash
//         + Eq
//         + Ord
//         + 'static
//         + Default
//         + std::fmt::Debug
//         + std::fmt::Display,
// {
//     fn from(v: T) -> Self {
//         Self(v, PhantomData)
//     }
// }
// impl<Ix, T> RefreshFor<EmgEdgeItem<Ix>> for EffectingPath<Ix, T>
// where
//     T: RefreshFor<EmgEdgeItem<Ix>> + Clone + 'static,
//     Ix: Clone
//         + std::hash::Hash
//         + Eq
//         + Ord
//         + 'static
//         + Default
//         + std::fmt::Debug
//         + std::fmt::Display,
// {
//     fn refresh_for(&self, who: &mut EmgEdgeItem<Ix>) {
//         // self.effect_with_path(p,who);
//         // who.refresh_use(self);
//     }
// }
/// using at tree building
impl<Ix, Message> RefreshFor<EmgEdgeItem<Ix>> for AnimationE<Message>
where
    Message: Clone + std::fmt::Debug + 'static + PartialEq,
    Ix: Clone
        + std::hash::Hash
        + Eq
        + Ord
        + 'static
        + Default
        + std::fmt::Debug
        + std::fmt::Display,
{
    fn refresh_for(&self, edge: &mut EmgEdgeItem<Ix>) {
        //NOTE 当 tree 宏 中 在 edge中使用 am类型
        trace!(
            "AnimationE  RefreshFor EmgEdgeItem snapshot: \n{:#?}",
            illicit::Snapshot::get()
        );
        if let Ok(path) = illicit::get::<EPath<Ix>>() {
            let p = (*path).clone();
            self.effecting_edge_path(&*edge, p);
        }
    }
}

#[cfg(test)]
mod refresh_test {
    use std::time::Duration;

    use emg::edge_index_no_source;
    use emg::node_index;
    use emg_animation::to;
    use emg_core::vector;
    use emg_core::{into_vector, IdStr};
    use emg_refresh::RefreshForUse;
    use emg_state::CloneStateVar;
    use emg_state::{use_state, Dict, StateVar};
    use seed_styles as styles;
    use seed_styles::CssWidth;

    use styles::px;

    #[allow(unused)]
    use styles::{pc, width};

    use crate::animation::global_clock;
    use crate::EPath;
    use crate::GraphEdgesDict;
    use crate::{anima, AnimationE, EmgEdgeItem};

    #[allow(dead_code)]
    #[derive(Debug, Clone, PartialEq)]
    enum Message {
        A,
    }

    #[test]
    fn edge() {
        let e_dict_sv: StateVar<GraphEdgesDict<IdStr>> = use_state(Dict::new());
        let root_e_source = use_state(None);
        let root_e_target = use_state(Some(node_index("root")));
        let mut root_e = EmgEdgeItem::default_with_wh_in_topo(
            root_e_source.watch(),
            root_e_target.watch(),
            e_dict_sv.watch(),
            1920,
            1080,
        );

        let css_w: StateVar<CssWidth> = use_state(width(px(99)));
        let a: AnimationE<Message> = anima![css_w];
        illicit::Layer::new()
            .offer(EPath::<String>(vector![edge_index_no_source("root")]))
            .enter(|| {
                root_e.refresh_for_use(&a);
                // root_e.refresh_use(&a);
            });

        let now = global_clock();

        a.interrupt(vector![
            to(into_vector![width(px(0))]),
            to(into_vector![width(px(1))])
        ]);

        now.set(Duration::from_millis(16));
        insta::assert_debug_snapshot!("anima_refresh_edge_16", &a);
        insta::assert_debug_snapshot!("anima_refresh_edge_16_edge", &root_e);
        now.set(Duration::from_millis(33));
        insta::assert_debug_snapshot!("anima_refresh_edge_33", &a);
        insta::assert_debug_snapshot!("anima_refresh_edge_33_edge", &root_e);

        // a.effecting_edge_path(&root_e, EPath(vector![edge_index_no_source("root")]));

        // let mut pe = PathEItem(EPath(vector![edge_index_no_source("root")]), root_e);
        // bb.refresh_for(&mut pe.1);
        // bb.refresh_for(&mut pe);
        // let fff = bbb.as_ref();
        // pe.refresh_use(fff);
    }
}
