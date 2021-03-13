/*
 * @Author: Rais
 * @Date: 2021-02-10 16:20:21
 * @LastEditTime: 2021-03-12 18:11:12
 * @LastEditors: Rais
 * @Description:
 */
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

    use crate::RefreshFor;
    use crate::RefreshUseFor;
    use lazy_static::__Deref;
    use wasm_bindgen_test::*;

    use super::*;

    impl RefreshFor<String> for i32 {
        fn refresh_for(&self, el: &mut String) {
            *el = format!("{},{}", el, self);
        }
    }

    #[wasm_bindgen_test]

    fn test_anchor() {
        console_log::init_with_level(log::Level::Debug).ok();

        #[allow(unused)]
        use anchors::singlethread::*;

        crate::ENGINE.with(|_e| {
            log::info!("============= get engine");
        });
        let mut s = String::from("sss");

        let n = 99i32;

        let mut ff = Var::new(String::from("hello"));
        let ff2 = Var::new(2i32);
        ff.refresh_use(&ff2);
        ff2.refresh_for(&mut ff);
        log::info!("==== test_anchor: {}", ff.get().deref());
        // ─────────────────────────────────────────────────────────────────

        s.refresh_use(&ff2);
        ff2.refresh_for(&mut s);
        log::info!("==== test_anchor: {}", &s);
        assert_eq!("sss,2,2", &s);
        // ─────────────────────────────────────────────────────────────────

        ff.refresh_use(&n);
        n.refresh_for(&mut ff);
        log::info!("==== test_anchor: {}", ff.get().deref());
        assert_eq!("hello,2,2,99,99", ff.get().deref());
        // ─────────────────────────────────────────────────────────────────

        let a = Var::new(4i32);

        ff.refresh_use(&a);
        a.refresh_for(&mut ff);
        log::info!("==== test_anchor: {}", ff.get().deref());

        assert_eq!("hello,2,2,99,99,4,4", ff.get().deref());
    }
    #[wasm_bindgen_test]

    fn test_refresher_for() {
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
        let a = Refresher::new(|| 99);
        a.refresh_for(&mut f);
        a.refresh_for(&mut f);
        log::info!("{}", &f);
        assert_eq!("xx,99,99", f);
    }
}
