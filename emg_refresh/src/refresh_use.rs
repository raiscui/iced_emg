// #![feature(specialization)]

/*
 * @Author: Rais
 * @Date: 2021-02-10 18:27:38
 * @LastEditTime: 2021-05-22 09:41:39
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
    use std::convert::TryFrom;
    use std::rc::Rc;
    use tracing::info;
    use wasm_bindgen_test::wasm_bindgen_test;

    use crate::{
        impl_refresh::RefreshUseNoWarper, test::setup_tracing, RefreshFor, RefreshWhoNoWarper,
        Refresher,
    };

    use super::*;

    // impl RtUpdateFor<String> for i32 {
    //     fn refresh_for(&self, el: &mut String) {
    //         *el = format!("{},{}", el, self);
    //     }
    // }
    impl RefreshWhoNoWarper for String {}
    impl RefreshUseNoWarper for String {}
    impl RefreshFor<Self> for String {
        fn refresh_for(&self, el: &mut Self) {
            *el = format!("{},{}", el, self);
        }
    }
    impl RefreshFor<i32> for String {
        fn refresh_for(&self, el: &mut i32) {
            *el = i32::try_from(self.len()).unwrap();
        }
    }

    #[wasm_bindgen_test]
    fn realtime_update() {
        setup_tracing();
        let mut f = String::from("xx");
        let a = Refresher::new(|| 99);
        let b = Refresher::new(|| String::from("string.."));
        a.refresh_for(&mut f);
        a.refresh_for(&mut f);
        b.refresh_for(&mut f);
        let rca = Rc::new(a.clone());
        let rc_b_string = Rc::new(b);
        f.refresh_use(&a);
        f.refresh_use(rca.as_ref());
        f.refresh_use(rca.as_ref());
        f.refresh_use(rc_b_string.as_ref());

        let mut n = 0;

        n.refresh_use(&f);
        f.refresh_use(&n);

        // let xxx: i16 = 2;

        info!("{}", &f);
        // log::info!("{}", &n);
        assert_eq!("xx,99,99,string..,99,99,99,string..,35", f);
    }
}
