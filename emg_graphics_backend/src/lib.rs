mod error;
mod viewport;
// ────────────────────────────────────────────────────────────────────────────────
pub mod renderer;

pub mod backend;
pub mod window;

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
