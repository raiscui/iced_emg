// #![feature(specialization)]

/*
 * @Author: Rais
 * @Date: 2021-02-10 18:27:38
 * @LastEditTime: 2021-03-23 16:57:35
 * @LastEditors: Rais
 * @Description:
 */

use crate::RefreshFor;

// ────────────────────────────────────────────────────────────────────────────────

// pub trait UpdateUse {
//     type Who;
//     fn update_use(&mut self, updater: &dyn Updater<Who = Self::Who>);
// }

// impl<S> UpdateUse for S {
//     type Who = S;
//     default fn update_use(&mut self, updater: &dyn Updater<Who = S>) {
//         updater.update_it(self);
//         // self
//     }
// }

// ────────────────────────────────────────────────────────────────────────────────
#[allow(clippy::module_name_repetitions)]
pub trait RefreshUseFor<Who> {
    fn refresh_use(&mut self, updater: &dyn RefreshFor<Who>);
}
// ────────────────────────────────────────────────────────────────────────────────
// @ impl RefreshUseFor────────────────────────────────────────────────────────────────────────────────

impl<Who> RefreshUseFor<Self> for Who {
    #[inline]
    default fn refresh_use(&mut self, updater: &dyn RefreshFor<Self>) {
        updater.refresh_for(self);
    }
}
// ────────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod updater_test1 {
    use std::rc::Rc;

    use wasm_bindgen_test::*;

    use crate::{impl_refresh::RefreshUseNoWarper, RefreshFor, RefreshWhoNoWarper, Refresher};

    use super::*;

    // impl RtUpdateFor<String> for i32 {
    //     fn refresh_for(&self, el: &mut String) {
    //         *el = format!("{},{}", el, self);
    //     }
    // }
    impl RefreshWhoNoWarper for String {}
    impl RefreshUseNoWarper for String {}
    impl RefreshFor<String> for String {
        fn refresh_for(&self, el: &mut String) {
            *el = format!("{},{}", el, self);
        }
    }
    impl RefreshFor<i32> for String {
        fn refresh_for(&self, el: &mut i32) {
            *el = self.len() as i32
        }
    }

    #[wasm_bindgen_test]

    fn realtime_update() {
        console_log::init_with_level(log::Level::Debug).ok();

        let mut f = String::from("xx");
        let a = Refresher::new(|| 99);
        let b = Refresher::new(|| String::from("string.."));
        a.refresh_for(&mut f);
        a.refresh_for(&mut f);
        b.refresh_for(&mut f);
        let rca = Rc::new(a.clone());
        let rcb = Rc::new(b);
        f.refresh_use(&a);
        f.refresh_use(rca.as_ref());
        f.refresh_use(rca.as_ref());
        f.refresh_use(rcb.as_ref());

        let mut n = 0;

        n.refresh_use(&f);
        f.refresh_use(&n);

        // let xxx: i16 = 2;

        log::info!("{}", &f);
        // log::info!("{}", &n);
        assert_eq!("xx,99,99,string..,99,99,99,string..,35", f);
    }
}
