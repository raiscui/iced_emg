#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![warn(clippy::nursery)]
#![feature(min_specialization)]
#![feature(box_into_inner)]
// ────────────────────────────────────────────────────────────────────────────────
// #![feature(auto_traits)]
// #![feature(negative_impls)]
// ────────────────────────────────────────────────────────────────────────────────

// #![feature(specialization)]
pub mod use_state_impl;
pub use anchors::singlethread::Anchor;
pub use anchors::singlethread::MultiAnchor as AnchorMultiAnchor;
use emg_core::GenericSize;
pub use topo;
pub use use_state_impl::state_store;
pub use use_state_impl::state_store_with;
pub use use_state_impl::use_state;
pub use use_state_impl::CloneStateAnchor;
pub use use_state_impl::CloneStateVar;
pub use use_state_impl::Dict;
pub use use_state_impl::GStateStore;
pub use use_state_impl::SkipKeyCollection;
pub use use_state_impl::StateAnchor;
pub use use_state_impl::StateMultiAnchor;
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
    use std::rc::Rc;

    use crate::{state_store, use_state, CloneStateVar, StateAnchor, StateMultiAnchor, StateVar};

    struct X {
        a: StateVar<i32>,
        b: StateVar<Vec<i32>>,
        c: StateVar<i32>,
    }
    #[test]
    fn it_works() {
        let x = Rc::new(X {
            a: use_state(0),
            b: use_state(vec![]),
            c: use_state(0),
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
        println!("w {:?}", f);
        println!("w {:?}", f);
        println!("w {:?}", f);
        println!("w {:?}", f);
        state_store().borrow().engine_mut().stabilize();
        println!("{:?}", x.b);
        state_store().borrow().engine_mut().stabilize();

        println!("{:?}", x.b);
    }
}
