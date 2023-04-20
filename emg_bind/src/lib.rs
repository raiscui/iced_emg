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
#![feature(associated_type_defaults)]

// #![feature(associated_type_bounds)]
// bumpalo
// #![feature(allocator_api)]
// ────────────────────────────────────────────────────────────────────────────────
mod application;
mod error;
mod result;
mod sandbox;
mod window;

// ────────────────────────────────────────────────────────────────────────────────
pub mod element;
pub mod executor;
pub mod settings;
pub use emg_element::graph_edit;

// ────────────────────────────────────────────────────────────────────────────────
#[cfg(all(feature = "gpu", feature = "wasm32"))]
compile_error!("current no support");

#[cfg(all(feature = "gpu"))]
// pub use emg_piet_gpu as renderer;
pub use emg_vello as renderer;
#[cfg(all(feature = "gpu"))]
pub use emg_winit as runtime;
// ────────────────────────────────────────────────────────────────────────────────

// ────────────────────────────────────────────────────────────────────────────────
#[cfg(target_arch = "wasm32")]
pub use crate::runtime::widget;
#[cfg(target_arch = "wasm32")]
pub use crate::runtime::Settings;
#[cfg(target_arch = "wasm32")]
pub use crate::runtime::{GelType, GraphType};
#[cfg(target_arch = "wasm32")]
pub use emg_web as runtime;
// ────────────────────────────────────────────────────────────────────────────────
pub use application::Application;
pub use emg;
pub use emg_common as common;
pub use emg_common::any;
pub use emg_common::better_any;
pub use emg_common::mouse;
pub use emg_layout as layout;
pub use emg_msg_macro::emg_msg;
pub use emg_orders::Orders;
pub use emg_shaping as shaping;
pub use emg_state as state;
pub use emg_state::topo;
pub use error::Error;
pub use executor::Executor;
pub use renderer::Renderer;
pub use result::Result;
pub use runtime::{futures, Command};
pub use sandbox::Sandbox;
pub use settings::Settings;

// ─────────────────────────────────────────────────────────────────────────────
pub mod trait_prelude {
    pub use crate::Orders;
}
pub mod gtree_macro_prelude {
    pub use crate::{common::mouse::CLICK, element::gtree_macro_prelude::*};
}
pub mod emg_msg_macro_prelude {
    pub use crate::common::{any, better_any};
    pub use crate::emg_msg;
}
// ────────────────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod test1 {
    use emg_common::{mouse, IdStr};

    fn aaaa(x: impl Into<IdStr>) {
        let f = x.into();
        println!("{f:?}");
    }

    #[test]
    fn aa() {
        let n = "xx";
        aaaa(n);
    }
    #[test]
    fn xx() {
        let f = mouse::CLICK;
        println!("{f:?}");
    }
}

// ────────────────────────────────────────────────────────────────────────────────

// pub use sandbox::Sandbox;
// #[cfg(target_arch = "wasm32")]
// pub use runtime::settings;

// mod g_tree_builder_fn_for_node_item;
// mod g_tree_builder_fn_for_node_item_rc;
// mod graph_store;

// ────────────────────────────────────────────────────────────────────────────────
// mod state_store;
// mod topo_store;

const VEC_SMALL: usize = 4;

// ────────────────────────────────────────────────────────────────────────────────
// ────────────────────────────────────────────────────────────────────────────────
#[cfg(all(test, feature = "gpu"))]
mod tests {
    use emg_msg_macro::emg_msg;

    use crate::element::GTreeBuilderElement;
    use crate::emg_msg_macro_prelude::{any, better_any};
    use crate::gtree_macro_prelude::{gtree, Checkbox, GtreeInitCall, IdStr, Layer, Tid};

    #[emg_msg]
    enum M {
        IncrementPressed,
    }
    fn tree_build() -> GTreeBuilderElement<M> {
        gtree! {
            @="a" Layer [
                @="b" Checkbox::new(false,"abcd",|_|M::IncrementPressed) => [
                    ]
            ]
        }
    }

    fn tree_build2() -> GTreeBuilderElement<M> {
        gtree! {
                @="a" Checkbox::new(false,"abcd",|_|M::IncrementPressed) => [
                    ]
        }
    }

    #[test]
    fn tree_build_tests() {
        let a = tree_build();
        #[cfg(feature = "insta")]
        insta::assert_debug_snapshot!("tree-a", a);
        let b = tree_build2();
        #[cfg(feature = "insta")]
        insta::assert_debug_snapshot!("tree-b", b);
    }
}
#[cfg(all(test, target_arch = "wasm32"))]
mod tests {

    #[allow(unused)]
    use crate::{
        common::{
            better_any::{impl_tid, tid, type_id, Tid, TidAble, TidExt},
            IdStr, TypeCheck,
        },
        layout::{add_values::*, css, styles::*, EmgEdgeItem},
        runtime::{node_ref, EventCallback, EventMessage, GElement, GTreeBuilderElement},
        shaping::{EqShaping, Shaper, Shaping, ShapingUse},
        state::{use_state, CloneState, CloneStateAnchor, StateMultiAnchor},
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
