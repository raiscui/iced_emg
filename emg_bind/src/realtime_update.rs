/*
 * @Author: Rais
 * @Date: 2021-02-10 16:20:21
 * @LastEditTime: 2021-02-18 18:57:27
 * @LastEditors: Rais
 * @Description:
 */
use std::{cell::RefCell, rc::Rc};

use crate::UpdateUse;
pub struct RealTimeUpdater<Use>(Rc<dyn Fn() -> Use>);
impl<Use> RealTimeUpdater<Use> {
    pub fn new(f: impl Fn() -> Use + 'static) -> Self {
        RealTimeUpdater(Rc::new(f))
    }
    // pub fn new(f: Rc<dyn Fn() -> Use>) -> Self {
    //     RealTimeUpdater(f)
    // }
}
pub struct RealTimeUpdaterFor<Who>(Rc<dyn Fn(&mut Who)>);

impl<Who> RealTimeUpdaterFor<Who> {
    pub fn new(f: impl Fn(&mut Who) + 'static) -> Self {
        RealTimeUpdaterFor(Rc::new(f))
    }
    // pub fn new(f: Rc<dyn Fn(&mut In)>) -> Self {
    //     RealTimeUpdaterFor(f)
    // }
}

pub trait RtUpdateFor<Who> {
    fn update_for(&self, widget_like: &mut Who);
}

impl<Who> RtUpdateFor<Who> for RealTimeUpdaterFor<Who> {
    fn update_for(&self, widget_like: &mut Who) {
        (self.0)(widget_like);
    }
}
impl<Who, Use> RtUpdateFor<Who> for RealTimeUpdater<Use>
where
    Use: RtUpdateFor<Who>,
{
    fn update_for(&self, widget_like: &mut Who) {
        (self.0)().update_for(widget_like);
    }
}

// pub trait Updater {
//     type Who;
//     fn update_it(&self, widget_like: &mut Self::Who);
//     // where
//     //     Self: RtUpdateFor<Self::Who>;
// }

// impl<Who> Updater for RealTimeUpdaterFor<Who> {
//     type Who = Who;
//     fn update_it(&self, widget_like: &mut Who) {
//         // widget_like.update_use(self)
//         (self.0)(widget_like);
//     }
// }
// impl<Who> Updater for Box<dyn RtUpdateFor<Who>> {
//     type Who = Who;
//     fn update_it(&self, widget_like: &mut Who) {
//         self.update_for(widget_like)
//     }
// }
// impl<Who> Updater for Rc<dyn RtUpdateFor<Who>> {
//     type Who = Who;
//     fn update_it(&self, widget_like: &mut Who) {
//         self.update_for(widget_like)
//     }
// }

#[cfg(test)]
mod updater_test {
    use wasm_bindgen_test::*;

    use super::*;

    impl RtUpdateFor<String> for i32 {
        fn update_for(&self, el: &mut String) {
            *el = format!("{},{}", el, self);
        }
    }

    #[wasm_bindgen_test]

    fn updater() {
        let a = RealTimeUpdaterFor(Rc::new(|xx: &mut String| xx.push_str("ddd")));

        // let f: Vec<Rc<dyn Updater>> = vec![Rc::new(a)];
    }
    #[wasm_bindgen_test]

    fn realtime_update_in() {
        console_log::init_with_level(log::Level::Debug).ok();

        let mut f = String::from("ccc");

        let a = RealTimeUpdaterFor(Rc::new(|xx: &mut String| xx.push_str("ddd")));
        let add = RealTimeUpdaterFor::new(|xx: &mut String| xx.push_str("ddd"));
        a.update_for(&mut f);
        a.update_for(&mut f);
        log::info!("{}", &f);
        assert_eq!("cccdddddd", f)
    }
    #[wasm_bindgen_test]

    fn realtime_update() {
        console_log::init_with_level(log::Level::Debug).ok();

        let mut f = String::from("xx");
        let a = RealTimeUpdater(Rc::new(|| 99));
        a.update_for(&mut f);
        a.update_for(&mut f);
        log::info!("{}", &f);
        assert_eq!("xx,99,99", f);
    }
}
