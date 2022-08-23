#![feature(box_patterns)]
#![feature(specialization)]
#![feature(trait_upcasting)]

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
pub use g_node::{GelType, GraphType, NItem, E, N};
pub use g_tree_builder::{GTreeBuilderElement, GTreeBuilderFn};
pub use graph_program::GraphProgram;
pub use node_builder::NodeBuilderWidget;
pub use widget::Widget;

// ────────────────────────────────────────────────────────────────────────────────
#[cfg(all(feature = "gpu"))]
use emg_native::{PaintCtx, RenderContext};
// ────────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {}
}
