use std::rc::Rc;

use crate::{RTUpdateFor, RealTimeUpdater};

/*
 * @Author: Rais
 * @Date: 2021-02-10 18:27:38
 * @LastEditTime: 2021-02-13 12:20:17
 * @LastEditors: Rais
 * @Description:
 */

pub trait UpdateUse {
    type T;
    fn update_use(self, updater: &impl RTUpdateFor<Self::T>) -> Self::T;
}

// impl<T> crate::UpdateUse for T {
//     // type U = Rc<dyn crate::RTUpdateFor<T>>;
//     fn update_use(mut self, updater: Rc<dyn crate::RTUpdateFor<T>>) -> T {
//         updater.update_for(&mut self);
//         self
//     }
// }
impl<S> crate::UpdateUse for S {
    type T = S;
    fn update_use(mut self, updater: &impl RTUpdateFor<Self::T>) -> Self::T {
        updater.update_for(&mut self);
        self
    }
}

#[cfg(test)]
mod updater_test1 {
    use wasm_bindgen_test::*;

    use crate::{RTUpdateFor, RealTimeUpdater, RealTimeUpdaterFor};

    use super::*;

    // impl RTUpdateFor<String> for i32 {
    //     fn update_for(&self, el: &mut String) {
    //         *el = format!("{},{}", el, self);
    //     }
    // }
    impl RTUpdateFor<String> for String {
        fn update_for(&self, el: &mut String) {
            *el = format!("{},{}", el, self);
        }
    }
    impl RTUpdateFor<i32> for String {
        fn update_for(&self, el: &mut i32) {
            *el = self.len() as i32
        }
    }

    #[wasm_bindgen_test]

    fn realtime_update() {
        console_log::init_with_level(log::Level::Debug).ok();

        let mut f = String::from("xx");
        let a = RealTimeUpdater::new(|| 99);
        let b = RealTimeUpdater::new(|| String::from("string.."));
        a.update_for(&mut f);
        a.update_for(&mut f);
        b.update_for(&mut f);
        f = f.update_use(&a);
        f = f.update_use(&b);

        let mut n = 0;

        n = n.update_use(&f);
        f = f.update_use(&n);
        let xxx: i16 = 2;

        log::info!("{}", &f);
        log::info!("{}", &n);
        assert_eq!("xx,99,99,string..,99,string..,29", f);
    }
}
