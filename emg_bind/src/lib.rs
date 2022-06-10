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
#![feature(negative_impls)] // NOTE for Gid refresh
#![feature(min_specialization)] // NOTE for Gid refresh
#![feature(box_patterns)]
#![feature(trait_upcasting)]
#![feature(iter_collect_into)]
// #![feature(associated_type_bounds)]
// bumpalo
// #![feature(allocator_api)]
// #![feature(generic_associated_types)]

pub mod emg_web;
#[cfg(not(target_arch = "wasm32"))]
pub use iced as runtime;

#[cfg(target_arch = "wasm32")]
pub use iced_web as iced_runtime;

#[cfg(target_arch = "wasm32")]
pub use emg_web as emg_runtime;

mod g_element;
mod g_tree_builder_element;
mod g_tree_builder_fn_for_node_item;
// mod graph_store;
mod animation;
mod bind_view;
pub mod g_node;
// mod gid;
mod graph_layout;
mod impl_refresh;
mod orders;
mod sandbox;
// ────────────────────────────────────────────────────────────────────────────────
// ────────────────────────────────────────────────────────────────────────────────
// ────────────────────────────────────────────────────────────────────────────────
// pub use gid::Gid;
pub mod event;
pub mod window;

// ────────────────────────────────────────────────────────────────────────────────
pub use emg_animation::Tick;
pub use emg_orders::Orders;
// mod state_store;
// mod topo_store;
pub use bind_view::*;
pub use emg::{edge_index_no_source, Outgoing};
pub use emg_msg_macro::emg_msg;
pub use emg_runtime::*;
pub use g_element::*;
pub use g_tree_builder_element::*;
pub use graph_layout::*;
pub use sandbox::Sandbox;

const VEC_SMALL: usize = 4;

// pub use state_store::GStateStore;
// pub use state_store::G_STATE_STORE;
// pub use topo_store::use_state;
// pub use topo_store::CloneState;
// pub use topo_store::StateAccess;
// ────────────────────────────────────────────────────────────────────────────────
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
#[macro_export]
macro_rules! map_fn_callback_return_to_option_ms {
    ($cb_type:ty,( $( $value:ident ) , * ), $callback:expr, $panic_text:literal, $output_type:tt) => {{
        let t_type = std::any::TypeId::of::<MsU>();
        if t_type == std::any::TypeId::of::<Message>() {
            $output_type::new(move |$($value),*| {
                (&mut Some($callback.call(($($value),*))) as &mut dyn std::any::Any)
                    .downcast_mut::<Option<Message>>()
                    .and_then(Option::take)
            })
        } else if t_type == std::any::TypeId::of::<Option<Message>>() {
            $output_type::new(move |$($value),*| {
                (&mut $callback.call(($($value),*)) as &mut dyn std::any::Any)
                    .downcast_mut::<Option<Message>>()
                    .and_then(Option::take)
            })
        } else if t_type == std::any::TypeId::of::<()>() {
            $output_type::new(move |$($value),*| {
                $callback.call(($($value),*));
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
