#![feature(box_patterns)]
#![feature(specialization)]
#![feature(trait_upcasting)]
//
#![feature(auto_traits)]
#![feature(negative_impls)] //for event callback;
#![feature(is_some_and)]
#![feature(iter_collect_into)]
#![feature(cell_update)]
#![feature(let_chains)]
//
// ────────────────────────────────────────────────────────────────────────────────

pub mod component;
mod error;
mod g_element;
mod g_node;
mod g_tree_builder;
mod graph_program;
mod impl_refresh;
mod node_builder;
pub mod widget;
// ────────────────────────────────────────────────────────────────────────────────
pub use g_element::{node_ref, GElement};
pub use g_node::{EventMatchsSa, GelType, GraphMethods, GraphType, NItem, E, N};

pub use g_tree_builder::{GTreeBuilderElement, GTreeBuilderFn, GTreeInit, InitdTree};
pub use graph_program::{EventAndCtx, GraphProgram};
pub use node_builder::{EventCallback, EventMessage, EventNode, IntoOptionMs, NodeBuilderWidget};
pub use widget::Widget;
pub mod graph_edit;
// ────────────────────────────────────────────────────────────────────────────────
#[cfg(all(feature = "gpu"))]
use emg_native as platform;

#[cfg(all(feature = "gpu"))]
use emg_vello as renderer;
// #[cfg(all(feature = "gpu"))]
// use emg_vello::SceneFrag;
// ────────────────────────────────────────────────────────────────────────────────
pub mod prelude {
    pub use crate::{
        component::*, error::Error as ElementError, graph_edit, node_ref, widget::*, EventCallback,
        EventMessage, GElement, GTreeBuilderElement, GelType, GraphType, InitdTree,
    };
}
pub mod gtree_macro_prelude {
    pub use crate::prelude::*;
    pub use gtree::gtree;

    pub use crate::g_tree_builder::GtreeInitCall;
    pub use emg_common::{
        better_any::{impl_tid, tid, type_id, Tid, TidAble, TidExt},
        IdStr, TypeCheck,
    };
    pub use emg_layout::{add_values::*, ccsa_macro_prelude, css, styles::*, EmgEdgeItem};
    pub use emg_shaping::{EqShaping, Shaper, Shaping, ShapingUse};
    pub use emg_state::{use_state, CloneState, CloneStateAnchor, StateMultiAnchor};
}
// ─────────────────────────────────────────────────────────────────────────────

// ─────────────────────────────────────────────────────────────────────────────

// #[macro_export]
// macro_rules! map_fn_callback_return_to_option_ms {
//     ($cb_type:ty,( $( $value:ident ) , * ), $callback:expr, $panic_text:literal, $output_type:tt) => {{
//         let t_type = std::any::TypeId::of::<MsU>();
//         if t_type == std::any::TypeId::of::<Message>() {
//             $output_type::new(move |$($value),*|->Option<Message> {
//                 (&mut Some($callback.call(($($value),*))) as &mut dyn std::any::Any)
//                     .downcast_mut::<Option<Message>>()
//                     .and_then(Option::take)
//             })
//         } else if t_type == std::any::TypeId::of::<Option<Message>>() {
//             $output_type::new(move |$($value),*|->Option<Message>{
//                 (&mut $callback.call(($($value),*)) as &mut dyn std::any::Any)
//                     .downcast_mut::<Option<Message>>()
//                     .and_then(Option::take)
//             })
//         } else if t_type == std::any::TypeId::of::<()>() {
//             $output_type::new(move |$($value),*|->Option<Message>{
//                 $callback.call(($($value),*));
//                 None
//             }) as $output_type<$cb_type>
//         } else {
//             panic!($panic_text);
//         }
//     }};
// }

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {}
}
