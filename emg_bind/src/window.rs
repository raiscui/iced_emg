//! dom window
mod event;
mod observe;
pub use event::Event;
pub use event::WindowEventRecipe;
pub use observe::observe_size;
