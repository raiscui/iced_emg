use derive_more::Display;
use na::{Affine3, Isometry3, Matrix4, Point3, Rotation3, Similarity3, Translation3, Vector3};
use nalgebra as na;
use seed_styles::{pc, px, CssHeight, CssWidth, ExactLength, Percent, Style};

type Vec3 = Vector3<f64>;
type Mat4 = Matrix4<f64>;
type Trans3 = Translation3<f64>;
type Rot3 = Rotation3<f64>;
type Transform9 = Affine3<f64>;
type pos = Point3<f64>;

#[derive(Display, Clone, Debug)]
#[display(fmt = "width: {};")]
pub enum GenericSize {
    #[display(fmt = "auto")]
    Auto,
    Length(ExactLength),
    Percentage(Percent),
    #[display(fmt = "initial")]
    Initial,
    #[display(fmt = "inherit")]
    Inherit,
    StringValue(String),
}
impl Default for GenericSize {
    fn default() -> Self {
        Self::Length(px(0))
    }
}

impl From<ExactLength> for GenericSize {
    fn from(length: ExactLength) -> Self {
        Self::Length(length)
    }
}
impl From<Percent> for GenericSize {
    fn from(percentage: Percent) -> Self {
        Self::Percentage(percentage)
    }
}
struct WHSize {
    w: GenericSize,
    h: GenericSize,
}
impl Default for WHSize {
    fn default() -> Self {
        Self {
            w: px(16).into(),
            h: px(16).into(),
        }
    }
}

struct Layout {
    size: WHSize,
    origin: GenericSize,
    align: GenericSize,
}

impl Default for Layout {
    fn default() -> Self {
        Self {
            size: Default::default(),
            origin: pc(0).into(),
            align: pc(0).into(),
        }
    }
}

struct LayoutCompiled {
    w_either: Option<GenericSize>,
    origin_x_either: Option<GenericSize>,
    align_x_either: Option<GenericSize>,
    h_either: Option<GenericSize>,
    origin_y_either: Option<GenericSize>,
    align_y_either: Option<GenericSize>,
}
impl Default for LayoutCompiled {
    fn default() -> Self {
        Self {
            w_either: None,
            origin_x_either: None,
            align_x_either: None,
            h_either: None,
            origin_y_either: None,
            align_y_either: None,
        }
    }
}

struct Transforms {
    loc: Trans3,
    scale: Vec3,
    rotate: Rot3,
}
impl Default for Transforms {
    fn default() -> Self {
        Self {
            loc: Trans3::identity(),
            scale: Vec3::from_element(0.),
            rotate: Rot3::identity(),
        }
    }
}
struct M4Data {
    m4: Transform9,
    m4_def: Transform9,
    layout_m4: Transform9,
    //TODO m4fDef:ED msg -> M4.Mat4
    world_inv: Transform9,
    layout_inv: Transform9,
    m4offset: Transform9,
}
impl Default for M4Data {
    fn default() -> Self {
        Self {
            m4: Transform9::identity(),
            m4_def: Transform9::identity(),
            layout_m4: Transform9::identity(),
            //TODO m4fDef:ED msg -> M4.Mat4
            world_inv: Transform9::identity(),
            layout_inv: Transform9::identity(),
            m4offset: Transform9::identity(),
        }
    }
}
#[derive(Default)]
struct EdgeDataOutput {
    loc_styles: Style,
}
#[derive(Default)]
struct EdgeData {
    id: String,
    layout: Layout,
    compiled: LayoutCompiled,
    matrix: M4Data,
    // transforms_am: Transforms,
    ed_output: EdgeDataOutput,
    // animations:
}

#[cfg(test)]
mod tests {
    use seed_styles::CssWidthTrait;

    use super::*;
    #[test]
    fn it_works() {
        println!("{:?}", Transform9::identity());
        let cc = Transform9::identity();
        let p = px(2);
        let s = Style::default();
        println!("{:?}", cc.inverse());
        println!("{:?}", s.w(px(11)).render());
        // ─────────────────────────────────────────────────────────────────
    }
}
