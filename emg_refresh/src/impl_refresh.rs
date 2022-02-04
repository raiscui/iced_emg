use std::{clone::Clone, rc::Rc};

/*
 * @Author: Rais
 * @Date: 2021-02-19 16:16:22
 * @LastEditTime: 2022-02-04 21:44:58
 * @LastEditors: Rais
 * @Description:
 */
use crate::RefreshFor;

use crate::{RefreshForUse, Refresher, RefresherFor};
use emg_state::{CloneStateAnchor, CloneStateVar, StateAnchor, StateVar};
use tracing::{debug, warn};
// ────────────────────────────────────────────────────────────────────────────────

// impl<Who> RefreshUseFor<Who> for AnchorWithUpdater<Who>
// where
//     Who: std::clone::Clone + GeneralRefreshFor,
// {
//     fn refresh_use(&mut self, updater: &dyn RefreshFor<Who>) {
//         let mut v = self.get();
//         updater.refresh_for(&mut v);
//         self.get_setter().set(v);
//     }
// }

// @ impl RefreshFor────────────────────────────────────────────────────────────────────────────────
pub auto trait RefreshWhoNoWarper {}
pub auto trait RefreshUseNoWarper {}
// ────────────────────────────────────────────────────────────────────────────────

// ────────────────────────────────────────────────────────────────────────────────

// impl<Use> !RefreshUseNoWarper for Vec<Use> {}
impl<Use> !RefreshUseNoWarper for Box<Use> {}
impl<Use> !RefreshUseNoWarper for Vec<Box<Use>> {}
impl<Use> !RefreshUseNoWarper for Rc<Use> {}
impl<Use> !RefreshUseNoWarper for StateVar<Use> {}
impl<Use> !RefreshUseNoWarper for StateAnchor<Use> {}
impl<Who> !RefreshWhoNoWarper for StateVar<Who> {}
impl<Use> !RefreshWhoNoWarper for StateAnchor<Use> {}
impl<'a, Use> !RefreshUseNoWarper for RefresherFor<'a, Use> {}
impl<'a, Use> !RefreshUseNoWarper for Refresher<'a, Use> {}
// ────────────────────────────────────────────────────────────────────────────────
// impl<Who> RefreshFor<Who> for Vector<Box<dyn RefreshFor<Who>>>
// where
//     Who: RefreshWhoNoWarper,
// {
//     default fn refresh_for(&self, who: &mut Who) {
//         self.iter().for_each(|i| {
//             let ii = i.as_ref();
//             who.refresh_for_use(ii);
//         });
//         // for i in self.iter() {
//         //     let ii = i.as_ref();
//         //     who.refresh_for_use(ii);
//         // }
//     }
// }
impl<Who> RefreshFor<Who> for Vec<Box<dyn RefreshFor<Who>>>
where
    Who: RefreshWhoNoWarper,
{
    default fn refresh_for(&self, who: &mut Who) {
        for i in self {
            let ii = i.as_ref();
            who.refresh_for_use(ii);
        }
    }
}
impl<Who> RefreshFor<Who> for Box<dyn RefreshFor<Who>>
where
    Who: RefreshWhoNoWarper,
{
    default fn refresh_for(&self, who: &mut Who) {
        let r = self.as_ref();
        who.refresh_for_use(r);
    }
}
// impl<Who> RefreshFor<Who> for Vector<Rc<dyn RefreshFor<Who>>>
// where
//     Who: RefreshWhoNoWarper,
// {
//     default fn refresh_for(&self, who: &mut Who) {
//         for i in self {
//             let ii = i.as_ref();
//             who.refresh_for_use(ii);
//         }
//     }
// }
impl<Who> RefreshFor<Who> for Vec<Rc<dyn RefreshFor<Who>>>
where
    Who: RefreshWhoNoWarper,
{
    default fn refresh_for(&self, who: &mut Who) {
        for i in self {
            let ii = i.as_ref();
            who.refresh_for_use(ii);
        }
    }
}
impl<Who> RefreshFor<Who> for Rc<dyn RefreshFor<Who>>
where
    Who: RefreshWhoNoWarper,
{
    default fn refresh_for(&self, who: &mut Who) {
        let r = self.as_ref();
        who.refresh_for_use(r);
    }
}
// impl RefreshUseNoWarper for Vec<u8> {}
impl<Who, Use> RefreshFor<Who> for Vec<Use>
where
    Who: RefreshWhoNoWarper,

    Use: RefreshUseNoWarper + RefreshFor<Who>,
{
    default fn refresh_for(&self, who: &mut Who) {
        for i in self {
            who.refresh_for_use(i);
        }
    }
}

impl<Who, Use> RefreshFor<Who> for Rc<Use>
where
    Who: RefreshWhoNoWarper,
    Use: RefreshUseNoWarper + RefreshFor<Who>,
{
    fn refresh_for(&self, who: &mut Who) {
        let u_s_e = self.as_ref();
        who.refresh_for_use(u_s_e);
    }
}
impl<Who, Use> RefreshFor<Who> for Box<Use>
where
    Who: RefreshWhoNoWarper,
    Use: RefreshUseNoWarper + RefreshFor<Who>,
{
    default fn refresh_for(&self, who: &mut Who) {
        who.refresh_for_use(self.as_ref());
    }
}

impl<Who, Use> RefreshFor<StateVar<Who>> for Use
where
    Who: RefreshWhoNoWarper + Clone + 'static + std::fmt::Debug,
    Use: RefreshUseNoWarper + RefreshFor<Who>,
{
    fn refresh_for(&self, who: &mut StateVar<Who>) {
        debug!("==========refresh_for StateVar");
        let mut w = who.get();
        w.refresh_for_use(self);
        who.set(w);
    }
}
// ────────────────────────────────────────────────────────────────────────────────

impl<Who, Use> RefreshFor<Who> for StateVar<Use>
where
    Who: RefreshWhoNoWarper,
    Use: RefreshUseNoWarper + RefreshFor<Who> + Clone + 'static,
{
    default fn refresh_for(&self, who: &mut Who) {
        warn!(
            "who:{:?} Refresh use StateVar:{:?}",
            &std::any::type_name::<Who>(),
            &std::any::type_name::<Use>()
        );

        who.refresh_for_use(&self.get());
    }
}
// ────────────────────────────────────────────────────────────────────────────────

impl<Who, Use> RefreshFor<StateVar<Who>> for StateVar<Use>
where
    Who: RefreshWhoNoWarper + Clone + 'static + std::fmt::Debug,
    Use: RefreshUseNoWarper + RefreshFor<Who> + Clone + 'static,
{
    fn refresh_for(&self, who: &mut StateVar<Who>) {
        let mut w = who.get();
        w.refresh_for_use(&self.get());

        who.set(w);
    }
}
// ────────────────────────────────────────────────────────────────────────────────

// impl<Who> RefreshFor<Who> for RefresherForSelf<Who> {
//     fn refresh_for(&self, who: &mut Who) {
//         self.get()(who);
//     }
// }
impl<'a, Who> RefreshFor<Who> for RefresherFor<'a, Who>
where
    Who: RefreshWhoNoWarper,
{
    fn refresh_for(&self, who: &mut Who) {
        self.get()(who);
    }
}

impl<'a, Who, Use> RefreshFor<Who> for Refresher<'a, Use>
where
    Who: RefreshWhoNoWarper,
    Use: RefreshUseNoWarper + RefreshFor<Who>,
{
    fn refresh_for(&self, who: &mut Who) {
        // self.get()().refresh_for(who);
        who.refresh_for_use(&self.get());
    }
}

// ────────────────────────────────────────────────────────────────────────────────

impl<Who, Use> RefreshFor<StateVar<Who>> for StateAnchor<Use>
where
    Who: RefreshWhoNoWarper + Clone + 'static + std::fmt::Debug,
    Use: RefreshUseNoWarper + RefreshFor<Who> + Clone + 'static + std::fmt::Debug,
{
    fn refresh_for(&self, who: &mut StateVar<Who>) {
        let u_s_e = self.get();
        let mut w = who.get();
        w.refresh_for_use(&u_s_e);
        who.set(w);
    }
}
impl<Who, Use> RefreshFor<StateVar<Who>> for Vec<Rc<StateAnchor<Use>>>
where
    Who: RefreshWhoNoWarper + Clone + 'static + std::fmt::Debug,
    Use: RefreshUseNoWarper + RefreshFor<Who> + Clone + 'static + std::fmt::Debug,
{
    fn refresh_for(&self, who: &mut StateVar<Who>) {
        for sa in self {
            let u_s_e = sa.get();
            let mut w = who.get();
            w.refresh_for_use(&u_s_e);
            who.set(w);
        }
    }
}
impl<Who> RefreshFor<StateVar<Who>> for Vec<Box<(dyn RefreshFor<Who> + 'static)>>
where
    Who: RefreshWhoNoWarper + Clone + 'static + std::fmt::Debug,
{
    fn refresh_for(&self, who: &mut StateVar<Who>) {
        for sa in self {
            let u_s_e = sa.as_ref();
            let mut w = who.get();
            w.refresh_for_use(u_s_e);
            who.set(w);
        }
    }
}
impl<Who, Use> RefreshFor<StateVar<Who>> for Vec<Box<Use>>
where
    Who: RefreshWhoNoWarper + Clone + 'static + std::fmt::Debug,
    Use: RefreshUseNoWarper + RefreshFor<Who> + Clone + 'static + std::fmt::Debug,
{
    fn refresh_for(&self, who: &mut StateVar<Who>) {
        for sa in self {
            let u_s_e = sa.as_ref();
            let mut w = who.get();
            w.refresh_for_use(u_s_e);
            who.set(w);
        }
    }
}

impl<Who, Use> RefreshFor<StateVar<Who>> for Vec<Box<StateAnchor<Use>>>
where
    Who: RefreshWhoNoWarper + Clone + 'static + std::fmt::Debug,
    Use: RefreshUseNoWarper + RefreshFor<Who> + Clone + 'static + std::fmt::Debug,
{
    fn refresh_for(&self, who: &mut StateVar<Who>) {
        for sa in self {
            let u_s_e = sa.get();
            let mut w = who.get();
            w.refresh_for_use(&u_s_e);
            who.set(w);
        }
    }
}

impl<Who, Use> RefreshFor<Who> for StateAnchor<Use>
where
    Who: RefreshWhoNoWarper,
    Use: RefreshUseNoWarper + RefreshFor<Who> + Clone + 'static + std::fmt::Debug,
{
    default fn refresh_for(&self, who: &mut Who) {
        let u_s_e = self.get();
        // log::debug!(" ============ StateAnchor get:{:?}", &u_s_e);
        who.refresh_for_use(&u_s_e);
    }
}
