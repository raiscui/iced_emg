// #![feature(specialization)]

/*
 * @Author: Rais
 * @Date: 2021-02-10 18:27:38
 * @LastEditTime: 2023-02-21 12:20:25
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

pub trait ShapingUseOtherShaper<Use> {
    #[must_use]
    fn shaping_use_other_shaper(&mut self, updater: &dyn Shaping<Use>) -> bool;
}

// ────────────────────────────────────────────────────────────────────────────────
#[allow(clippy::module_name_repetitions)]
pub trait ShapingUseDyn {
    #[must_use]
    fn shaping_use_dyn(&mut self, updater: &dyn Shaping<Self>) -> bool;
}
// ────────────────────────────────────────────────────────────────────────────────
// @ impl ShapeOfUse ────────────────────────────────────────────────────────────────────────────────

impl<Who> ShapingUseDyn for Who {
    // #[inline]
    #[must_use]
    default fn shaping_use_dyn(&mut self, updater: &dyn Shaping<Self>) -> bool {
        updater.shaping(self)
    }
}
// ────────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod updater_test1 {
    use std::convert::TryFrom;
    use std::rc::Rc;
    use tracing::info;

    use crate::{
        impl_shaping::ShapingUseNoWarper, test::setup_tracing, Shaper, Shaping, ShapingWhoNoWarper,
    };

    use super::ShapingUseDyn;

    // impl RtUpdateFor<String> for i32 {
    //     fn shaping(&self, el: &mut String) {
    //         *el = format!("{},{}", el, self);
    //     }
    // }
    impl ShapingWhoNoWarper for String {}
    impl ShapingUseNoWarper for String {}
    impl Shaping<Self> for String {
        fn shaping(&self, el: &mut Self) -> bool {
            *el = format!("{el},{self}");
            true
        }
    }
    impl Shaping<i32> for String {
        fn shaping(&self, el: &mut i32) -> bool {
            *el = i32::try_from(self.len()).unwrap();
            true
        }
    }

    #[test]
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

        f.shaping_use_dyn(&a);
        f.shaping_use_dyn(rca.as_ref());
        f.shaping_use_dyn(rca.as_ref());
        f.shaping_use_dyn(rc_b_string.as_ref());

        let mut n = 0;

        n.shaping_use_dyn(&f);
        f.shaping_use_dyn(&n);

        // let xxx: i16 = 2;

        info!("{}", &f);
        // log::info!("{}", &n);
        assert_eq!("xx,99,99,string..,99,99,99,string..,35", f);
    }
}
