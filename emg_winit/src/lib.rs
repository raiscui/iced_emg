mod error;
mod mode;
mod proxy;
mod semantic_position;
// ────────────────────────────────────────────────────────────────────────────────

pub mod application;
pub mod clipboard;
pub mod conversion;
pub mod settings;
pub mod window;

#[cfg(feature = "system")]
pub mod system;

// ────────────────────────────────────────────────────────────────────────────────
pub use application::Application;
pub use clipboard::Clipboard;
pub use emg_element::GraphProgram;
pub use emg_native::*;
pub use error::Error;
pub use mode::Mode;
pub use proxy::Proxy;
pub use semantic_position::SemanticPosition;
pub use settings::Settings;
pub use winit;

// pub use emg_graphics::Viewport;
// ────────────────────────────────────────────────────────────────────────────────

// ────────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {}
}
