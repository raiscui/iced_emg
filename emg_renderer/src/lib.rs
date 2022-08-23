mod paint_ctx;
// ────────────────────────────────────────────────────────────────────────────────
pub use paint_ctx::*;
pub use piet::kurbo::Rect;
use piet::kurbo::Size;

pub use piet::{Color, RenderContext};

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {}
}
