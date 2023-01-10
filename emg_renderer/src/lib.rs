// ────────────────────────────────────────────────────────────────────────────────
// pub use piet::kurbo::Affine;
// pub use piet::kurbo::Point;
// pub use piet::kurbo::Rect;
// pub use piet::kurbo::Size;
// pub use piet::kurbo::Vec2;

// pub use piet::{Color, Error, RenderContext};

// #[cfg(test)]
// mod tests {

//     #[test]
//     fn it_works() {}
// }

pub use peniko::{
    kurbo::{Affine, PathEl, Point, Rect, Shape, Size, Vec2},
    BlendMode, Brush, BrushRef, Color, Fill, Stroke,
};

pub trait SceneCtx {
    type Ctx<'a>: SceneBuilder
    where
        Self: 'a;

    fn new(transform: Option<Affine>) -> Self;

    fn gen_builder(&mut self) -> Self::Ctx<'_>;

    fn get_transform(&self) -> Option<Affine>;
}

pub trait SceneBuilder {
    type SceneCtx;
    fn push_layer(
        &mut self,
        blend: impl Into<BlendMode>,
        alpha: f32,
        transform: Affine,
        shape: &impl Shape,
    );
    fn pop_layer(&mut self);

    fn fill<'b>(
        &mut self,
        style: Fill,
        transform: Affine,
        brush: impl Into<BrushRef<'b>>,
        brush_transform: Option<Affine>,
        shape: &impl Shape,
    );

    /// Strokes a shape using the specified style and brush.
    fn stroke<'b>(
        &mut self,
        style: &Stroke,
        transform: Affine,
        brush: impl Into<BrushRef<'b>>,
        brush_transform: Option<Affine>,
        shape: &impl Shape,
    );

    /// Appends a fragment to the scene.
    fn append(&mut self, fragment: &Self::SceneCtx, transform: Option<Affine>);

    /// Completes construction and finalizes the underlying scene.
    fn finish(self);
}
