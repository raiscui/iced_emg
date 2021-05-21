#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![warn(clippy::nursery)]
// #![warn(clippy::cargo)]
// #![deny(unsafe_code)]
#![feature(unboxed_closures, fn_traits, thread_local)]
// #![feature(specialization)]
#![feature(drain_filter)]
// ────────────────────────────────────────────────────────────────────────────────
#![feature(convert_float_to_int)] //RafEventRecipe:  (timestamp * 1000.).trunc().to_int_unchecked::<u64>()

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
mod animation;
mod bind_view;
mod graph_layout;
mod impl_refresh;
mod layer;
mod node_builder;
mod orders;
mod sandbox;
// ────────────────────────────────────────────────────────────────────────────────
// ────────────────────────────────────────────────────────────────────────────────
pub use runtime::Hasher;
pub mod event;
pub mod subscription;
pub mod window;

// ────────────────────────────────────────────────────────────────────────────────
pub use emg_animation::Tick;
pub use emg_orders::Orders;
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

// ────────────────────────────────────────────────────────────────────────────────
// @TODO Refactor once `optin_builtin_traits` or `negative_impls`
// @TODO is stable (https://github.com/seed-rs/seed/issues/391).
// --
// @TODO Remove `'static` bound from all `MsU`s once `optin_builtin_traits`, `negative_impls`
// @TODO or https://github.com/rust-lang/rust/issues/41875 is stable.
#[macro_export]
macro_rules! map_callback_return_to_option_ms {
    ($cb_type:ty, $callback:expr, $panic_text:literal, $output_type:tt) => {{
        let t_type = std::any::TypeId::of::<MsU>();
        if t_type == std::any::TypeId::of::<Message>() {
            $output_type::new(move |value| {
                (&mut Some($callback.call_once((value,))) as &mut dyn std::any::Any)
                    .downcast_mut::<Option<Message>>()
                    .and_then(Option::take)
            })
        } else if t_type == std::any::TypeId::of::<Option<Message>>() {
            $output_type::new(move |value| {
                (&mut $callback.call_once((value,)) as &mut dyn std::any::Any)
                    .downcast_mut::<Option<Message>>()
                    .and_then(Option::take)
            })
        } else if t_type == std::any::TypeId::of::<()>() {
            $output_type::new(move |value| {
                $callback.call_once((value,));
                None
            }) as $output_type<$cb_type>
        } else {
            panic!($panic_text);
        }
    }};
}
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
