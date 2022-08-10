#![allow(clippy::module_name_repetitions)]

//! dom window
// mod event;
mod observe;
mod settings;
// pub use event::Event;
// pub use event::WindowEventRecipe;
pub use observe::observe_size;
pub use settings::Settings;
