mod backend;
mod scene_ctx;
// ────────────────────────────────────────────────────────────────────────────────

pub mod settings;
pub mod window;

// ────────────────────────────────────────────────────────────────────────────────
pub use backend::Backend;
pub use scene_ctx::SceneFrag;
pub use settings::Settings;
// ────────────────────────────────────────────────────────────────────────────────

pub type Renderer = emg_graphics_backend::Renderer<Backend>;

// ────────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {}
}
