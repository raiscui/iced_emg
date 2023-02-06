/*
 * @Author: Rais
 * @Date: 2021-02-10 16:20:21
 * @LastEditTime: 2023-02-04 21:11:07
 * @LastEditors: Rais
 * @Description:
 */
use emg_common::{
    better_any::{impl_tid, Tid, TidAble, TidExt},
    dyn_partial_eq::DynPartialEq,
    TypeCheckObjectSafeTid,
};
use std::{panic::Location, rc::Rc};
use tracing::{debug, error, warn};

use crate::{ShapingUse, ShapingWhoNoWarper};

#[derive(Clone)]
pub struct Shaper<Use>(Rc<dyn Fn() -> Use>);
impl<Use> Shaper<Use> {
    pub fn new<F: Fn() -> Use + 'static>(f: F) -> Self {
        Self(Rc::new(f))
    }
    #[must_use]
    pub fn get(&self) -> Use {
        (self.0)()
        // Rc::clone(&self.0)()
    }
}
impl<Use> Eq for Shaper<Use> {}
impl<Use> PartialEq for Shaper<Use> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(
            (std::ptr::addr_of!(*self.0)).cast::<u8>(),
            (std::ptr::addr_of!(*other.0)).cast::<u8>(),
        )
        // Rc::ptr_eq(&self.0, &other.0)
    }
}
// impl<Use: 'static> DynPartialEq for Shaper<Use> {
//     fn as_any(&self) -> &dyn core::any::Any {
//         self
//     }
//     fn box_eq(&self, other: &dyn core::any::Any) -> bool {
//         other.downcast_ref::<Self>().map_or(false, |a| self == a)
//     }
// }

#[derive(Clone)]
pub struct ShaperFor<'a, Who>(Rc<dyn Fn(&mut Who) -> bool + 'a>);

impl<'a, Who> ShaperFor<'a, Who> {
    pub fn new<F: Fn(&mut Who) -> bool + 'a>(f: F) -> Self {
        ShaperFor(Rc::new(f))
    }
    #[must_use]
    pub fn get(&self) -> Rc<dyn Fn(&mut Who) -> bool + 'a> {
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
//     //     ShaperFor(f)
//     // }
// }
// ────────────────────────────────────────────────────────────────────────────────
// pub trait TryRefreshFor<Who> {
//     fn try_refresh_for(&self, who: &mut Who);
// }
// impl<Who: std::fmt::Debug + 'static + Clone, Use: Sized + Clone + std::fmt::Debug + 'static>
//     TryRefreshFor<Who> for Use
// {
//     fn try_refresh_for(&self, who: &mut Who) {
//         warn!(
//             "[try_refresh_for] self:{} try  who  {}>",
//             std::any::type_name::<Self>(),
//             std::any::type_name::<Who>()
//         );
//         let w =
//             unsafe { (who as &mut dyn Any).downcast_mut_unchecked::<Box<dyn ShapingUse<Self>>>() };
//         // {
//         let xx = &mut **w;
//         warn!("w change xx");
//         xx.shaping_use(self);
//         warn!("xx shaping_use self");

//         let xxxx = (&*w as &dyn Any).downcast_ref::<Who>();
//         *who = (*xxxx.unwrap()).clone();
//         // } else {
//         warn!("try_refresh failed: use {:?} for who:{:?}", &self, &who);
//         // }
//     }
// }

pub trait ShapingUseAny {
    #[must_use]
    fn shaping_use_any(&mut self, any: &dyn Tid) -> bool;
}
impl<Who: for<'a> Tid<'a>> ShapingUseAny for Who {
    #[must_use]
    #[track_caller]
    default fn shaping_use_any(&mut self, any: &dyn Tid) -> bool {
        if let Some(same_type_as_self) = any.downcast_any_ref::<Self>() {
            debug!("default impl 成功 downcast to any Self");
            return self.shaping_use(same_type_as_self);
        }
        // ─────────────────────────────────────────────────────────────

        warn!(
            "default impl downcast to any Self 失败, self: {},\nlocation:{}",
            std::any::type_name::<Self>(),
            Location::caller()
        );

        false
    }
}
// impl
//     TryShapingUse for Rc<Use>
// {
//     fn try_refresh_for(&self, who: &mut Who) {
//         warn!(
//             "[try_refresh_for] self:{} try downcast to Rc<dyn Shaping<{}>>",
//             std::any::type_name::<Self>(),
//             std::any::type_name::<Who>()
//         );
//         let u = self.clone();
//         let any: &dyn Any = &u;
//         if let Some(u_s_e) = any.downcast_ref::<Rc<dyn Shaping<Who>>>() {
//             who.shape_of_use(&**u_s_e);
//         } else {
//             warn!("try_refresh failed: use {:?} for who:{:?}", &self, &who);
//         }
//     }
// }
// refresh

pub trait ShapingAny {
    fn shaping_any(&self, any: &mut dyn TypeCheckObjectSafeTid) -> bool;
}

pub trait ShapingDyn {
    fn shaping_dyn(&self, who: &mut dyn ShapingUse<Self>) -> bool;
}

pub trait Shaping<Who> {
    ///return : changed or not
    #[must_use]
    //TODO maybe change to Option<bool>  Some(true)->changed Some(false)->not changed  None->not impl
    fn shaping(&self, who: &mut Who) -> bool;
}
#[impl_tid]
impl<'a, Who> TidAble<'a> for Box<dyn Shaping<Who> + 'a> {}

//TODO make Result
#[cfg(not(feature = "no_default_shaping"))]
impl<Who, Use> Shaping<Who> for Use
where
    Use: Sized,
{
    #[must_use]
    default fn shaping(&self, _el: &mut Who) -> bool {
        // println!(
        //     "this is un implemented yet use ->\n{} \n shaping ->\n{}",
        //     std::any::type_name::<Use>(),
        //     std::any::type_name::<Who>()
        // );

        #[cfg(not(feature = "default_shaping_make_panic"))]
        {
            error!(
                "this is un implemented yet use ->\n{} \n shaping ->\n{}",
                std::any::type_name::<Use>(),
                std::any::type_name::<Who>()
            );
            false //not changed
        }

        #[cfg(feature = "default_shaping_make_panic")]
        panic!(
            "this is un implemented yet use ->\n{} \n shaping ->\n{}",
            std::any::type_name::<Use>(),
            std::any::type_name::<Who>()
        );
    }
}

// impl<Who> std::fmt::Debug for &dyn Shaping<Who>
// where
//     Self: std::fmt::Debug,
// {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         self.fmt(f)
//     }
// }

pub trait ShapingWithDebug<Who>: Shaping<Who> + core::fmt::Debug {}
impl<Who, Use> ShapingWithDebug<Who> for Use where Use: Shaping<Who> + core::fmt::Debug {}
// ────────────────────────────────────────────────────────────────────────────────

pub trait EqShaping<Who>: Shaping<Who> + DynPartialEq {}
// ────────────────────────────────────────────────────────────────────────────────

pub trait EqShapingWithDebug<Who>: EqShaping<Who> + core::fmt::Debug {}
// impl<Who, Use> EqShapingWithDebug<Who> for Use where Use: EqShaping<Who> + core::fmt::Debug {}
impl<Who, Use> EqShapingWithDebug<Who> for Use
where
    Who: ShapingWhoNoWarper,
    Use: EqShaping<Who> + core::fmt::Debug,
{
}
// ────────────────────────────────────────────────────────────────────────────────

impl<Who> core::cmp::Eq for dyn EqShaping<Who> {}

impl<Who> core::cmp::PartialEq for dyn EqShaping<Who> {
    fn eq(&self, other: &Self) -> bool {
        self.box_eq(other.as_any())
    }
}
impl<Who: 'static> core::cmp::PartialEq<dyn EqShaping<Who>> for Box<dyn EqShaping<Who>> {
    fn eq(&self, other: &dyn EqShaping<Who>) -> bool {
        self.box_eq(other.as_any())
    }
}
// ────────────────────────────────────────────────────────────────────────────────

impl<Who> core::cmp::Eq for dyn EqShapingWithDebug<Who> {}

impl<Who> core::cmp::PartialEq for dyn EqShapingWithDebug<Who> {
    fn eq(&self, other: &Self) -> bool {
        self.box_eq(other.as_any())
    }
}
impl<Who: 'static> core::cmp::PartialEq<dyn EqShapingWithDebug<Who>>
    for Box<dyn EqShapingWithDebug<Who>>
{
    fn eq(&self, other: &dyn EqShapingWithDebug<Who>) -> bool {
        self.box_eq(other.as_any())
    }
}

// ────────────────────────────────────────────────────────────────────────────────
// pub auto trait NotStateAnchor4Refresher {}
// impl<T> !NotStateAnchor4Refresher for StateAnchor<T> {}

// impl<Use> NotStateAnchor4Refresher for Shaper<Use> where Use: NotStateAnchor4Refresher {}

// impl<Who> PartialEq for dyn EqShaping<Who> + NotStateAnchor4Refresher {
//     fn eq(&self, other: &Self) -> bool {
//         self.box_eq(other.as_any())
//     }
// }

// impl<Who: 'static> PartialEq<dyn EqShaping<Who> + NotStateAnchor4Refresher>
//     for Box<dyn EqShaping<Who> + NotStateAnchor4Refresher>
// {
//     fn eq(&self, other: &(dyn EqShaping<Who> + NotStateAnchor4Refresher)) -> bool {
//         self.box_eq(other.as_any())
//     }
// }

// ────────────────────────────────────────────────────────────────────────────────

// ────────────────────────────────────────────────────────────────────────────────

// ────────────────────────────────────────────────────────────────────────────────

// pub trait Updater {
//     type Who;
//     fn update_it(&self, who: &mut Self::Who);
//     // where
//     //     Self: RtUpdateFor<Self::Who>;
// }

// impl<Who> Updater for ShaperFor<Who> {
//     type Who = Who;
//     fn update_it(&self, who: &mut Who) {
//         // who.update_use(self)
//         (self.0)(who);
//     }
// }
// impl<Who> Updater for Box<dyn RtUpdateFor<Who>> {
//     type Who = Who;
//     fn update_it(&self, who: &mut Who) {
//         self.shaping(who)
//     }
// }
// impl<Who> Updater for Rc<dyn RtUpdateFor<Who>> {
//     type Who = Who;
//     fn update_it(&self, who: &mut Who) {
//         self.shaping(who)
//     }
// }

#[cfg(test)]
#[allow(unused_variables)]
mod updater_test {

    // use crate::CloneState;
    use crate::ShapingUseDyn;
    use crate::{test::setup_tracing, Shaping};
    use tracing::info;
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::*;

    impl Shaping<String> for i32 {
        fn shaping(&self, el: &mut String) -> bool {
            *el = format!("{},{}", el, self);
            true
        }
    }

    use emg_state::CloneStateVar;

    use emg_state::use_state;

    #[wasm_bindgen_test]
    fn test_anchor() {
        setup_tracing();

        #[allow(unused)]
        let mut s = String::from("sss");

        let n = 99_i32;

        let mut ff = use_state(String::from("hello"));
        let ff2 = use_state(2_i32);
        let ff_w = ff2.watch();
        let ffw_vec = vec![Box::new(ff_w.clone()), Box::new(ff_w.clone())];
        ff.shaping_use_dyn(&ff2);
        ff.shaping_use_dyn(&ff_w);
        ff.shaping_use_dyn(&ffw_vec);
        ff2.shaping(&mut ff);
        info!("==== test_anchor: {}", ff.get());
        // ─────────────────────────────────────────────────────────────────

        s.shaping_use_dyn(&ff2);
        ff2.shaping(&mut s);
        info!("==== test_anchor: {}", &s);
        assert_eq!("sss,2,2", &s);
        // ─────────────────────────────────────────────────────────────────

        ff.shaping_use_dyn(&n);
        n.shaping(&mut ff);
        info!("==== test_anchor 2: {}", ff.get());
        assert_eq!("hello,2,2,2,2,2,99,99", ff.get());
        // ─────────────────────────────────────────────────────────────────

        let a = use_state(4_i32);

        ff.shaping_use_dyn(&a);
        a.shaping(&mut ff);
        info!("==== test_anchor 3: {}", ff.get());

        assert_eq!("hello,2,2,2,2,2,99,99,4,4", ff.get());
    }
    #[wasm_bindgen_test]

    fn test_shaper_for() {
        setup_tracing();

        let mut f = String::from("ccc");

        let a = ShaperFor(Rc::new(|xx: &mut String| {
            xx.push_str("ddd");
            true
        }));
        let add = ShaperFor::new(|xx: &mut String| {
            xx.push_str("ddd");
            true
        });
        a.shaping(&mut f);
        a.shaping(&mut f);
        info!("{}", &f);
        assert_eq!("cccdddddd", f)
    }
    #[wasm_bindgen_test]

    fn realtime_update() {
        setup_tracing();

        let mut f = String::from("xx");
        let a = Shaper::new(|| 99);
        a.shaping(&mut f);
        a.shaping(&mut f);
        info!("{}", &f);
        assert_eq!("xx,99,99", f);
    }
}
