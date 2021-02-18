use std::rc::Rc;

use crate::{RTUpdateFor, RealTimeUpdater};

/*
 * @Author: Rais
 * @Date: 2021-02-10 18:27:38
 * @LastEditTime: 2021-02-18 15:57:56
 * @LastEditors: Rais
 * @Description:
 */

// pub trait UpdateUse {
//     type Who;
//     fn update_use(&mut self, updater: &dyn Updater<Who = Self::Who>);
// }

// // impl<T> crate::UpdateUse for T {
// //     // type U = Rc<dyn crate::RTUpdateFor<T>>;
// //     fn update_use(mut self, updater: Rc<dyn crate::RTUpdateFor<T>>) -> T {
// //         updater.update_for(&mut self);
// //         self
// //     }
// // }
// impl<S> UpdateUse for S {
//     type Who = S;
//     default fn update_use(&mut self, updater: &dyn Updater<Who = S>) {
//         updater.update_it(self);
//         // self
//     }
// }

pub trait UpdateUse<Who> {
    fn update_use(&mut self, updater: &dyn RTUpdateFor<Who>);
}

struct SaveTest {
    ss: Vec<Rc<dyn UpdateUse<i32>>>,
}

impl<Who> UpdateUse<Who> for Who {
    // type Who = S;
    default fn update_use(&mut self, updater: &dyn RTUpdateFor<Who>) {
        updater.update_for(self);
        // self
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
        let rca = Rc::new(a) as Rc<dyn crate::RTUpdateFor<String>>;
        let rcb = Rc::new(b) as Rc<dyn crate::RTUpdateFor<String>>;
        f.update_use(rca.as_ref());
        f.update_use(rca.as_ref());
        f.update_use(rcb.as_ref());

        // let mut n = 0;

        // n.update_use(&f);
        // f.update_use(&n);

        // let xxx: i16 = 2;

        log::info!("{}", &f);
        // log::info!("{}", &n);
        // assert_eq!("xx,99,99,string..,99,string..,29", f);
    }
}
