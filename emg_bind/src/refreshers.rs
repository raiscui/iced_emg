/*
 * @Author: Rais
 * @Date: 2021-02-10 16:20:21
 * @LastEditTime: 2021-02-19 16:14:07
 * @LastEditors: Rais
 * @Description:
 */
use std::{cell::RefCell, rc::Rc};

#[derive(Clone)]
pub struct Refresher<Use>(Rc<dyn Fn() -> Use>);
impl<Use> Refresher<Use> {
    pub fn new(f: impl Fn() -> Use + 'static) -> Self {
        Refresher(Rc::new(f))
    }
    // pub fn new(f: Rc<dyn Fn() -> Use>) -> Self {
    //     Refresher(f)
    // }
}
#[derive(Clone)]
pub struct RefresherFor<Who>(Rc<dyn Fn(&mut Who)>);

impl<Who> RefresherFor<Who> {
    pub fn new(f: impl Fn(&mut Who) + 'static) -> Self {
        RefresherFor(Rc::new(f))
    }
    // pub fn new(f: Rc<dyn Fn(&mut In)>) -> Self {
    //     RefresherFor(f)
    // }
}

// refresh
pub trait RefreshFor<Who> {
    fn refresh_for(&self, widget_like: &mut Who);
}

impl<Who> RefreshFor<Who> for RefresherFor<Who> {
    fn refresh_for(&self, widget_like: &mut Who) {
        (self.0)(widget_like);
    }
}
impl<Who, Use> RefreshFor<Who> for Refresher<Use>
where
    Use: RefreshFor<Who>,
{
    fn refresh_for(&self, widget_like: &mut Who) {
        (self.0)().refresh_for(widget_like);
    }
}

// pub trait Updater {
//     type Who;
//     fn update_it(&self, widget_like: &mut Self::Who);
//     // where
//     //     Self: RtUpdateFor<Self::Who>;
// }

// impl<Who> Updater for RefresherFor<Who> {
//     type Who = Who;
//     fn update_it(&self, widget_like: &mut Who) {
//         // widget_like.update_use(self)
//         (self.0)(widget_like);
//     }
// }
// impl<Who> Updater for Box<dyn RtUpdateFor<Who>> {
//     type Who = Who;
//     fn update_it(&self, widget_like: &mut Who) {
//         self.refresh_for(widget_like)
//     }
// }
// impl<Who> Updater for Rc<dyn RtUpdateFor<Who>> {
//     type Who = Who;
//     fn update_it(&self, widget_like: &mut Who) {
//         self.refresh_for(widget_like)
//     }
// }

#[cfg(test)]
mod updater_test {
    use wasm_bindgen_test::*;

    use super::*;

    impl RefreshFor<String> for i32 {
        fn refresh_for(&self, el: &mut String) {
            *el = format!("{},{}", el, self);
        }
    }

    #[wasm_bindgen_test]

    fn updater() {
        let a = RefresherFor(Rc::new(|xx: &mut String| xx.push_str("ddd")));

        // let f: Vec<Rc<dyn Updater>> = vec![Rc::new(a)];
    }
    #[wasm_bindgen_test]

    fn realtime_update_in() {
        console_log::init_with_level(log::Level::Debug).ok();

        let mut f = String::from("ccc");

        let a = RefresherFor(Rc::new(|xx: &mut String| xx.push_str("ddd")));
        let add = RefresherFor::new(|xx: &mut String| xx.push_str("ddd"));
        a.refresh_for(&mut f);
        a.refresh_for(&mut f);
        log::info!("{}", &f);
        assert_eq!("cccdddddd", f)
    }
    #[wasm_bindgen_test]

    fn realtime_update() {
        console_log::init_with_level(log::Level::Debug).ok();

        let mut f = String::from("xx");
        let a = Refresher(Rc::new(|| 99));
        a.refresh_for(&mut f);
        a.refresh_for(&mut f);
        log::info!("{}", &f);
        assert_eq!("xx,99,99", f);
    }
}
