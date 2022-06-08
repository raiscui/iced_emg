/*
 * @Author: Rais
 * @Date: 2021-02-10 16:20:21
 * @LastEditTime: 2022-06-07 20:43:20
 * @LastEditors: Rais
 * @Description:
 */
use dyn_partial_eq::DynPartialEq;
use std::rc::Rc;

#[derive(Clone)]
pub struct Refresher<'a, Use>(Rc<dyn Fn() -> Use + 'a>);
impl<'a, Use> Refresher<'a, Use> {
    pub fn new<F: Fn() -> Use + 'a>(f: F) -> Self {
        Refresher(Rc::new(f))
    }
    #[must_use]
    pub fn get(&self) -> Use {
        (self.0)()
        // Rc::clone(&self.0)()
    }
}
impl<'a, Use> Eq for Refresher<'a, Use> {}
impl<'a, Use> PartialEq for Refresher<'a, Use> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(
            (std::ptr::addr_of!(*self.0)).cast::<u8>(),
            (std::ptr::addr_of!(*other.0)).cast::<u8>(),
        )
    }
}
impl<Use: 'static> DynPartialEq for Refresher<'static, Use> {
    fn as_any(&self) -> &dyn core::any::Any {
        self
    }
    fn box_eq(&self, other: &dyn core::any::Any) -> bool {
        other.downcast_ref::<Self>().map_or(false, |a| self == a)
    }
}

#[derive(Clone)]
pub struct RefresherFor<'a, Who>(Rc<dyn Fn(&mut Who) + 'a>);

impl<'a, Who> RefresherFor<'a, Who> {
    pub fn new<F: Fn(&mut Who) + 'a>(f: F) -> Self {
        RefresherFor(Rc::new(f))
    }
    #[must_use]
    pub fn get(&self) -> Rc<dyn Fn(&mut Who) + 'a> {
        Rc::clone(&self.0)
    }
}
// #[derive(Clone)]
// pub struct RefresherForSelf<SelfEl, Use>(Rc<dyn Fn(&mut SelfEl) -> Use>);

// impl<SelfEl, Use> RefresherForSelf<SelfEl, Use> {
//     pub fn new(f: impl Fn(&mut SelfEl) -> Use + 'static) -> Self {
//         RefresherForSelf(Rc::new(f))
//     }
//     pub fn get(&self, el: &mut SelfEl) -> Use {
//         (self.0)(el)
//     }
//     // pub fn new(f: Rc<dyn Fn(&mut In)>) -> Self {
//     //     RefresherFor(f)
//     // }
// }
// ────────────────────────────────────────────────────────────────────────────────

// refresh
pub trait RefreshFor<Who> {
    fn refresh_for(&self, who: &mut Who);
}
pub trait EqRefreshFor<Who>: RefreshFor<Who> + DynPartialEq {}

impl<Who> core::cmp::Eq for dyn EqRefreshFor<Who> + '_ {}

impl<Who> core::cmp::PartialEq for dyn EqRefreshFor<Who> + '_ {
    fn eq(&self, other: &Self) -> bool {
        self.box_eq(other.as_any())
    }
}
impl<Who> core::cmp::PartialEq<dyn EqRefreshFor<Who> + '_> for Box<dyn EqRefreshFor<Who> + '_> {
    fn eq(&self, other: &dyn EqRefreshFor<Who>) -> bool {
        self.box_eq(other.as_any())
    }
}
// ────────────────────────────────────────────────────────────────────────────────

// ────────────────────────────────────────────────────────────────────────────────

// ────────────────────────────────────────────────────────────────────────────────

// ────────────────────────────────────────────────────────────────────────────────

// pub trait Updater {
//     type Who;
//     fn update_it(&self, who: &mut Self::Who);
//     // where
//     //     Self: RtUpdateFor<Self::Who>;
// }

// impl<Who> Updater for RefresherFor<Who> {
//     type Who = Who;
//     fn update_it(&self, who: &mut Who) {
//         // who.update_use(self)
//         (self.0)(who);
//     }
// }
// impl<Who> Updater for Box<dyn RtUpdateFor<Who>> {
//     type Who = Who;
//     fn update_it(&self, who: &mut Who) {
//         self.refresh_for(who)
//     }
// }
// impl<Who> Updater for Rc<dyn RtUpdateFor<Who>> {
//     type Who = Who;
//     fn update_it(&self, who: &mut Who) {
//         self.refresh_for(who)
//     }
// }

#[cfg(test)]
#[allow(unused_variables)]
mod updater_test {

    // use crate::CloneState;
    use crate::RefreshForUse;
    use crate::{test::setup_tracing, RefreshFor};
    use tracing::info;
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::*;

    impl RefreshFor<String> for i32 {
        fn refresh_for(&self, el: &mut String) {
            *el = format!("{},{}", el, self);
        }
    }

    use emg_state::CloneStateVar;

    use emg_state::use_state;

    #[wasm_bindgen_test]
    fn test_anchor() {
        setup_tracing();

        #[allow(unused)]
        let mut s = String::from("sss");

        let n = 99_i32;

        let mut ff = use_state(String::from("hello"));
        let ff2 = use_state(2_i32);
        let ff_w = ff2.watch();
        let ffw_vec = vec![Box::new(ff_w.clone()), Box::new(ff_w.clone())];
        ff.refresh_for_use(&ff2);
        ff.refresh_for_use(&ff_w);
        ff.refresh_for_use(&ffw_vec);
        ff2.refresh_for(&mut ff);
        info!("==== test_anchor: {}", ff.get());
        // ─────────────────────────────────────────────────────────────────

        s.refresh_for_use(&ff2);
        ff2.refresh_for(&mut s);
        info!("==== test_anchor: {}", &s);
        assert_eq!("sss,2,2", &s);
        // ─────────────────────────────────────────────────────────────────

        ff.refresh_for_use(&n);
        n.refresh_for(&mut ff);
        info!("==== test_anchor 2: {}", ff.get());
        assert_eq!("hello,2,2,2,2,2,99,99", ff.get());
        // ─────────────────────────────────────────────────────────────────

        let a = use_state(4_i32);

        ff.refresh_for_use(&a);
        a.refresh_for(&mut ff);
        info!("==== test_anchor 3: {}", ff.get());

        assert_eq!("hello,2,2,2,2,2,99,99,4,4", ff.get());
    }
    #[wasm_bindgen_test]

    fn test_refresher_for() {
        setup_tracing();

        let mut f = String::from("ccc");

        let a = RefresherFor(Rc::new(|xx: &mut String| xx.push_str("ddd")));
        let add = RefresherFor::new(|xx: &mut String| xx.push_str("ddd"));
        a.refresh_for(&mut f);
        a.refresh_for(&mut f);
        info!("{}", &f);
        assert_eq!("cccdddddd", f)
    }
    #[wasm_bindgen_test]

    fn realtime_update() {
        setup_tracing();

        let mut f = String::from("xx");
        let a = Refresher::new(|| 99);
        a.refresh_for(&mut f);
        a.refresh_for(&mut f);
        info!("{}", &f);
        assert_eq!("xx,99,99", f);
    }
}
