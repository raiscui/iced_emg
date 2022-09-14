// #![feature(specialization)]

/*
 * @Author: Rais
 * @Date: 2021-02-10 18:27:38
 * @LastEditTime: 2022-09-14 16:36:59
 * @LastEditors: Rais
 * @Description:
 */

use crate::Shaping;

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
pub trait ShapeOfUse<Who> {
    fn shape_of_use(&mut self, updater: &dyn Shaping<Who>);
}
// ────────────────────────────────────────────────────────────────────────────────
// @ impl ShapeOfUse ────────────────────────────────────────────────────────────────────────────────

impl<Who> ShapeOfUse<Self> for Who {
    // #[inline]
    default fn shape_of_use(&mut self, updater: &dyn Shaping<Self>) {
        updater.shaping(self);
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
        impl_refresh::ShapingUseNoWarper, test::setup_tracing, Shaper, Shaping, ShapingWhoNoWarper,
    };

    use super::*;

    // impl RtUpdateFor<String> for i32 {
    //     fn shaping(&self, el: &mut String) {
    //         *el = format!("{},{}", el, self);
    //     }
    // }
    impl ShapingWhoNoWarper for String {}
    impl ShapingUseNoWarper for String {}
    impl Shaping<Self> for String {
        fn shaping(&self, el: &mut Self) {
            *el = format!("{},{}", el, self);
        }
    }
    impl Shaping<i32> for String {
        fn shaping(&self, el: &mut i32) {
            *el = i32::try_from(self.len()).unwrap();
        }
    }

    #[wasm_bindgen_test]
    fn realtime_update() {
        setup_tracing();
        let mut f = String::from("xx");
        let a = Shaper::new(|| 99);
        let b = Shaper::new(|| String::from("string.."));
        a.shaping(&mut f);
        a.shaping(&mut f);
        b.shaping(&mut f);
        let rca = Rc::new(a.clone());
        let rc_b_string = Rc::new(b);

        f.shape_of_use(&a);
        f.shape_of_use(rca.as_ref());
        f.shape_of_use(rca.as_ref());
        f.shape_of_use(rc_b_string.as_ref());

        let mut n = 0;

        n.shape_of_use(&f);
        f.shape_of_use(&n);

        // let xxx: i16 = 2;

        info!("{}", &f);
        // log::info!("{}", &n);
        assert_eq!("xx,99,99,string..,99,99,99,string..,35", f);
    }
}
