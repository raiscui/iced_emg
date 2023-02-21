/*
 * @Author: Rais
 * @Date: 2021-03-29 19:22:19
 * @LastEditTime: 2023-02-21 12:11:39
 * @LastEditors: Rais
 * @Description:
 */
mod native;

use emg_shaping::{Shaping, ShapingUseDyn, ShapingUseNoWarper, ShapingWhoNoWarper};
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
impl<Ix> ShapingWhoNoWarper for EmgEdgeItem<Ix> where
    Ix: Clone + std::hash::Hash + Eq + Ord + 'static + Default
{
}

//TODO this is warper , try not write this way
impl<T> ShapingUseNoWarper for Css<T> where T: CssValueTrait + Clone + 'static {}

impl<Ix> Shaping<EmgEdgeItem<Ix>> for Box<dyn Shaping<EmgEdgeItem<Ix>>>
where
    Ix: Clone + std::hash::Hash + Eq + Ord + 'static + Default,
    EmgEdgeItem<Ix>: ShapingWhoNoWarper,
{
    #[track_caller]
    fn shaping(&self, who: &mut EmgEdgeItem<Ix>) -> bool {
        let _g = trace_span!(
            "!!!!!!!!!!!!!!-> Shaping<EdgeItem> for Box<(dyn Shaping<EdgeItem> + 'static)>"
        )
        .entered();
        // let ii = i.as_ref();
        who.shaping_use_dyn(self.as_ref())
    }
}

impl<Ix> Shaping<EmgEdgeItem<Ix>> for Rc<dyn Shaping<EmgEdgeItem<Ix>>>
where
    Ix: Clone + std::hash::Hash + Eq + Ord + 'static + Default,
    EmgEdgeItem<Ix>: ShapingWhoNoWarper,
{
    #[track_caller]
    fn shaping(&self, who: &mut EmgEdgeItem<Ix>) -> bool {
        let _g = trace_span!(
            "!!!!!!!!!!!!!!-> Shaping<EdgeItem> for Box<(dyn Shaping<EdgeItem> + 'static)>"
        )
        .entered();
        // let ii = i.as_ref();
        who.shaping_use_dyn(self.as_ref())
    }
}

impl<Ix, Use> Shaping<EmgEdgeItem<Ix>> for StateVar<Use>
where
    Ix: Clone + std::hash::Hash + Eq + Ord + 'static + Default,
    EmgEdgeItem<Ix>: ShapingWhoNoWarper,
    Use: ShapingUseNoWarper + Shaping<EmgEdgeItem<Ix>> + Clone + 'static,
{
    default fn shaping(&self, who: &mut EmgEdgeItem<Ix>) -> bool {
        let rc_v = self.get_var_with(emg_state::Var::get);
        warn!("Edge [default!!] Refresh use StateVar current value !!!");
        who.shaping_use_dyn(&*rc_v)
    }
}
// ────────────────────────────────────────────────────────────────────────────────
// ────────────────────────────────────────────────────────────────────────────────
impl<Ix> Shaping<EmgEdgeItem<Ix>> for StateVar<CssWidth>
where
    Ix: Clone + std::hash::Hash + Eq + Ord + 'static + Default,
    EmgEdgeItem<Ix>: ShapingWhoNoWarper,
{
    fn shaping(&self, who: &mut EmgEdgeItem<Ix>) -> bool {
        warn!("Edge  Refresh use StateVar<CssWidth>");

        who.layout.w.set(self.watch().into());

        // who.shaping_use(&*rc_var);
        true
    }
}
impl<Ix> Shaping<EmgEdgeItem<Ix>> for StateAnchor<CssWidth>
where
    Ix: Clone + std::hash::Hash + Eq + Ord + 'static + Default,
    EmgEdgeItem<Ix>: ShapingWhoNoWarper,
    // Use: ShapingUseNoWarper + Shaping<EmgEdgeItem<Ix>> + Clone + 'static,
{
    fn shaping(&self, who: &mut EmgEdgeItem<Ix>) -> bool {
        warn!("Edge  Refresh use StateAnchor<CssWidth>");

        who.layout.w.set(self.clone().into());

        // who.shaping_use(&*rc_var);
        true
    }
}
impl<Ix> Shaping<EmgEdgeItem<Ix>> for StateVar<CssHeight>
where
    Ix: Clone + std::hash::Hash + Eq + Ord + 'static + Default,
    EmgEdgeItem<Ix>: ShapingWhoNoWarper,
    // Use: ShapingUseNoWarper + Shaping<EmgEdgeItem<Ix>> + Clone + 'static,
{
    fn shaping(&self, who: &mut EmgEdgeItem<Ix>) -> bool {
        warn!("Edge  Refresh use StateVar<CssHeight>");

        who.layout.h.set(self.watch().into());

        // who.shaping_use(&*rc_var);
        true
    }
}
impl<Ix> Shaping<EmgEdgeItem<Ix>> for StateAnchor<CssHeight>
where
    Ix: Clone + std::hash::Hash + Eq + Ord + 'static + Default,
    EmgEdgeItem<Ix>: ShapingWhoNoWarper,
    // Use: ShapingUseNoWarper + Shaping<EmgEdgeItem<Ix>> + Clone + 'static,
{
    fn shaping(&self, who: &mut EmgEdgeItem<Ix>) -> bool {
        warn!("Edge  Refresh use StateAnchor<CssHeight>");

        who.layout.h.set(self.clone().into());

        // who.shaping_use(&*rc_var);
        true
    }
}

// ────────────────────────────────────────────────────────────────────────────────
// ────────────────────────────────────────────────────────────────────────────────

// impl<Ix> Shaping<EmgEdgeItem<Ix>> for Vec<Box<(dyn Shaping<EmgEdgeItem<Ix>> + 'static)>>
// where
//     Ix: Clone + std::hash::Hash + Eq + Ord + 'static + Default,
// {
//     #[track_caller]
//     fn shaping(&self, who: &mut EmgEdgeItem<Ix>) {
//         for i in self {
//             let _g = trace_span!(
//                 "-> Shaping<EdgeItem> for Vec<Box<(dyn Shaping<EdgeItem> + 'static)>>"
//             )
//             .entered();
//             // let ii = i.as_ref();
//             who.shaping_use(i.as_ref());
//         }
//     }
// }
// impl Shaping<EdgeData> for Vec<Box<(dyn Shaping<EdgeData> + 'static)>> {
//     #[track_caller]
//     fn shaping(&self, _who: &mut EdgeData) {
//         panic!("!!!!!!");
//         // for i in self {
//         //     // let ii = i.as_ref();
//         //     who.shaping_use(i.as_ref());
//         // }
//     }
// }
//TODO 做 不是refresh 版本的
#[track_caller]
fn css_refresh_edgedata<Use, Ix>(css: &Css<Use>, ei: &mut EmgEdgeItem<Ix>) -> bool
where
    Use: CssValueTrait + std::clone::Clone,
    Ix: Clone + std::hash::Hash + Eq + Ord + 'static + Default,
{
    let _g = trace_span!("-> css_refresh_edgedata").entered();

    let any = &css.0 as &dyn Any;
    if let Some(css_width) = any.downcast_ref::<CssWidth>() {
        debug!("dyn match CssWidth {}", &css_width);
        return ei.shaping_use_dyn(css_width);
    }

    if let Some(css_height) = any.downcast_ref::<CssHeight>() {
        debug!("dyn match CssHeight {}", &css_height);
        return ei.shaping_use_dyn(css_height);
    }

    {
        // @ 不唯一, 多次会重复 ─────────────────────────────────────────────────────────────────

        ei.other_css_styles.set_with_once(|s| {
            let mut tmp_s = s.clone();
            let t = css.0.clone();

            tmp_s
                .updated_at
                .push_back(format!("{}", Location::caller()));

            t.update_style(&mut tmp_s);
            tmp_s
        });
    }
    true
}
// ────────────────────────────────────────────────────────────────────────────────

impl<Use, Ix> Shaping<EmgEdgeItem<Ix>> for Css<Use>
where
    Use: CssValueTrait + std::clone::Clone,
    Ix: Clone + std::hash::Hash + Eq + Ord + 'static + Default,
{
    #[track_caller]
    fn shaping(&self, who: &mut EmgEdgeItem<Ix>) -> bool {
        let _g = trace_span!("-> Shaping<EdgeItem> for Css<Use>").entered();

        css_refresh_edgedata(self, who)
    }
}

// ────────────────────────────────────────────────────────────────────────────────

impl<Ix> Shaping<EmgEdgeItem<Ix>> for CssWidth
where
    Ix: Clone + std::hash::Hash + Eq + Ord + 'static + Default,
{
    #[track_caller]
    fn shaping(&self, who: &mut EmgEdgeItem<Ix>) -> bool {
        let _g = trace_span!("-> Shaping<EmgEdgeItem> for CssWidth").entered();

        who.layout.w.set(self.clone().into());
        true
    }
}

impl<Ix> Shaping<EmgEdgeItem<Ix>> for CssHeight
where
    Ix: Clone + std::hash::Hash + Eq + Ord + 'static + Default,
{
    #[track_caller]
    fn shaping(&self, who: &mut EmgEdgeItem<Ix>) -> bool {
        let _g = trace_span!("-> Shaping<EmgEdgeItem> for CssHeight").entered();

        who.layout.h.set(self.clone().into());
        true
    }
}
impl<Ix> Shaping<EmgEdgeItem<Ix>> for OriginX
where
    Ix: Clone + std::hash::Hash + Eq + Ord + 'static + Default,
{
    #[track_caller]
    fn shaping(&self, who: &mut EmgEdgeItem<Ix>) -> bool {
        let _g = trace_span!("-> Shaping<EmgEdgeItem> for OriginX").entered();

        who.layout.origin_x.set(self.clone().into());
        true
    }
}
impl<Ix> Shaping<EmgEdgeItem<Ix>> for OriginY
where
    Ix: Clone + std::hash::Hash + Eq + Ord + 'static + Default,
{
    #[track_caller]
    fn shaping(&self, who: &mut EmgEdgeItem<Ix>) -> bool {
        let _g = trace_span!("-> Shaping<EmgEdgeItem> for OriginY").entered();

        who.layout.origin_y.set(self.clone().into());
        true
    }
}

impl<Ix> Shaping<EmgEdgeItem<Ix>> for AlignX
where
    Ix: Clone + std::hash::Hash + Eq + Ord + 'static + Default,
{
    #[track_caller]
    fn shaping(&self, who: &mut EmgEdgeItem<Ix>) -> bool {
        let _g = trace_span!("-> Shaping<EmgEdgeItem> for AlignX").entered();

        who.layout.align_x.set(self.clone().into());
        true
    }
}
impl<Ix> Shaping<EmgEdgeItem<Ix>> for AlignY
where
    Ix: Clone + std::hash::Hash + Eq + Ord + 'static + Default,
{
    #[track_caller]
    fn shaping(&self, who: &mut EmgEdgeItem<Ix>) -> bool {
        let _g = trace_span!("-> Shaping<EmgEdgeItem> for AlignY").entered();

        who.layout.align_y.set(self.clone().into());
        true
    }
}

// ────────────────────────────────────────────────────────────────────────────────

// #[derive(Clone)]
// pub struct EffectingPath<Ix, T>(T, PhantomData<Ix>)
// where
//     T: Shaping<EmgEdgeItem<Ix>> + Clone + 'static,
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
//     T: Shaping<EmgEdgeItem<Ix>> + Clone + 'static,
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
//     T: Shaping<EmgEdgeItem<Ix>> + Clone + 'static,
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
// impl<Ix, T> Shaping<EmgEdgeItem<Ix>> for EffectingPath<Ix, T>
// where
//     T: Shaping<EmgEdgeItem<Ix>> + Clone + 'static,
//     Ix: Clone
//         + std::hash::Hash
//         + Eq
//         + Ord
//         + 'static
//         + Default
//         + std::fmt::Debug
//         + std::fmt::Display,
// {
//     fn shaping(&self, who: &mut EmgEdgeItem<Ix>) {
//         // self.effect_with_path(p,who);
//         // who.shaping_use(self);
//     }
// }
/// using at tree building
impl<Ix, Message> Shaping<EmgEdgeItem<Ix>> for AnimationE<Message>
where
    Message: Clone + std::fmt::Debug + 'static + PartialEq,
    Ix: std::borrow::Borrow<str>
        + Clone
        + std::hash::Hash
        + Eq
        + Ord
        + 'static
        + Default
        + std::fmt::Debug
        + std::fmt::Display,
{
    fn shaping(&self, edge: &mut EmgEdgeItem<Ix>) -> bool {
        //NOTE 当 tree 宏 中 在 edge中使用 am类型
        trace!(
            "AnimationE  Shaping EmgEdgeItem snapshot: \n{:#?}",
            illicit::Snapshot::get()
        );
        illicit::get::<EPath<Ix>>().map_or_else(
            |e| {
                panic!(" cannot get illicit env EPath for animationE::effecting_edge_path,e:{e:?}");
            },
            |path| {
                debug!("effecting_edge_path in shaping");
                let p = (*path).clone();

                //TODO 当 P不存在了,动画会怎样?
                self.effecting_edge_path(&*edge, p);
            },
        );
        //TODO return bool changed or not in self.effecting_edge_path
        true
    }
}

#[cfg(test)]
mod refresh_test {
    use std::time::Duration;

    use emg::{edge_index_no_source, node_index};
    use emg_animation::to;
    use emg_common::{im::vector, into_smvec, IdStr};
    use emg_shaping::ShapingUseDyn;
    use emg_state::{use_state, CloneStateVar, Dict, StateVar};
    use seed_styles as styles;
    use seed_styles::CssWidth;

    use styles::px;

    #[allow(unused)]
    use styles::{pc, width};

    use crate::global_clock;
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
        let e_dict_sv: StateVar<GraphEdgesDict<IdStr>> = use_state(Dict::new);
        let root_e_source = use_state(|| None);
        let root_e_target = use_state(|| Some(node_index("root")));
        let mut root_e = EmgEdgeItem::default_with_wh_in_topo(
            root_e_source.watch(),
            root_e_target.watch(),
            e_dict_sv.watch(),
            1920,
            1080,
        );

        let css_w: StateVar<CssWidth> = use_state(|| width(px(99)));
        let a: AnimationE<Message> = anima![css_w];
        illicit::Layer::new()
            .offer(EPath::<IdStr>(vector![edge_index_no_source("root")]))
            .enter(|| {
                root_e.shaping_use_dyn(&a);
                // root_e.shaping_use(&a);
            });

        let now = global_clock();

        a.interrupt([to(into_smvec![width(px(0))]), to(into_smvec![width(px(1))])]);

        now.set(Duration::from_millis(16));
        insta::assert_debug_snapshot!("anima_refresh_edge_16", &a);
        insta::assert_debug_snapshot!("anima_refresh_edge_16_edge", &root_e);
        now.set(Duration::from_millis(33));
        insta::assert_debug_snapshot!("anima_refresh_edge_33", &a);
        insta::assert_debug_snapshot!("anima_refresh_edge_33_edge", &root_e);

        // a.effecting_edge_path(&root_e, EPath(vector![edge_index_no_source("root")]));

        // let mut pe = PathEItem(EPath(vector![edge_index_no_source("root")]), root_e);
        // bb.shaping(&mut pe.1);
        // bb.shaping(&mut pe);
        // let fff = bbb.as_ref();
        // pe.shaping_use(fff);
    }
}
