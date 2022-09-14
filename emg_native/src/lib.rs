#![feature(fn_traits)]
#![feature(specialization)]
// #![feature(trivial_bounds)] // for emg_state::StateAnchor<emg_common::nalgebra::Translation<f64, 3>>: emg_shaping::ShapingWhoNoWarper;
// ────────────────────────────────────────────────────────────────────────────────
pub mod bus;
pub mod clipboard;
pub mod command;
pub mod event;
pub mod future_runtime;
pub mod keyboard;
pub mod mouse;
pub mod paint_ctx;
pub mod program;
pub mod renderer;
pub mod system;
pub mod touch;
pub mod widget;
pub mod window;
// ────────────────────────────────────────────────────────────────────────────────
#[cfg(feature = "debug")]
#[path = "debug/basic.rs"]
mod debug;
#[cfg(not(feature = "debug"))]
#[path = "debug/null.rs"]
mod debug;
// ────────────────────────────────────────────────────────────────────────────────
pub use clipboard::Clipboard;
pub use command::Command;
pub use debug::Debug;
// pub use element::Element;
pub use emg_common::time;
pub use emg_common::Pos;
pub use emg_futures::{executor, futures};
pub use event::Event;
pub use executor::Executor;
pub use future_runtime::FutureRuntime;
// pub use hasher::Hasher;
// pub use layout::Layout;
// pub use overlay::Overlay;
pub use paint_ctx::{PaintCtx, WidgetState, DPR};
pub use program::Program;
// pub use renderer::*;
// pub use shell::Shell;
// pub use theme::Theme;
// pub use user_interface::UserInterface;
pub use bus::Bus;
pub use widget::Widget;

// ────────────────────────────────────────────────────────────────────────────────
use emg_state::use_state;
use static_init::dynamic;

#[dynamic]
pub static G_POS: emg_state::StateVar<Option<Pos<f64>>> = use_state(None);

// ────────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {}
}
