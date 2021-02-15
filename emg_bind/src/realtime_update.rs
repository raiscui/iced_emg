/*
 * @Author: Rais
 * @Date: 2021-02-10 16:20:21
 * @LastEditTime: 2021-02-13 12:17:37
 * @LastEditors: Rais
 * @Description:
 */
use std::{cell::RefCell, rc::Rc};
pub struct RealTimeUpdater<Use>(Rc<dyn Fn() -> Use>);
impl<Use> RealTimeUpdater<Use> {
    pub fn new(f: impl Fn() -> Use + 'static) -> Self {
        RealTimeUpdater(Rc::new(f))
    }
    // pub fn new(f: Rc<dyn Fn() -> Use>) -> Self {
    //     RealTimeUpdater(f)
    // }
}
pub struct RealTimeUpdaterFor<In>(Rc<dyn Fn(&mut In)>);

impl<In> RealTimeUpdaterFor<In> {
    pub fn new(f: impl Fn(&mut In) + 'static) -> Self {
        RealTimeUpdaterFor(Rc::new(f))
    }
    // pub fn new(f: Rc<dyn Fn(&mut In)>) -> Self {
    //     RealTimeUpdaterFor(f)
    // }
}

pub trait RTUpdateFor<In> {
    fn update_for(&self, widget_like: &mut In);
}

impl<In> RTUpdateFor<In> for RealTimeUpdaterFor<In> {
    fn update_for(&self, widget_like: &mut In) {
        (self.0)(widget_like);
    }
}
impl<In, Use> RTUpdateFor<In> for RealTimeUpdater<Use>
where
    Use: RTUpdateFor<In>,
{
    fn update_for(&self, widget_like: &mut In) {
        (self.0)().update_for(widget_like);
    }
}

#[cfg(test)]
mod updater_test {
    use wasm_bindgen_test::*;

    use super::*;

    impl RTUpdateFor<String> for i32 {
        fn update_for(&self, el: &mut String) {
            *el = format!("{},{}", el, self);
        }
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
