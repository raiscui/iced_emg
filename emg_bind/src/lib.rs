#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![warn(clippy::nursery)]
// #![warn(clippy::cargo)]
// #![deny(unsafe_code)]
#![feature(unboxed_closures, fn_traits, thread_local)]
#![feature(specialization)]
#![feature(drain_filter)]
// ────────────────────────────────────────────────────────────────────────────────
#![feature(convert_float_to_int)] //RafEventRecipe:  (timestamp * 1000.).trunc().to_int_unchecked::<u64>()
#![feature(negative_impls)] // NOTE for Gid refresh
#![feature(auto_traits)]
// #![feature(min_specialization)] // NOTE for Gid refresh
#![feature(box_patterns)]
#![feature(trait_upcasting)]
#![feature(iter_collect_into)]
// #![feature(associated_type_bounds)]
// bumpalo
// #![feature(allocator_api)]
// #![feature(generic_associated_types)]
// ────────────────────────────────────────────────────────────────────────────────

// ────────────────────────────────────────────────────────────────────────────────

pub mod executor;

// ────────────────────────────────────────────────────────────────────────────────
pub use emg;
pub use emg_common as common;
pub use emg_common::any;
pub use emg_common::better_any;
pub use emg_common::result::Result;
pub use emg_layout as layout;
pub use emg_msg_macro::emg_msg;
pub use emg_refresh as refresh;
pub use emg_state as state;
pub use emg_state::topo;
#[cfg(target_arch = "wasm32")]
pub use emg_web as runtime;
pub use gtree::gtree;
// pub use sandbox::Sandbox;
// #[cfg(target_arch = "wasm32")]
// pub use runtime::settings;

pub use crate::runtime::widget;
#[cfg(target_arch = "wasm32")]
pub use crate::runtime::Settings;
#[cfg(target_arch = "wasm32")]
pub use crate::runtime::{GelType, GraphType};
// mod g_tree_builder_fn_for_node_item;
// mod g_tree_builder_fn_for_node_item_rc;
// mod graph_store;

// ────────────────────────────────────────────────────────────────────────────────
// mod state_store;
// mod topo_store;

const VEC_SMALL: usize = 4;

// ────────────────────────────────────────────────────────────────────────────────
// ────────────────────────────────────────────────────────────────────────────────
#[cfg(all(test, target_arch = "wasm32"))]
mod tests {

    #[allow(unused)]
    use crate::{
        common::{
            better_any::{impl_tid, tid, type_id, Tid, TidAble, TidExt},
            IdStr, TypeCheck,
        },
        layout::{add_values::*, css, styles::*, EmgEdgeItem},
        refresh::{EqRefreshFor, RefreshFor, RefreshUse, Refresher},
        runtime::{node_ref, EventCallback, EventMessage, GElement, GTreeBuilderElement},
        state::{use_state, CloneStateAnchor, CloneStateVar, StateMultiAnchor},
    };

    #[allow(unused)]
    use std::rc::Rc;
    // #[allow(unused)]
    // use GElement::*;

    #[test]
    fn xx() {
        // let f = node_ref();
    }
}
