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

pub use uuid::Uuid;

mod application;
mod button;
mod g_element;
mod g_tree_builder_element;
// mod graph_store;
mod bind_view;
mod graph_layout;
mod impl_refresh;
mod layer;
mod node_builder;
mod sandbox;
// ────────────────────────────────────────────────────────────────────────────────
pub use runtime::Hasher;
pub mod event;
pub mod subscription;
pub mod window;
// ────────────────────────────────────────────────────────────────────────────────

// mod state_store;
// mod topo_store;
pub use application::{Application, Command, Element};
pub use bind_view::*;
pub use button::Button;
pub use emg::{edge_index_no_source, Outgoing};
pub use g_element::*;
pub use g_tree_builder_element::*;
pub use graph_layout::*;
pub use layer::Layer;
pub use node_builder::*;
pub use sandbox::Sandbox;
pub use subscription::Subscription;
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
