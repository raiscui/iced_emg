pub use emg_renderer::*;
pub trait Renderer {
    type ImplRenderContext: crate::RenderContext;
    fn new_paint_ctx(&self) -> PaintCtx<Self::ImplRenderContext>;

    fn on_loop_destroyed(&mut self);
}
