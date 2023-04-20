#![cfg(target_arch = "wasm32")]
/*
 * @Author: Rais
 * @Date: 2021-09-01 09:05:39
 * @LastEditTime: 2023-02-20 17:31:16
 * @LastEditors: Rais
 * @Description:
 */
#![feature(box_patterns)]
#![feature(specialization)]
// ────────────────────────────────────────────────────────────────────────────────
#![feature(fn_traits)]
// unboxed_closures
// unboxed_closures
// #![feature(drain_filter)]
// #![feature(convert_float_to_int)] //RafEventRecipe:  (timestamp * 1000.).trunc().to_int_unchecked::<u64>()
// #![feature(negative_impls)] // NOTE for Gid refresh
// #![feature(auto_traits)]
// // #![feature(min_specialization)] // NOTE for Gid refresh
#![feature(trait_upcasting)]
// #![feature(iter_collect_into)]
// ────────────────────────────────────────────────────────────────────────────────

mod application;
mod bus;
mod command;
// mod event;
mod g_element;
mod g_tree_builder;
mod g_tree_builder_fn_for_node_item_rc_sv;
mod hasher;
mod impl_refresh;
mod node_builder;
mod orders;
mod result;
mod sandbox;
mod settings;
mod window;
// ────────────────────────────────────────────────────────────────────────────────
mod g_node;
pub mod widget;

// ────────────────────────────────────────────────────────────────────────────────
pub use crate::result::Result;
pub use bus::Bus;
pub use command::Command;
pub use dodrio;
pub use g_element::{node_ref, GElement};
pub use g_node::node_item_rc_sv::{GelType, GraphType};
pub use g_tree_builder::GTreeBuilderElement;
pub use hasher::Hasher;
pub use settings::Settings;

pub use node_builder::{EventCallback, EventMessage, EventNode, NodeBuilderWidget};

pub use crate::application::Application;
pub use crate::executor::Executor;
// pub mod subscription;
pub use emg_futures::executor;
pub use emg_futures::futures;
pub use emg_orders::Orders;

// pub use subscription::Subscription;
// ────────────────────────────────────────────────────────────────────────────────
// pub use crate::iced_runtime::executor;
// pub use crate::iced_runtime::futures;
// pub use crate::iced_runtime::Command;
// pub use crate::iced_runtime::Executor;

// ────────────────────────────────────────────────────────────────────────────────
// TODO Refactor once `optin_builtin_traits` or `negative_impls`
// TODO is stable (https://github.com/seed-rs/seed/issues/391).
// --
// TODO Remove `'static` bound from all `MsU`s once `optin_builtin_traits`, `negative_impls`
// TODO or https://github.com/rust-lang/rust/issues/41875 is stable.
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
