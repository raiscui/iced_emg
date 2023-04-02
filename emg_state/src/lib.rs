#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![warn(clippy::nursery)]
#![feature(specialization)]
#![feature(box_into_inner)]
#![feature(auto_traits)]
#![feature(negative_impls)]
#![feature(is_some_and)]
#![feature(closure_track_caller)]
#![feature(iter_collect_into)]
// ────────────────────────────────────────────────────────────────────────────────
use emg_common::GenericSize;
use general_struct::TopoKey;

// ─────────────────────────────────────────────────────────────────────────────

// #![feature(specialization)]
pub(crate) mod g_store;

pub mod error;
pub mod general_fns;
pub mod general_struct;
pub mod general_traits;
pub mod state_anchor;
pub mod state_lit;
pub mod state_var;
pub mod state_voa;
pub mod test_sv;
pub use anchors;
pub use anchors::collections::ord_map_methods::Dict;
pub use anchors::dict;
pub use anchors::singlethread::Anchor;
pub use anchors::singlethread::Engine;
pub use anchors::singlethread::MultiAnchor as AnchorMultiAnchor;
pub use anchors::singlethread::Var;
pub use g_store::DepsVarTopoKey;
pub use g_store::GStateStore;
pub use g_store::SkipKeyCollection;
pub use general_fns::state_store;
pub use general_struct::StorageKey;
pub use general_traits::CloneState;
pub use general_traits::CloneStateAnchor;
pub use general_traits::StateTypeCheck;
pub use state_anchor::StateAnchor;
pub use state_anchor::StateMultiAnchor;
pub use state_var::use_state;
pub use state_var::StateVar;
pub use state_voa::use_state_voa;
pub use state_voa::StateVOA;
pub use topo;

// ────────────────────────────────────────────────────────────────────────────────
//TODO use this replace StorageKey for b a callback fn
#[derive(Clone, Copy, Eq, PartialEq, Debug, Hash)]
pub struct CallbackFnStorageKey(TopoKey);
// ─────────────────────────────────────────────────────────────────────────────

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
    use crate::{dict, general_traits::StateFn, state_var::use_state};
    use std::{panic::Location, rc::Rc};

    use crate::{state_store, CloneState, StateAnchor, StateMultiAnchor, StateVar};

    struct X {
        a: StateVar<i32>,
        b: StateVar<Vec<i32>>,
        c: StateVar<i32>,
    }

    #[topo::nested]
    fn t1_in_topo() -> StateVar<i32> {
        use_state(|| 1)
    }

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
