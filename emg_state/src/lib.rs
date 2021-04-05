#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![warn(clippy::nursery)]

pub mod use_state_impl;
pub use anchors::singlethread::MultiAnchor;
pub use topo;
pub use use_state_impl::use_state;
pub use use_state_impl::CloneStateAnchor;
pub use use_state_impl::CloneStateVar;
pub use use_state_impl::StateAnchor;
pub use use_state_impl::StateMultiAnchor;
pub use use_state_impl::StateVar;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
