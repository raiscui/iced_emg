#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![warn(clippy::nursery)]
// #![feature(negative_impls)]
// #![feature(auto_traits)]

use derive_more::Display;
use derive_more::From;
use emg_refresh::{RefreshFor, RefreshUseFor, RefreshUseNoWarper, RefreshWhoNoWarper};
use na::{Affine3, Isometry3, Matrix4, Point3, Rotation3, Similarity3, Translation3, Vector3};
use nalgebra as na;
pub use seed_styles as styles;
use std::rc::Rc;
use styles::{pc, px, CssValueTrait, ExactLength, Percent, Style, UpdateStyle};

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
#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
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

#[derive(Default, Debug, Clone)]
struct EdgeDataOutput {
    loc_styles: Style,
}
#[derive(Default, Debug, Clone)]
pub struct EdgeData {
    id: String,
    layout: Layout,
    compiled: LayoutCompiled,
    matrix: M4Data,
    // transforms_am: Transforms,
    ed_output: EdgeDataOutput,
    // animations:
}
#[derive(From, Clone, Debug)]
pub enum EdgeItem {
    EdgeData(EdgeData),
    String(String),
    Empty,
}

impl Default for EdgeItem {
    fn default() -> Self {
        Self::Empty
    }
}
pub fn e() -> EdgeData {
    EdgeData::default()
}

// TODO lifetime
#[derive(Clone)]
struct Css<T>(T)
where
    T: CssValueTrait + Clone + 'static;

impl<T: std::clone::Clone + seed_styles::CssValueTrait> From<T> for Css<T> {
    fn from(v: T) -> Self {
        Self(v)
    }
}

// impl RefreshWhoNoWarper for EdgeData {}
// TODO lifetime
impl RefreshFor<EdgeData> for Vec<Box<(dyn RefreshFor<EdgeData> + 'static)>> {
    fn refresh_for(&self, who: &mut EdgeData) {
        for i in self {
            let ii = i.as_ref();
            who.refresh_use(ii);
        }
    }
}
impl<Use> RefreshFor<EdgeData> for Css<Use>
where
    Use: CssValueTrait + std::clone::Clone,
{
    fn refresh_for(&self, who: &mut EdgeData) {
        let t = self.0.clone();
        t.update_style(&mut who.ed_output.loc_styles);
    }
}
//TODO lifetime
pub fn css<Use: CssValueTrait + std::clone::Clone + 'static>(v: Use) -> impl RefreshFor<EdgeData> {
    Css(v)
}
// impl<Use> RefreshFor<EdgeData> for Css<Use>
// where
//     Use: CssValueTrait + std::clone::Clone,
// {
//     fn refresh_for(&self, who: &mut EdgeData) {
//         let t = self.0.clone();
//         t.update_style(&mut who.ed_output.loc_styles);
//     }
// }
impl RefreshFor<EdgeData> for Style {
    fn refresh_for(&self, who: &mut EdgeData) {
        who.ed_output.loc_styles = self.clone();
    }
}
impl RefreshFor<EdgeData> for Layout {
    fn refresh_for(&self, who: &mut EdgeData) {
        who.layout = self.clone();
    }
}

#[cfg(test)]
mod tests {

    use seed_styles::CssWidth;

    use super::*;

    #[test]
    fn it_works() {
        use seed_styles::CssWidthTrait;

        println!("{:?}", Transform9::identity());
        let cc = Transform9::identity();
        let p = px(2);
        let cc = CssWidth::from(px(2));
        let mut e = EdgeData::default();
        e.refresh_use(&Css(cc));
        println!("{:?}", &e);
        let mut s = Style::default();
        s = s.style_child("dddddd");
        println!("{:?}", &s);
        println!("{:?}", &s.render());
        // println!("{:?}", cc.inverse());
        // println!("{:?}", &mut s.w(px(11)).render());
        // println!("{:#?}", e());
        // ─────────────────────────────────────────────────────────────────
    }
}
