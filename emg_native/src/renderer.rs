pub use emg_renderer::*;
pub trait Renderer {
    type ImplRenderContext: crate::RenderContext;
    //TODO use widget_state, not default
    fn new_paint_ctx(&self) -> crate::PaintCtx<Self::ImplRenderContext>;

    fn on_loop_destroyed(&mut self);
}
