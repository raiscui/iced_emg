#![feature(box_patterns)]
#![feature(specialization)]
#![feature(trait_upcasting)]
// #![feature(fn_traits)] // macro_rules! map_fn_callback_return_to_option_ms;
//
#![feature(auto_traits)]
#![feature(negative_impls)] //for event callback;

//
// ────────────────────────────────────────────────────────────────────────────────

mod g_element;
mod g_node;
mod g_tree_builder;
mod graph_program;
mod impl_refresh;
mod node_builder;
mod widget;
// ────────────────────────────────────────────────────────────────────────────────
pub use g_element::{node_ref, GElement};
pub use g_node::{EventMatchsSa, GelType, GraphMethods, GraphType, NItem, E, N};
pub use g_tree_builder::{GTreeBuilderElement, GTreeBuilderFn};
pub use graph_program::GraphProgram;
pub use node_builder::{EventCallback, EventMessage, EventNode, IntoOptionMs, NodeBuilderWidget};
pub use widget::Widget;

// ────────────────────────────────────────────────────────────────────────────────
#[cfg(all(feature = "gpu"))]
use emg_native::{renderer::RenderContext, PaintCtx};
// ────────────────────────────────────────────────────────────────────────────────
// TODO Refactor once `optin_builtin_traits` or `negative_impls`
// TODO is stable (https://github.com/seed-rs/seed/issues/391).
// --
// TODO Remove `'static` bound from all `MsU`s once `optin_builtin_traits`, `negative_impls`
// TODO or https://github.com/rust-lang/rust/issues/41875 is stable.

#[macro_export]
macro_rules! map_fn_callback_return_to_option_ms {
    ($cb_type:ty,( $( $value:ident ) , * ), $callback:expr, $panic_text:literal, $output_type:tt) => {{
        let t_type = std::any::TypeId::of::<MsU>();
        if t_type == std::any::TypeId::of::<Message>() {
            $output_type::new(move |$($value),*|->Option<Message> {
                (&mut Some($callback.call(($($value),*))) as &mut dyn std::any::Any)
                    .downcast_mut::<Option<Message>>()
                    .and_then(Option::take)
            })
        } else if t_type == std::any::TypeId::of::<Option<Message>>() {
            $output_type::new(move |$($value),*|->Option<Message>{
                (&mut $callback.call(($($value),*)) as &mut dyn std::any::Any)
                    .downcast_mut::<Option<Message>>()
                    .and_then(Option::take)
            })
        } else if t_type == std::any::TypeId::of::<()>() {
            $output_type::new(move |$($value),*|->Option<Message>{
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
    fn it_works() {}
}
