use std::{alloc::Global, clone::Clone, rc::Rc};

/*
 * @Author: Rais
 * @Date: 2021-02-19 16:16:22
 * @LastEditTime: 2023-03-16 16:55:37
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

impl<Use, A> !ShapingUseNoWarper for Box<Use, A> {}
impl<Use, A> !ShapingUseNoWarper for Vec<Use, A> {}
impl<Use> !ShapingUseNoWarper for Rc<Use> {}
impl<Use> !ShapingUseNoWarper for StateVar<Use> {}
impl<Use> !ShapingUseNoWarper for StateAnchor<Use> {}
impl<Who> !ShapingWhoNoWarper for StateVar<Who> {}
impl<Use> !ShapingWhoNoWarper for StateAnchor<Use> {}
impl<'a, Use> !ShapingUseNoWarper for ShaperFor<'a, Use> {}
impl<Use> !ShapingUseNoWarper for Shaper<Use> {}
// ────────────────────────────────────────────────────────────────────────────────

impl<Who> Shaping<Who> for Vec<Box<dyn Shaping<Who>>>
where
    Who: ShapingWhoNoWarper,
{
    default fn shaping(&self, who: &mut Who) -> bool {
        let mut is_changed = false;
        for i in self {
            is_changed |= who.shaping_use_dyn(i.as_ref());
        }
        is_changed
    }
}
impl<Who> Shaping<Who> for Box<dyn Shaping<Who>>
where
    Who: ShapingWhoNoWarper,
{
    default fn shaping(&self, who: &mut Who) -> bool {
        who.shaping_use_dyn(self.as_ref())
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
    default fn shaping(&self, who: &mut Who) -> bool {
        let mut is_changed = false;
        for i in self {
            is_changed |= who.shaping_use_dyn(i.as_ref());
        }
        is_changed
    }
}
impl<Who> Shaping<Who> for Rc<dyn Shaping<Who>>
where
    Who: ShapingWhoNoWarper,
{
    default fn shaping(&self, who: &mut Who) -> bool {
        who.shaping_use_dyn(self)
    }
}
// impl ShapingUseNoWarper for Vec<u8> {}
impl<Who, Use> Shaping<Who> for Vec<Use>
where
    Who: ShapingWhoNoWarper,

    Use: ShapingUseNoWarper + Shaping<Who>,
{
    default fn shaping(&self, who: &mut Who) -> bool {
        let mut is_changed = false;
        for i in self {
            is_changed |= who.shaping_use_dyn(i);
        }
        is_changed
    }
}

impl<Who, Use> Shaping<Who> for Rc<Use>
where
    Who: ShapingWhoNoWarper,
    Use: ShapingUseNoWarper + Shaping<Who>,
{
    fn shaping(&self, who: &mut Who) -> bool {
        who.shaping_use_dyn(self.as_ref())
    }
}
impl<Who, Use> Shaping<Who> for Box<Use>
where
    Who: ShapingWhoNoWarper,
    Use: ShapingUseNoWarper + Shaping<Who>,
{
    default fn shaping(&self, who: &mut Who) -> bool {
        who.shaping_use_dyn(self.as_ref())
    }
}

impl<Who, Use> Shaping<StateVar<Who>> for Use
where
    Who: ShapingWhoNoWarper + Clone + 'static + std::fmt::Debug,
    Use: ShapingUseNoWarper + Shaping<Who>,
{
    fn shaping(&self, who: &mut StateVar<Who>) -> bool {
        debug!("==========shaping StateVar");
        let mut w = who.get();
        let is_changed = w.shaping_use_dyn(self);
        if is_changed {
            who.set(w);
        }
        is_changed
    }
}
// ────────────────────────────────────────────────────────────────────────────────
//NOTE  禁用,因为某些情况 用户是希望 使用 StateVar watch 变更 到 who,在这里默认行为并不唯一
// impl<Who, Use> Shaping<Who> for StateVar<Use>
// where
//     Who: ShapingWhoNoWarper,
//     Use: ShapingUseNoWarper + Shaping<Who> + Clone + 'static,
// {
//     default fn shaping(&self, who: &mut Who) -> bool {
//         warn!(
//             "who:{:?} Refresh use StateVar:{:?}",
//             &std::any::type_name::<Who>(),
//             &std::any::type_name::<Use>()
//         );

//         who.shaping_use_dyn(&self.get())
//     }
// }
// ────────────────────────────────────────────────────────────────────────────────
//NOTE 不确定默认行为
// impl<Who, Use> Shaping<StateVar<Who>> for StateVar<Use>
// where
//     Who: ShapingWhoNoWarper + Clone + 'static + std::fmt::Debug,
//     Use: ShapingUseNoWarper + Shaping<Who> + Clone + 'static,
// {
//     fn shaping(&self, who: &mut StateVar<Who>) -> bool {
//         let mut w = who.get();
//         let is_changed = w.shaping_use_dyn(&self.get());

//         if is_changed {
//             who.set(w);
//         }
//         is_changed
//     }
// }
// ────────────────────────────────────────────────────────────────────────────────

impl<'a, Who> Shaping<Who> for ShaperFor<'a, Who>
where
    Who: ShapingWhoNoWarper,
{
    fn shaping(&self, who: &mut Who) -> bool {
        self.get()(who)
    }
}

impl<Who, Use> Shaping<Who> for Shaper<Use>
where
    Who: ShapingWhoNoWarper,
    Use: ShapingUseNoWarper + Shaping<Who>,
{
    fn shaping(&self, who: &mut Who) -> bool {
        // self.get()().shaping(who);
        who.shaping_use_dyn(&self.get())
    }
}

// ────────────────────────────────────────────────────────────────────────────────
//NOTE 不确定默认行为
// impl<Who, Use> Shaping<StateVar<Who>> for StateAnchor<Use>
// where
//     Who: ShapingWhoNoWarper + Clone + 'static + std::fmt::Debug,
//     Use: ShapingUseNoWarper + Shaping<Who> + Clone + 'static + std::fmt::Debug,
// {
//     fn shaping(&self, who: &mut StateVar<Who>) -> bool {
//         let mut w = who.get();
//         let is_changed = w.shaping_use_dyn(&self.get());
//         if is_changed {
//             who.set(w);
//         }
//         is_changed
//     }
// }
//NOTE 不确定默认行为
// impl<Who, Use> Shaping<StateVar<Who>> for Vec<Rc<StateAnchor<Use>>>
// where
//     Who: ShapingWhoNoWarper + Clone + 'static + std::fmt::Debug,
//     Use: ShapingUseNoWarper + Shaping<Who> + Clone + 'static + std::fmt::Debug,
// {
//     fn shaping(&self, who: &mut StateVar<Who>) -> bool {
//         let mut is_changed = false;
//         let mut w = who.get();
//         for sa in self {
//             is_changed |= w.shaping_use_dyn(&sa.get());
//         }
//         if is_changed {
//             who.set(w);
//         }
//         is_changed
//     }
// }
impl<Who> Shaping<StateVar<Who>> for Vec<Box<(dyn Shaping<Who> + 'static)>>
where
    Who: ShapingWhoNoWarper + Clone + 'static + std::fmt::Debug,
{
    fn shaping(&self, who: &mut StateVar<Who>) -> bool {
        let mut is_changed = false;
        let mut w = who.get();
        for sa in self {
            is_changed |= w.shaping_use_dyn(sa.as_ref());
        }
        if is_changed {
            who.set(w);
        }
        is_changed
    }
}
impl<Who, Use> Shaping<StateVar<Who>> for Vec<Box<Use>>
where
    Who: ShapingWhoNoWarper + Clone + 'static + std::fmt::Debug,
    Use: ShapingUseNoWarper + Shaping<Who> + Clone + 'static + std::fmt::Debug,
{
    fn shaping(&self, who: &mut StateVar<Who>) -> bool {
        let mut is_changed = false;
        let mut w = who.get();

        for sa in self {
            is_changed |= w.shaping_use_dyn(sa.as_ref());
        }
        if is_changed {
            who.set(w);
        }
        is_changed
    }
}

//NOTE 不确定默认行为
// impl<Who, Use> Shaping<StateVar<Who>> for Vec<Box<StateAnchor<Use>>>
// where
//     Who: ShapingWhoNoWarper + Clone + 'static + std::fmt::Debug,
//     Use: ShapingUseNoWarper + Shaping<Who> + Clone + 'static + std::fmt::Debug,
// {
//     fn shaping(&self, who: &mut StateVar<Who>) -> bool {
//         let mut is_changed = false;
//         let mut w = who.get();
//         for sa in self {
//             is_changed |= w.shaping_use_dyn(&sa.get());
//         }
//         if is_changed {
//             who.set(w);
//         }
//         is_changed
//     }
// }

impl<Who, Use> Shaping<Who> for StateAnchor<Use>
where
    Who: ShapingWhoNoWarper,
    Use: ShapingUseNoWarper + Shaping<Who> + Clone + 'static,
{
    default fn shaping(&self, who: &mut Who) -> bool {
        // log::debug!(" ============ StateAnchor get:{:?}", &u_s_e);
        who.shaping_use_dyn(&self.get())
    }
}
impl<Who, Use> EqShaping<Who> for Use
where
    Who: ShapingWhoNoWarper,
    Use: Shaping<Who> + DynPartialEq,
{
}
