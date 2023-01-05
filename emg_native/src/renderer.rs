pub use emg_renderer::*;

pub trait Renderer {
    type SceneCtx;
    //TODO use widget_state, not default

    // fn with_translation(
    //     &mut self,
    //     translation: Vector,
    //     f: impl FnOnce(&mut Self),
    // ) {
    //     let current_primitives = std::mem::take(&mut self.primitives);

    //     f(self);

    //     let layer_primitives =
    //         std::mem::replace(&mut self.primitives, current_primitives);

    //     self.primitives.push(Primitive::Translate {
    //         translation,
    //         content: Box::new(Primitive::Group {
    //             primitives: layer_primitives,
    //         }),
    //     });
    // }

    fn on_loop_destroyed(&mut self);
}
