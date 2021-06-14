/*
 * @Author: Rais
 * @Date: 2021-03-29 19:22:19
 * @LastEditTime: 2021-06-13 16:05:09
 * @LastEditors: Rais
 * @Description:
 */

use std::{any::Any, panic::Location};

use emg_refresh::{RefreshFor, RefreshUseFor, RefreshUseNoWarper, RefreshWhoNoWarper};

use emg_state::{StateAnchor, StateVar};
pub use seed_styles as styles;
use styles::{CssHeight, CssValueTrait, CssWidth, UpdateStyle};
use tracing::{debug, trace_span, warn};

use crate::{
    add_values::{AlignX, AlignY, OriginX, OriginY},
    animation::AnimationEdge,
    Css, EmgEdgeItem,
};

// ────────────────────────────────────────────────────────────────────────────────

//TODO lifetime
impl<Ix> RefreshWhoNoWarper for EmgEdgeItem<Ix> where
    Ix: Clone + std::hash::Hash + Eq + Ord + 'static + Default
{
}

// impl<T> RefreshUseNoWarper for Css<T> where T: CssValueTrait + Clone + 'static {}

impl<Ix> RefreshFor<EmgEdgeItem<Ix>> for Box<(dyn RefreshFor<EmgEdgeItem<Ix>>)>
where
    Ix: Clone + std::hash::Hash + Eq + Ord + 'static + Default,
{
    #[track_caller]
    fn refresh_for(&self, who: &mut EmgEdgeItem<Ix>) {
        let _g = trace_span!(
            "!!!!!!!!!!!!!!-> RefreshFor<EdgeItem> for Vec<Box<(dyn RefreshFor<EdgeItem> + 'static)>>"
        )
        .entered();
        // let ii = i.as_ref();
        who.refresh_use(self.as_ref());
    }
}

impl<Ix, Use> RefreshFor<EmgEdgeItem<Ix>> for StateVar<Use>
where
    Ix: Clone + std::hash::Hash + Eq + Ord + 'static + Default,
    EmgEdgeItem<Ix>: RefreshWhoNoWarper,
    Use: RefreshUseNoWarper + RefreshFor<EmgEdgeItem<Ix>> + Clone + 'static,
{
    #[allow(clippy::redundant_closure_for_method_calls)]
    default fn refresh_for(&self, who: &mut EmgEdgeItem<Ix>) {
        let rc_var = self.get_var_with(|x| x.get());
        warn!("Edge  Refresh use StateVar");
        who.refresh_use(&*rc_var);
    }
}

impl<Ix> RefreshFor<EmgEdgeItem<Ix>> for StateVar<CssWidth>
where
    Ix: Clone + std::hash::Hash + Eq + Ord + 'static + Default,
    EmgEdgeItem<Ix>: RefreshWhoNoWarper,
    // Use: RefreshUseNoWarper + RefreshFor<EmgEdgeItem<Ix>> + Clone + 'static,
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
        ed.refresh_use(css_width);
        return;
    }

    if let Some(css_height) = any.downcast_ref::<CssHeight>() {
        debug!("dyn match CssHeight {}", &css_height);
        ed.refresh_use(css_height);
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
impl<Ix, Message> RefreshFor<EmgEdgeItem<Ix>> for AnimationEdge<Ix, Message>
where
    Ix: Clone + std::hash::Hash + Eq + Ord + 'static + Default,
    Message: Clone + std::fmt::Debug + PartialEq,
{
    #[track_caller]
    fn refresh_for(&self, who: &mut EmgEdgeItem<Ix>) {
        let _g =
            trace_span!("-> RefreshFor<EmgEdgeItem> using AnimationEdge<Ix, Message>").entered();

        // who.layout.w.set(self.clone().into());
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
