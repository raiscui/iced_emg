#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![warn(clippy::nursery)]
#![feature(min_specialization)]
// #![feature(specialization)]
pub mod use_state_impl;
pub use anchors::singlethread::Anchor;
pub use anchors::singlethread::MultiAnchor as AnchorMultiAnchor;
pub use topo;
pub use use_state_impl::state_store;
pub use use_state_impl::state_store_with;
pub use use_state_impl::use_state;
pub use use_state_impl::CloneStateAnchor;
pub use use_state_impl::CloneStateVar;
pub use use_state_impl::Dict;
pub use use_state_impl::GStateStore;
pub use use_state_impl::StateAnchor;
pub use use_state_impl::StateMultiAnchor;
pub use use_state_impl::StateVar;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let x = 2 + 2;
        assert_eq!(x, 4);
    }
}
