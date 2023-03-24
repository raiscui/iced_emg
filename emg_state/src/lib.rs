#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![warn(clippy::nursery)]
#![feature(min_specialization)]
#![feature(box_into_inner)]
#![feature(auto_traits)]
#![feature(negative_impls)]
#![feature(is_some_and)]
#![feature(closure_track_caller)]
// ────────────────────────────────────────────────────────────────────────────────

// #![feature(specialization)]
pub mod error;
pub mod state_lit;
pub mod use_state_impl;
pub use anchors;
pub use anchors::dict;
pub use anchors::singlethread::Anchor;
pub use anchors::singlethread::MultiAnchor as AnchorMultiAnchor;
pub use anchors::singlethread::Var;
use emg_common::GenericSize;
pub use topo;
pub use use_state_impl::reset_state;
pub use use_state_impl::state_store;
// pub use use_state_impl::state_store_with;
pub use use_state_impl::use_state;
pub use use_state_impl::CloneStateAnchor;
pub use use_state_impl::CloneStateVar;
pub use use_state_impl::DepsVarTopoKey;
pub use use_state_impl::Dict;
pub use use_state_impl::GStateStore;
pub use use_state_impl::SkipKeyCollection;
pub use use_state_impl::StateAnchor;
pub use use_state_impl::StateMultiAnchor;
pub use use_state_impl::StateTypeCheck;
pub use use_state_impl::StateVar;
pub use use_state_impl::StorageKey;
// ────────────────────────────────────────────────────────────────────────────────

impl ::core::ops::Mul<f64> for StateAnchor<GenericSize> {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self {
        self.map(move |gs: &GenericSize| gs.clone() * rhs)
    }
}
impl ::core::ops::Add for StateAnchor<GenericSize> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        (&self, &rhs).map(|gs: &GenericSize, gs2: &GenericSize| gs.clone() + gs2.clone())
    }
}
#[cfg(test)]
mod tests {
    use crate::dict;
    use std::{panic::Location, rc::Rc};

    use crate::{state_store, use_state, CloneStateVar, StateAnchor, StateMultiAnchor, StateVar};

    struct X {
        a: StateVar<i32>,
        b: StateVar<Vec<i32>>,
        c: StateVar<i32>,
    }

    // #[track_caller]
    #[topo::nested]
    fn t1_in_topo() -> StateVar<i32> {
        use_state(|| 1)
    }
    // #[track_caller]
    fn call_t1() -> StateVar<i32> {
        t1_in_topo()
    }

    #[topo::nested(slot = "&i")]
    fn t2_in_topo(i: i32) -> StateVar<i32> {
        use_state(|| 1)
    }
    #[allow(unused)]
    fn call_t2(i: i32) -> StateVar<i32> {
        t2_in_topo(i)
    }

    fn t3() -> StateVar<i32> {
        use_state(|| 1)
    }

    #[topo::nested]
    fn call_t3_in_topo() -> StateVar<i32> {
        println!("line:{}", Location::caller());

        t3()
    }
    fn call_call_t3() -> StateVar<i32> {
        call_t3_in_topo()
    }

    #[test]
    fn loop_check() {}

    #[test]
    fn topo_test() {
        let a = call_t1();
        let b = call_t1();
        println!("{:?}", a.id());
        println!("{:?}", b.id());
        assert_eq!(a, b);

        let a = t2_in_topo(1);
        let b = t2_in_topo(1);
        println!("{:?}", a.id());
        println!("{:?}", b.id());
        assert_eq!(a, b);

        let a = t3();
        let b = t3();
        println!("{:?}", a.id());
        println!("{:?}", b.id());
        assert_eq!(a, b);

        let a = call_t3_in_topo();
        let b = call_t3_in_topo();
        println!("{:?}", a.id());
        println!("{:?}", b.id());
        assert_ne!(a, b);

        let a = call_call_t3();
        let b = call_call_t3();
        println!("{:?}", a.id());
        println!("{:?}", b.id());
        assert_eq!(a, b);
    }
    #[test]
    fn it_works() {
        let _f = dict! {1=>2};
        let x = Rc::new(X {
            a: use_state(|| 0),
            b: use_state(std::vec::Vec::new),
            c: use_state(|| 0),
        });

        let new_b: StateAnchor<Vec<i32>> =
            (&x.a.watch(), &x.b.watch()).map(|a: &i32, b: &Vec<i32>| {
                b.clone()
                    .into_iter()
                    .map(|v| v + (*a))
                    .filter(|v| *v < 10)
                    .collect::<Vec<_>>()
            });
        let xx = x.clone();
        let new_b_w = new_b.map(move |l: &Vec<i32>| {
            println!("in watch ");
            xx.b.set(l.clone());
        });
        let a_w = x.a.watch().map(|_| println!(" in a watch ---"));
        let c_w = x.c.watch().map(|_| println!("in c watch--"));

        state_store()
            .borrow()
            .engine_mut()
            .mark_observed(new_b_w.anchor());
        state_store()
            .borrow()
            .engine_mut()
            .mark_observed(a_w.anchor());
        state_store()
            .borrow()
            .engine_mut()
            .mark_observed(c_w.anchor());

        println!("{:?}", x.b);
        x.b.set(vec![1, 2, 3, 4, 5, 6, 7, 8]);
        println!("{:?}", x.b);
        x.a.set(1);
        let f = x.b.watch();
        println!("w {f:?}");
        println!("w {f:?}");
        println!("w {f:?}");
        println!("w {f:?}");
        state_store().borrow().engine_mut().stabilize();
        println!("{:?}", x.b);
        state_store().borrow().engine_mut().stabilize();

        println!("{:?}", x.b);
    }
}
