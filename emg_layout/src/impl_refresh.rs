/*
 * @Author: Rais
 * @Date: 2021-03-29 19:22:19
 * @LastEditTime: 2021-04-25 20:06:05
 * @LastEditors: Rais
 * @Description:
 */

use std::{any::Any, panic::Location};

use emg_refresh::{RefreshFor, RefreshUseFor, RefreshWhoNoWarper};

pub use seed_styles as styles;
use styles::{CssHeight, CssValueTrait, CssWidth, UpdateStyle};
use tracing::{debug, trace, trace_span};

use crate::{
    add_values::{AlignX, AlignY, OriginX, OriginY},
    Css, EmgEdgeItem, GenericLoc, GenericWH,
};

// ────────────────────────────────────────────────────────────────────────────────

//TODO lifetime
impl<Ix> RefreshWhoNoWarper for EmgEdgeItem<Ix> where
    Ix: Clone + std::hash::Hash + Eq + Ord + 'static + Default
{
}

// impl<T> RefreshUseNoWarper for Css<T> where T: CssValueTrait + Clone + 'static {}

//NOTE: overwrite default impl<Who> RefreshFor<Who> for Box<dyn RefreshFor<Who>>
impl<Ix> RefreshFor<EmgEdgeItem<Ix>> for Box<(dyn RefreshFor<EmgEdgeItem<Ix>> + 'static)>
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
fn css_refresh_edgedata<Use, Ix>(css: &Css<Use>, ed: &EmgEdgeItem<Ix>)
where
    Use: CssValueTrait + std::clone::Clone,
    Ix: Clone + std::hash::Hash + Eq + Ord + 'static + Default,
{
    let _g = trace_span!("-> css_refresh_edgedata").entered();

    let any = &css.0 as &dyn Any;
    if let Some(css_width) = any.downcast_ref::<CssWidth>() {
        debug!("match CssWidth {}", &css_width);
        ed.layout.size.set_with(|size| {
            let new = GenericWH {
                w: css_width.clone().into(),
                ..size.clone()
            };
            debug!("new {}", &new);
            new
        });
        return;
    }

    if let Some(css_height) = any.downcast_ref::<CssHeight>() {
        debug!("match CssHeight {}", &css_height);
        ed.layout.size.set_with(|size| {
            let new = GenericWH {
                h: css_height.clone().into(),
                ..size.clone()
            };

            debug!("new {}", &new);
            new
        });
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
impl<Ix> RefreshFor<EmgEdgeItem<Ix>> for OriginX
where
    Ix: Clone + std::hash::Hash + Eq + Ord + 'static + Default,
{
    #[track_caller]
    fn refresh_for(&self, who: &mut EmgEdgeItem<Ix>) {
        let _g = trace_span!("-> RefreshFor<EmgEdgeItem> for OriginX").entered();

        who.layout.origin.set_with(|origin| {
            let new = GenericLoc {
                x: self.clone().into(),
                ..origin.clone()
            };
            trace!("new {}", &new);
            new
        });
    }
}
impl<Ix> RefreshFor<EmgEdgeItem<Ix>> for OriginY
where
    Ix: Clone + std::hash::Hash + Eq + Ord + 'static + Default,
{
    #[track_caller]
    fn refresh_for(&self, who: &mut EmgEdgeItem<Ix>) {
        let _g = trace_span!("-> RefreshFor<EmgEdgeItem> for OriginY").entered();

        who.layout.origin.set_with(|origin| {
            let new = GenericLoc {
                y: self.clone().into(),
                ..origin.clone()
            };
            trace!("new {}", &new);
            new
        });
    }
}
impl<Ix> RefreshFor<EmgEdgeItem<Ix>> for AlignX
where
    Ix: Clone + std::hash::Hash + Eq + Ord + 'static + Default,
{
    #[track_caller]
    fn refresh_for(&self, who: &mut EmgEdgeItem<Ix>) {
        let _g = trace_span!("-> RefreshFor<EmgEdgeItem> for AlignX").entered();

        who.layout.align.set_with(|align| {
            let new = GenericLoc {
                x: self.clone().into(),
                ..align.clone()
            };
            trace!("new {}", &new);
            new
        });
    }
}
impl<Ix> RefreshFor<EmgEdgeItem<Ix>> for AlignY
where
    Ix: Clone + std::hash::Hash + Eq + Ord + 'static + Default,
{
    #[track_caller]
    fn refresh_for(&self, who: &mut EmgEdgeItem<Ix>) {
        let _g = trace_span!("-> RefreshFor<EmgEdgeItem> for AlignY").entered();

        who.layout.align.set_with(|align| {
            let new = GenericLoc {
                y: self.clone().into(),
                ..align.clone()
            };
            trace!("new {}", &new);
            new
        });
    }
}
// impl RefreshWhoNoWarper for EdgeItemNode {}
// impl<Who, Use> RefreshFor<Option<Who>> for Css<Use>
// where
//     Use: CssValueTrait + std::clone::Clone,
//     Css<Use>: RefreshFor<Who>,
// {
//     #[track_caller]
//     fn refresh_for(&self, who: &mut Option<Who>) {
//         let _g = trace_span!("-> RefreshFor<Option<Who>> for Css<Use>").entered();

//         who.as_mut().unwrap().refresh_use(self);
//     }
// }
// impl<Use> RefreshFor<EdgeData> for Css<Use>
// where
//     Use: CssValueTrait + std::clone::Clone,
// {
//     #[track_caller]
//     fn refresh_for(&self, who: &mut EdgeData) {
//         debug!("refresh_for : CssValueTrait:{}", &self.0);

//         css_refresh_edgedata(self, who);
//     }
// }

// impl RefreshFor<EdgeData> for Style {
//     fn refresh_for(&self, who: &mut EdgeData) {
//         who.path_styles.set(self.clone());
//     }
// }
// impl RefreshFor<EdgeData> for Style {
//     fn refresh_for(&self, who: &mut EdgeData) {
//         who.ed_output
//             .other_styles
//             .set_with(|s| s.clone().custom_style(self.clone()));
//     }
// }
// impl RefreshFor<EdgeData> for Layout {
//     fn refresh_for(&self, who: &mut EdgeData) {
//         who.layout = self.clone();
//     }
// }
