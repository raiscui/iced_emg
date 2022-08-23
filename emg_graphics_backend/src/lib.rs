mod error;
// ────────────────────────────────────────────────────────────────────────────────
pub mod renderer;

pub mod backend;
pub mod window;

// ────────────────────────────────────────────────────────────────────────────────
pub use renderer::Renderer;

pub use backend::Backend;
pub use error::Error;

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {}
}
