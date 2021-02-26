#![feature(unboxed_closures, fn_traits, thread_local)]
#![feature(min_specialization)]
// #![feature(specialization)]
#![feature(negative_impls)]
#![feature(auto_traits)]

#[cfg(not(target_arch = "wasm32"))]
pub use iced as runtime;

#[cfg(target_arch = "wasm32")]
pub use iced_web as runtime;
#[cfg(target_arch = "wasm32")]
pub use uuid::Uuid;

mod anchor;
mod g_tree_builder_element;
mod graph_store;
mod impl_refresh;
mod layer;
mod refresh_use;
mod refreshers;
mod state_store;
mod topo_store;

pub use anchor::*;
pub use g_tree_builder_element::*;
pub use graph_store::*;
pub use layer::Layer;
pub use refresh_use::RefreshUseFor;
pub use refreshers::RefreshFor;
pub use refreshers::Refresher;
pub use refreshers::RefresherFor;
pub use state_store::GStateStore;
pub use topo_store::use_state;
pub use topo_store::CloneState;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
