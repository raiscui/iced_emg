mod error;
mod viewport;
// ────────────────────────────────────────────────────────────────────────────────
pub mod renderer;

pub mod backend;
pub mod window;

use emg_common::Pos;
use emg_state::{use_state, StateVar};
// ────────────────────────────────────────────────────────────────────────────────
pub use renderer::Renderer;

pub use backend::Backend;
pub use error::Error;
pub use viewport::Viewport;

// ────────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {}
}
