#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![warn(clippy::nursery)]
// #![warn(clippy::cargo)]
// #![deny(unsafe_code)]
#![feature(unboxed_closures, fn_traits, thread_local)]
// #![feature(specialization)]
#![feature(drain_filter)]

// bumpalo
// #![feature(allocator_api)]
// #![feature(generic_associated_types)]

#[cfg(not(target_arch = "wasm32"))]
pub use iced as runtime;

#[cfg(target_arch = "wasm32")]
pub use iced_web as runtime;
#[cfg(target_arch = "wasm32")]
pub use uuid::Uuid;

mod application;
mod button;
mod g_element;
mod g_tree_builder_element;
// mod graph_store;
mod emg_impl;
mod impl_refresh;
mod layer;
mod node_builder;
mod sandbox;
// mod state_store;
// mod topo_store;

pub use application::{Application, Command, Element, Subscription};
pub use button::Button;
pub use emg_impl::*;
pub use g_element::*;
pub use g_tree_builder_element::*;
pub use layer::Layer;
pub use node_builder::*;

pub use sandbox::Sandbox;
// pub use state_store::GStateStore;
// pub use state_store::G_STATE_STORE;
// pub use topo_store::use_state;
// pub use topo_store::CloneState;
// pub use topo_store::StateAccess;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
