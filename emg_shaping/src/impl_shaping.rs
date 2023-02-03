use std::{clone::Clone, rc::Rc};

/*
 * @Author: Rais
 * @Date: 2021-02-19 16:16:22
 * @LastEditTime: 2022-06-17 14:27:34
 * @LastEditors: Rais
 * @Description:
 */
use crate::{EqShaping, Shaping};

use crate::{Shaper, ShaperFor, ShapingUseDyn};
use dyn_partial_eq::DynPartialEq;
use emg_state::{CloneStateAnchor, CloneStateVar, StateAnchor, StateVar};
use tracing::{debug, warn};
// ────────────────────────────────────────────────────────────────────────────────

// impl<Who> RefreshUseFor<Who> for AnchorWithUpdater<Who>
// where
//     Who: std::clone::Clone + GeneralRefreshFor,
// {
//     fn shaping_use(&mut self, updater: &dyn Shaping<Who>) {
//         let mut v = self.get();
//         updater.shaping(&mut v);
//         self.get_setter().set(v);
//     }
// }

// @ impl Shaping────────────────────────────────────────────────────────────────────────────────
pub auto trait ShapingWhoNoWarper {}
pub auto trait ShapingUseNoWarper {}
// ────────────────────────────────────────────────────────────────────────────────

// ────────────────────────────────────────────────────────────────────────────────

// impl<Use> !ShapingUseNoWarper for Vec<Use> {}
impl<Use> !ShapingUseNoWarper for Box<Use> {}
impl<Use> !ShapingUseNoWarper for Vec<Box<Use>> {}
impl<Use> !ShapingUseNoWarper for Rc<Use> {}
impl<Use> !ShapingUseNoWarper for StateVar<Use> {}
impl<Use> !ShapingUseNoWarper for StateAnchor<Use> {}
impl<Who> !ShapingWhoNoWarper for StateVar<Who> {}
impl<Use> !ShapingWhoNoWarper for StateAnchor<Use> {}
impl<'a, Use> !ShapingUseNoWarper for ShaperFor<'a, Use> {}
impl<Use> !ShapingUseNoWarper for Shaper<Use> {}
// ────────────────────────────────────────────────────────────────────────────────
// impl<Who> Shaping<Who> for Vector<Box<dyn Shaping<Who>>>
// where
//     Who: ShapingWhoNoWarper,
// {
//     default fn shaping(&self, who: &mut Who) {
//         self.iter().for_each(|i| {
//             let ii = i.as_ref();
//             who.shape_of_use(ii);
//         });
//         // for i in self.iter() {
//         //     let ii = i.as_ref();
//         //     who.shape_of_use(ii);
//         // }
//     }
// }
impl<Who> Shaping<Who> for Vec<Box<dyn Shaping<Who>>>
where
    Who: ShapingWhoNoWarper,
{
    default fn shaping(&self, who: &mut Who) {
        for i in self {
            let ii = i.as_ref();
            who.shaping_use_dyn(ii);
        }
    }
}
impl<Who> Shaping<Who> for Box<dyn Shaping<Who>>
where
    Who: ShapingWhoNoWarper,
{
    default fn shaping(&self, who: &mut Who) {
        let r = self.as_ref();
        who.shaping_use_dyn(r);
    }
}
// impl<Who> Shaping<Who> for Vector<Rc<dyn Shaping<Who>>>
// where
//     Who: ShapingWhoNoWarper,
// {
//     default fn shaping(&self, who: &mut Who) {
//         for i in self {
//             let ii = i.as_ref();
//             who.shape_of_use(ii);
//         }
//     }
// }
impl<Who> Shaping<Who> for Vec<Rc<dyn Shaping<Who>>>
where
    Who: ShapingWhoNoWarper,
{
    default fn shaping(&self, who: &mut Who) {
        for i in self {
            let ii = i.as_ref();
            who.shaping_use_dyn(ii);
        }
    }
}
impl<Who> Shaping<Who> for Rc<dyn Shaping<Who>>
where
    Who: ShapingWhoNoWarper,
{
    default fn shaping(&self, who: &mut Who) {
        let r = self.as_ref();
        who.shaping_use_dyn(r);
    }
}
// impl ShapingUseNoWarper for Vec<u8> {}
impl<Who, Use> Shaping<Who> for Vec<Use>
where
    Who: ShapingWhoNoWarper,

    Use: ShapingUseNoWarper + Shaping<Who>,
{
    default fn shaping(&self, who: &mut Who) {
        for i in self {
            who.shaping_use_dyn(i);
        }
    }
}

impl<Who, Use> Shaping<Who> for Rc<Use>
where
    Who: ShapingWhoNoWarper,
    Use: ShapingUseNoWarper + Shaping<Who>,
{
    fn shaping(&self, who: &mut Who) {
        let u_s_e = self.as_ref();
        who.shaping_use_dyn(u_s_e);
    }
}
impl<Who, Use> Shaping<Who> for Box<Use>
where
    Who: ShapingWhoNoWarper,
    Use: ShapingUseNoWarper + Shaping<Who>,
{
    default fn shaping(&self, who: &mut Who) {
        who.shaping_use_dyn(self.as_ref());
    }
}

impl<Who, Use> Shaping<StateVar<Who>> for Use
where
    Who: ShapingWhoNoWarper + Clone + 'static + std::fmt::Debug,
    Use: ShapingUseNoWarper + Shaping<Who>,
{
    fn shaping(&self, who: &mut StateVar<Who>) {
        debug!("==========shaping StateVar");
        let mut w = who.get();
        w.shaping_use_dyn(self);
        who.set(w);
    }
}
// ────────────────────────────────────────────────────────────────────────────────
// impl<Who, Use> EqShaping<Who> for StateVar<Use>
// where
//     Who: ShapingWhoNoWarper,
//     Use: ShapingUseNoWarper + Shaping<Who> + Clone + 'static + PartialEq,
// {
// }

impl<Who, Use> Shaping<Who> for StateVar<Use>
where
    Who: ShapingWhoNoWarper,
    Use: ShapingUseNoWarper + Shaping<Who> + Clone + 'static,
{
    default fn shaping(&self, who: &mut Who) {
        warn!(
            "who:{:?} Refresh use StateVar:{:?}",
            &std::any::type_name::<Who>(),
            &std::any::type_name::<Use>()
        );

        who.shaping_use_dyn(&self.get());
    }
}
// ────────────────────────────────────────────────────────────────────────────────

impl<Who, Use> Shaping<StateVar<Who>> for StateVar<Use>
where
    Who: ShapingWhoNoWarper + Clone + 'static + std::fmt::Debug,
    Use: ShapingUseNoWarper + Shaping<Who> + Clone + 'static,
{
    fn shaping(&self, who: &mut StateVar<Who>) {
        let mut w = who.get();
        w.shaping_use_dyn(&self.get());

        who.set(w);
    }
}
// ────────────────────────────────────────────────────────────────────────────────

// impl<Who> Shaping<Who> for RefresherForSelf<Who> {
//     fn shaping(&self, who: &mut Who) {
//         self.get()(who);
//     }
// }
impl<'a, Who> Shaping<Who> for ShaperFor<'a, Who>
where
    Who: ShapingWhoNoWarper,
{
    fn shaping(&self, who: &mut Who) {
        self.get()(who);
    }
}

impl<Who, Use> Shaping<Who> for Shaper<Use>
where
    Who: ShapingWhoNoWarper,
    Use: ShapingUseNoWarper + Shaping<Who>,
{
    fn shaping(&self, who: &mut Who) {
        // self.get()().shaping(who);
        who.shaping_use_dyn(&self.get());
    }
}
// impl<Who, Use> EqShaping<Who> for Shaper<Use>
// where
//     Who: ShapingWhoNoWarper,
//     Use: ShapingUseNoWarper + Shaping<Who> + 'static,
// {
// }

// ────────────────────────────────────────────────────────────────────────────────

impl<Who, Use> Shaping<StateVar<Who>> for StateAnchor<Use>
where
    Who: ShapingWhoNoWarper + Clone + 'static + std::fmt::Debug,
    Use: ShapingUseNoWarper + Shaping<Who> + Clone + 'static + std::fmt::Debug,
{
    fn shaping(&self, who: &mut StateVar<Who>) {
        let u_s_e = self.get();
        let mut w = who.get();
        w.shaping_use_dyn(&u_s_e);
        who.set(w);
    }
}
impl<Who, Use> Shaping<StateVar<Who>> for Vec<Rc<StateAnchor<Use>>>
where
    Who: ShapingWhoNoWarper + Clone + 'static + std::fmt::Debug,
    Use: ShapingUseNoWarper + Shaping<Who> + Clone + 'static + std::fmt::Debug,
{
    fn shaping(&self, who: &mut StateVar<Who>) {
        for sa in self {
            let u_s_e = sa.get();
            let mut w = who.get();
            w.shaping_use_dyn(&u_s_e);
            who.set(w);
        }
    }
}
impl<Who> Shaping<StateVar<Who>> for Vec<Box<(dyn Shaping<Who> + 'static)>>
where
    Who: ShapingWhoNoWarper + Clone + 'static + std::fmt::Debug,
{
    fn shaping(&self, who: &mut StateVar<Who>) {
        for sa in self {
            let u_s_e = sa.as_ref();
            let mut w = who.get();
            w.shaping_use_dyn(u_s_e);
            who.set(w);
        }
    }
}
impl<Who, Use> Shaping<StateVar<Who>> for Vec<Box<Use>>
where
    Who: ShapingWhoNoWarper + Clone + 'static + std::fmt::Debug,
    Use: ShapingUseNoWarper + Shaping<Who> + Clone + 'static + std::fmt::Debug,
{
    fn shaping(&self, who: &mut StateVar<Who>) {
        for sa in self {
            let u_s_e = sa.as_ref();
            let mut w = who.get();
            w.shaping_use_dyn(u_s_e);
            who.set(w);
        }
    }
}

impl<Who, Use> Shaping<StateVar<Who>> for Vec<Box<StateAnchor<Use>>>
where
    Who: ShapingWhoNoWarper + Clone + 'static + std::fmt::Debug,
    Use: ShapingUseNoWarper + Shaping<Who> + Clone + 'static + std::fmt::Debug,
{
    fn shaping(&self, who: &mut StateVar<Who>) {
        for sa in self {
            let u_s_e = sa.get();
            let mut w = who.get();
            w.shaping_use_dyn(&u_s_e);
            who.set(w);
        }
    }
}
// impl<Who, Use> EqShaping<Who> for StateAnchor<Use>
// where
//     Who: ShapingWhoNoWarper,
//     Use: ShapingUseNoWarper + Shaping<Who> + Clone + 'static + std::cmp::PartialEq,
// {
// }

impl<Who, Use> Shaping<Who> for StateAnchor<Use>
where
    Who: ShapingWhoNoWarper,
    Use: ShapingUseNoWarper + Shaping<Who> + Clone + 'static,
{
    default fn shaping(&self, who: &mut Who) {
        let u_s_e = self.get();
        // log::debug!(" ============ StateAnchor get:{:?}", &u_s_e);
        who.shaping_use_dyn(&u_s_e);
    }
}
impl<Who, Use> EqShaping<Who> for Use
where
    Who: ShapingWhoNoWarper,
    Use: Shaping<Who> + DynPartialEq,
{
}
// impl<Who, Use> EqShaping<Who> for Use
// where
//     Who: ShapingWhoNoWarper,
//     Use: Shaping<Who> + 'static + PartialEq,
// {
// }
