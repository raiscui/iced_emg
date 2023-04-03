/*
 * @Author: Rais
 * @Date: 2021-03-29 19:22:19
 * @LastEditTime: 2023-04-03 12:39:12
 * @LastEditors: Rais
 * @Description:
 */
mod native;
use emg_shaping::{Shaping, ShapingUseDyn, ShapingUseNoWarper, ShapingWhoNoWarper};

use std::{any::Any, panic::Location};

use emg_state::{
    anchors::expert::{voa, CastIntoValOrAnchor},
    CloneState, StateAnchor, StateVar,
};
pub use seed_styles as styles;
use styles::{CssHeight, CssValueTrait, CssWidth, UpdateStyle};
use tracing::{debug, error, trace, trace_span, warn};

use crate::{
    add_values::{AlignX, AlignY, OriginX, OriginY},
    animation::AnimationE,
    Css, EPath, EmgEdgeItem,
};

// ────────────────────────────────────────────────────────────────────────────────

//TODO lifetime
impl ShapingWhoNoWarper for EmgEdgeItem {}

//TODO (check can disable this line )this is warper , try not write this way
impl<T> ShapingUseNoWarper for Css<T> where T: CssValueTrait + Clone + 'static {}

//NOTE no need , in Shaping crate, has default impl
// impl Shaping<EmgEdgeItem> for Box<dyn Shaping<EmgEdgeItem>>
// where
//     EmgEdgeItem: ShapingWhoNoWarper,
// {
//     #[track_caller]
//     fn shaping(&self, who: &mut EmgEdgeItem) -> bool {
//         let _g = trace_span!(
//             "!!!!!!!!!!!!!!-> Shaping<EdgeItem> for Box<(dyn Shaping<EdgeItem> + 'static)>"
//         )
//         .entered();
//         // let ii = i.as_ref();
//         who.shaping_use_dyn(self.as_ref())
//     }
// }

//NOTE no need , in Shaping crate, has default impl
// impl Shaping<EmgEdgeItem> for Rc<dyn Shaping<EmgEdgeItem>>
// where
//     EmgEdgeItem: ShapingWhoNoWarper,
// {
//     #[track_caller]
//     fn shaping(&self, who: &mut EmgEdgeItem) -> bool {
//         let _g = trace_span!(
//             "!!!!!!!!!!!!!!-> Shaping<EdgeItem> for Rc<(dyn Shaping<EdgeItem> + 'static)>"
//         )
//         .entered();
//         // let ii = i.as_ref();
//         who.shaping_use_dyn(self.as_ref())
//     }
// }

// impl<Use> Shaping<EmgEdgeItem> for StateVar<Use>
// where
//     EmgEdgeItem: ShapingWhoNoWarper,
//     Use: ShapingUseNoWarper + Shaping<EmgEdgeItem> + Clone + 'static,
// {
//     default fn shaping(&self, who: &mut EmgEdgeItem) -> bool {
//         let rc_v = self.get_var_with(emg_state::Var::get);

//         warn!(target:"shaping","Edge [default!!] Refresh use StateVar current value !!!, {} {}",Red.paint("this is only once shaping") ,std::any::type_name::<Use>());
//         who.shaping_use_dyn(&*rc_v)
//     }
// }
// ────────────────────────────────────────────────────────────────────────────────
// ────────────────────────────────────────────────────────────────────────────────
impl Shaping<EmgEdgeItem> for StateVar<CssWidth>
where
    EmgEdgeItem: ShapingWhoNoWarper,
{
    fn shaping(&self, who: &mut EmgEdgeItem) -> bool {
        warn!("Edge  Refresh use StateVar<CssWidth>");

        who.layout.w.set(self.watch().cast_into());

        true
    }
}
impl Shaping<EmgEdgeItem> for StateAnchor<CssWidth>
where
    EmgEdgeItem: ShapingWhoNoWarper,
{
    fn shaping(&self, who: &mut EmgEdgeItem) -> bool {
        warn!("Edge  Refresh use StateAnchor<CssWidth>");

        who.layout.w.set(self.clone().cast_into());

        true
    }
}
impl Shaping<EmgEdgeItem> for StateVar<CssHeight>
where
    EmgEdgeItem: ShapingWhoNoWarper,
{
    fn shaping(&self, who: &mut EmgEdgeItem) -> bool {
        warn!("Edge  Refresh use StateVar<CssHeight>");

        who.layout.h.set(self.watch().cast_into());

        true
    }
}
impl Shaping<EmgEdgeItem> for StateAnchor<CssHeight>
where
    EmgEdgeItem: ShapingWhoNoWarper,
{
    fn shaping(&self, who: &mut EmgEdgeItem) -> bool {
        warn!("Edge  Refresh use StateAnchor<CssHeight>");

        who.layout.h.set(self.clone().cast_into());

        true
    }
}
impl Shaping<EmgEdgeItem> for StateVar<AlignX>
where
    EmgEdgeItem: ShapingWhoNoWarper,
{
    fn shaping(&self, who: &mut EmgEdgeItem) -> bool {
        warn!(target:"shaping","Edge  Refresh use StateVar<CssWidth>");

        who.layout.align_x.set(self.watch().cast_into());

        true
    }
}
impl Shaping<EmgEdgeItem> for StateAnchor<AlignX>
where
    EmgEdgeItem: ShapingWhoNoWarper,
{
    fn shaping(&self, who: &mut EmgEdgeItem) -> bool {
        warn!(target:"shaping","Edge  Refresh use StateAnchor<CssWidth>");

        who.layout.align_x.set(self.clone().cast_into());

        true
    }
}
impl Shaping<EmgEdgeItem> for StateVar<AlignY>
where
    EmgEdgeItem: ShapingWhoNoWarper,
{
    fn shaping(&self, who: &mut EmgEdgeItem) -> bool {
        warn!(target:"shaping","Edge  Refresh use StateVar<CssWidth>");

        who.layout.align_y.set(self.watch().cast_into());

        true
    }
}
impl Shaping<EmgEdgeItem> for StateAnchor<AlignY>
where
    EmgEdgeItem: ShapingWhoNoWarper,
{
    fn shaping(&self, who: &mut EmgEdgeItem) -> bool {
        warn!(target:"shaping","Edge  Refresh use StateAnchor<CssWidth>");

        who.layout.align_y.set(self.clone().cast_into());

        true
    }
}
impl Shaping<EmgEdgeItem> for StateVar<OriginX>
where
    EmgEdgeItem: ShapingWhoNoWarper,
{
    fn shaping(&self, who: &mut EmgEdgeItem) -> bool {
        warn!("Edge  Refresh use StateVar<CssWidth>");

        who.layout.origin_x.set(self.watch().cast_into());

        true
    }
}
impl Shaping<EmgEdgeItem> for StateAnchor<OriginX>
where
    EmgEdgeItem: ShapingWhoNoWarper,
{
    fn shaping(&self, who: &mut EmgEdgeItem) -> bool {
        warn!("Edge  Refresh use StateAnchor<CssWidth>");

        who.layout.origin_x.set(self.clone().cast_into());

        true
    }
}
impl Shaping<EmgEdgeItem> for StateVar<OriginY>
where
    EmgEdgeItem: ShapingWhoNoWarper,
{
    fn shaping(&self, who: &mut EmgEdgeItem) -> bool {
        warn!("Edge  Refresh use StateVar<CssWidth>");

        who.layout.origin_y.set(self.watch().cast_into());

        true
    }
}
impl Shaping<EmgEdgeItem> for StateAnchor<OriginY>
where
    EmgEdgeItem: ShapingWhoNoWarper,
{
    fn shaping(&self, who: &mut EmgEdgeItem) -> bool {
        warn!("Edge  Refresh use StateAnchor<CssWidth>");

        who.layout.origin_y.set(self.clone().cast_into());

        true
    }
}
// ────────────────────────────────────────────────────────────────────────────────
// ────────────────────────────────────────────────────────────────────────────────

//TODO 移除,因为使用了 impl_refresh/native.rs ,但是要考虑 other_css_styles如何处理
#[track_caller]
fn css_refresh_edgedata<Use>(css: &Css<Use>, ei: &mut EmgEdgeItem) -> bool
where
    Use: CssValueTrait + std::clone::Clone,
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

impl<Use> Shaping<EmgEdgeItem> for Css<Use>
where
    Use: CssValueTrait + std::clone::Clone,
{
    #[track_caller]
    fn shaping(&self, who: &mut EmgEdgeItem) -> bool {
        let _g = trace_span!("-> Shaping<EdgeItem> for Css<Use>").entered();

        css_refresh_edgedata(self, who)
    }
}

// ────────────────────────────────────────────────────────────────────────────────

impl Shaping<EmgEdgeItem> for CssWidth {
    #[track_caller]
    fn shaping(&self, who: &mut EmgEdgeItem) -> bool {
        let _g = trace_span!("-> Shaping<EmgEdgeItem> for CssWidth").entered();

        who.layout.w.set(self.clone());
        true
    }
}

impl Shaping<EmgEdgeItem> for CssHeight {
    #[track_caller]
    fn shaping(&self, who: &mut EmgEdgeItem) -> bool {
        let _g = trace_span!("-> Shaping<EmgEdgeItem> for CssHeight").entered();

        who.layout.h.set(self.clone());
        true
    }
}
impl Shaping<EmgEdgeItem> for OriginX {
    #[track_caller]
    fn shaping(&self, who: &mut EmgEdgeItem) -> bool {
        let _g = trace_span!("-> Shaping<EmgEdgeItem> for OriginX").entered();

        who.layout.origin_x.set(self.clone());
        true
    }
}
impl Shaping<EmgEdgeItem> for OriginY {
    #[track_caller]
    fn shaping(&self, who: &mut EmgEdgeItem) -> bool {
        let _g = trace_span!("-> Shaping<EmgEdgeItem> for OriginY").entered();

        who.layout.origin_y.set(self.clone());
        true
    }
}

impl Shaping<EmgEdgeItem> for AlignX {
    #[track_caller]
    fn shaping(&self, who: &mut EmgEdgeItem) -> bool {
        let _g = trace_span!("-> Shaping<EmgEdgeItem> for AlignX").entered();
        who.layout.align_x.set(self.clone());
        true
    }
}
impl Shaping<EmgEdgeItem> for AlignY {
    #[track_caller]
    fn shaping(&self, who: &mut EmgEdgeItem) -> bool {
        let _g = trace_span!("-> Shaping<EmgEdgeItem> for AlignY").entered();

        who.layout.align_y.set(self.clone());
        true
    }
}

/// using at tree building
impl<Message> Shaping<EmgEdgeItem> for AnimationE<Message>
where
    Message: Clone + std::fmt::Debug + 'static + PartialEq,
{
    fn shaping(&self, edge: &mut EmgEdgeItem) -> bool {
        //NOTE 当 tree 宏 中 在 edge中使用 am类型
        trace!(
            "AnimationE  Shaping EmgEdgeItem snapshot: \n{:#?}",
            illicit::Snapshot::get()
        );
        //TODO 默认 动画 需要通 其他 类型一样 应用到全部path 只有特别指定 才会单独一条path
        illicit::get::<EPath>().map_or_else(
            |e| {
                panic!(" cannot get illicit env EPath for animationE::effecting_edge_path,e:{e:?}");
            },
            |path| {
                debug!("effecting_edge_path in shaping");
                let p = (*path).clone();

                //TODO 当 P不存在了,动画会怎样?
                //TODO 测试,有可能是应用到了全部 layout
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
    use emg_state::{use_state, CloneState, Dict, StateVar};
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
        let e_dict_sv: StateVar<GraphEdgesDict> = use_state(Dict::new);
        let root_e_source = use_state(|| None);
        let root_e_target = use_state(|| Some(node_index("root")));
        let mut root_e = EmgEdgeItem::default_with_wh_in_topo(
            &root_e_source.watch(),
            &root_e_target.watch(),
            e_dict_sv.watch(),
            1920,
            1080,
        );

        let css_w: StateVar<CssWidth> = use_state(|| width(px(99)));
        let a: AnimationE<Message> = anima![css_w];
        illicit::Layer::new()
            .offer(EPath(vector![edge_index_no_source("root")]))
            .enter(|| {
                root_e.shaping_use_dyn(&a);
                // root_e.shaping_use(&a);
            });

        let now = global_clock();

        a.interrupt([to(into_smvec![width(px(0))]), to(into_smvec![width(px(1))])]);

        now.set(Duration::from_millis(16));
        #[cfg(feature = "insta")]
        insta::assert_debug_snapshot!("anima_refresh_edge_16", &a);
        #[cfg(feature = "insta")]
        insta::assert_debug_snapshot!("anima_refresh_edge_16_edge", &root_e);
        now.set(Duration::from_millis(33));
        #[cfg(feature = "insta")]
        insta::assert_debug_snapshot!("anima_refresh_edge_33", &a);
        #[cfg(feature = "insta")]
        insta::assert_debug_snapshot!("anima_refresh_edge_33_edge", &root_e);

        // a.effecting_edge_path(&root_e, EPath(vector![edge_index_no_source("root")]));

        // let mut pe = PathEItem(EPath(vector![edge_index_no_source("root")]), root_e);
        // bb.shaping(&mut pe.1);
        // bb.shaping(&mut pe);
        // let fff = bbb.as_ref();
        // pe.shaping_use(fff);
    }
}
