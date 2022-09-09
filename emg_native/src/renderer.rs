pub use emg_renderer::*;

pub trait Renderer {
    type ImplRenderContext: RenderContext;
    //TODO use widget_state, not default

    fn on_loop_destroyed(&mut self);
}
