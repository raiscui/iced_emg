#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::used_underscore_binding)]
// ────────────────────────────────────────────────────────────────────────────────
#![feature(iter_intersperse)]

// // 固定 type  T:xxx
// #![feature(trivial_bounds)]
// #![feature(negative_impls)]
// #![feature(auto_traits)]

use std::{
    clone::Clone,
    cmp::{Eq, Ord},
    hash::Hash,
};

use calc::layout_calculating;
use derive_more::Display;
use derive_more::From;
use derive_more::Into;
use derive_more::TryInto;
use emg::EdgeIndex;
use emg_refresh::RefreshFor;
use emg_state::{
    topo, use_state, CloneStateAnchor, CloneStateVar, Dict, StateAnchor, StateMultiAnchor, StateVar,
};
use im::Vector;
use na::{
    Affine3, Isometry3, Matrix4, Point3, Rotation3, Similarity3, Translation3, Vector2, Vector3,
};
use nalgebra as na;
pub use seed_styles as styles;
use styles::{
    px, s, CssHeight, CssTransform, CssValueTrait, CssWidth, ExactLength, Percent, Style,
    UpdateStyle,
};
use styles::{CssHeightTrait, CssTransformTrait, CssWidthTrait};
//
// ────────────────────────────────────────────────────────────────────────────────

use indented::indented;
use tracing::{instrument, trace};
use tracing::{span, Level};
// ────────────────────────────────────────────────────────────────────────────────

mod calc;
mod impl_refresh;
// ────────────────────────────────────────────────────────────────────────────────

type Size2 = Vector2<f64>;
type Vec3 = Vector3<f64>;
type Trans3 = Translation3<f64>;
type Rot3 = Rotation3<f64>;
type Transform9 = Affine3<f64>;
type Pos3 = Point3<f64>;

#[derive(Display, Debug, PartialEq, PartialOrd, Copy, Clone, From, Into)]
struct Mat4(Matrix4<f64>);

// type Mat4 = Matrix4<f64>;
#[derive(Display, Clone, Debug, From, TryInto, PartialEq, PartialOrd, Eq)]
#[try_into(owned, ref, ref_mut)]
#[display(fmt = "{}")]
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
impl From<CssWidth> for GenericSize {
    fn from(w: CssWidth) -> Self {
        match w {
            CssWidth::Auto => Self::Auto,
            CssWidth::Length(x) => x.into(),
            CssWidth::Percentage(x) => x.into(),
            CssWidth::Initial => Self::Initial,
            CssWidth::Inherit => Self::Inherit,
            CssWidth::StringValue(x) => x.into(),
        }
    }
}
impl From<CssHeight> for GenericSize {
    fn from(w: CssHeight) -> Self {
        match w {
            CssHeight::Auto => Self::Auto,
            CssHeight::Length(x) => x.into(),
            CssHeight::Percentage(x) => x.into(),
            CssHeight::Initial => Self::Initial,
            CssHeight::Inherit => Self::Inherit,
            CssHeight::StringValue(x) => x.into(),
        }
    }
}

impl GenericSize {
    /// # Errors
    ///
    /// Will return `Err` if `self` does not `Length` and `Length`  unit is not px
    pub fn try_get_length_value(&self) -> Result<f64, &Self> {
        self.as_length()
            .and_then(|l| l.try_get_number().ok())
            .ok_or(self)
    }

    #[must_use]
    pub const fn as_length(&self) -> Option<&ExactLength> {
        if let Self::Length(v) = self {
            Some(v)
        } else {
            None
        }
    }
}
impl Default for GenericSize {
    fn default() -> Self {
        Self::Length(px(0))
    }
}

impl Default for GenericWH {
    fn default() -> Self {
        Self {
            w: px(16).into(),
            h: px(16).into(),
        }
    }
}
impl Default for GenericLoc {
    fn default() -> Self {
        Self {
            x: px(0).into(),
            y: px(0).into(),
            z: px(0).into(),
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

#[derive(Debug, Clone)]
struct EdgeDataOutput {
    loc_styles: StateAnchor<Style>,
    other_styles: StateVar<Style>,
    styles: StateAnchor<Style>,
    style_string: StateAnchor<String>,
}
#[derive(Display, Debug, Clone, PartialEq, PartialOrd, Eq)]
// #[derive(Debug, Clone, Into)]
// #[into(owned, ref, ref_mut)]
#[display(fmt = "(w:{},h:{})", w, h)]
pub struct GenericWH {
    w: GenericSize,
    h: GenericSize,
}

impl GenericWH {
    pub fn new(w: impl Into<GenericSize>, h: impl Into<GenericSize>) -> Self {
        Self {
            w: w.into(),
            h: h.into(),
        }
    }

    #[must_use]
    pub fn get_length_value(&self) -> (f64, f64) {
        (
            self.w
                .try_get_length_value()
                .expect("root size w get failed, expected Length Px struct"),
            self.h
                .try_get_length_value()
                .expect("root size h get failed, expected Length Px struct"),
        )
    }
}
#[derive(Display, Debug, Clone, PartialEq, PartialOrd)]
// #[derive(Debug, Clone, Into)]
// #[into(owned, ref, ref_mut)]
#[display(fmt = "(x:{},y:{},z:{})", x, y, z)]
pub struct GenericLoc {
    x: GenericSize,
    y: GenericSize,
    z: GenericSize,
}

impl GenericLoc {
    pub fn new(
        x: impl Into<GenericSize>,
        y: impl Into<GenericSize>,
        z: impl Into<GenericSize>,
    ) -> Self {
        Self {
            x: x.into(),
            y: y.into(),
            z: z.into(),
        }
    }

    #[must_use]
    pub fn get_length_value(&self) -> (f64, f64, f64) {
        (
            self.x
                .try_get_length_value()
                .expect("root size get failed, expected Length Px struct"),
            self.y
                .try_get_length_value()
                .expect("root size get failed, expected Length Px struct"),
            self.z
                .try_get_length_value()
                .expect("root size get failed, expected Length Px struct"),
        )
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct Layout<Ix>
where
    Ix: Clone + Hash + Eq + Default + PartialOrd + std::cmp::Ord + 'static,
{
    size: StateVar<GenericWH>,
    origin: StateVar<GenericLoc>,
    align: StateVar<GenericLoc>,
    path_styles: StateVar<Dict<EPath<Ix>, Style>>,
}
impl<Ix> Copy for Layout<Ix> where
    Ix: Clone + Hash + Eq + Default + PartialOrd + std::cmp::Ord + 'static
{
}

// impl<Ix> Default for Layout<Ix> {
//     #[topo::nested]
//     fn default() -> Self {
//         Self {
//             size: use_state(GenericWH::default()),
//             origin: use_state(GenericLoc::default()),
//             align: use_state(GenericLoc::default()),
//         }
//     }
// }

// impl<Ix> Ord for Layout<Ix> where Ix: Clone + Hash + Eq + Default + PartialOrd + std::cmp::Ord {}
// impl<Ix> Eq for Layout<Ix> where Ix: Clone + Hash + Eq + Default + PartialOrd + std::cmp::Ord {}
impl<Ix> std::fmt::Display for Layout<Ix>
where
    Ix: Clone + Hash + Eq + Default + Ord + 'static + std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let x = format!(
            "size:{{\n{};\n}}\norigin:{{\n{};\n}}\nalign:{{\n{};\npath_styles:{{\n{};\n}}",
            indented(&self.size),
            indented(&self.origin),
            indented(&self.align),
            indented(DictDisplay(self.path_styles.get()))
        );
        write!(f, "Layout {{\n{}\n}}", indented(&x))
    }
}

struct DictDisplay<K, V>(Dict<K, V>);
impl<K, V> std::fmt::Display for DictDisplay<K, V>
where
    K: std::fmt::Display + Ord,
    V: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let sv: String = self
            .0
            .iter()
            .map(|(k, v)| format!("{} :\n{}\n,\n", k, indented(v)))
            .fold(String::default(), |acc, v| format!("{}{}", acc, v));

        write!(f, "Dict {{\n{}\n}}", indented(&sv))
    }
}
#[derive(Display, Debug, Clone, PartialEq)]
#[display(
    fmt = "{{\nsize:\n{},\norigin:\n{},align:\n{},\ncoordinates_trans:\n{},\nmatrix:\n{},\nloc_styles:\n{},\n}}",
    "indented(size)",
    "indented(origin)",
    "indented(align)",
    "indented(coordinates_trans)",
    "indented(matrix)",
    "indented(loc_styles)"
)]
pub struct LayoutCalculated {
    size: StateAnchor<Size2>,
    origin: StateAnchor<Trans3>,
    align: StateAnchor<Trans3>,
    coordinates_trans: StateAnchor<Trans3>,
    matrix: StateAnchor<Mat4>,
    loc_styles: StateAnchor<Style>,
}
// impl std::fmt::Debug for Layoutcalculated {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_tuple("Layoutcalculated").finish()
//     }
// }

#[derive(Debug, Clone, PartialEq)]
pub struct EdgeData {
    calculated: LayoutCalculated,
    styles_string: StateAnchor<String>, // matrix: M4Data,
                                        // transforms_am: Transforms,
                                        // animations:
}

impl Eq for EdgeData {}

impl std::fmt::Display for EdgeData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let x = format!(
            "calculated:{{\n{};\nstyles_string:{{\n{};\n}}\n}}",
            indented(&self.calculated),
            indented(&self.styles_string)
        );
        write!(f, "EdgeData {{\n{}\n}}", indented(&x))
    }
}

// impl std::fmt::Debug for EdgeData {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         // let layout = self.layout.get();
//         // let size = layout.size.get();
//         // drop(layout);
//         // drop(size);
//         // let calculated = self.calculated.get();
//         // let calculated_size = calculated.size.get();
//         // drop(calculated);
//         f.debug_tuple("EdgeData")
//             // .field(&size)
//             // .field(&calculated_size)
//             .finish()
//     }
// }

impl From<Mat4> for CssTransform {
    fn from(Mat4(matrix): Mat4) -> Self {
        Self::from(format!(
            "matrix3d({},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{})",
            &matrix.m11,
            &matrix.m21,
            &matrix.m31,
            &matrix.m12,
            &matrix.m41,
            &matrix.m22,
            &matrix.m32,
            &matrix.m42,
            &matrix.m13,
            &matrix.m23,
            &matrix.m33,
            &matrix.m43,
            &matrix.m14,
            &matrix.m24,
            &matrix.m34,
            &matrix.m44
        ))
    }
}
impl<T> UpdateStyle<T> for Mat4
where
    Mat4: Into<T>,
    T: UpdateStyle<T>,
{
    fn update_style(self, style: &mut Style) {
        self.into().update_style(style)
    }
}

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Hash, Debug, Default)]
pub struct EPath<Ix: Clone + Hash + Eq + PartialEq + Default>(Vector<EdgeIndex<Ix>>);

impl<Ix> std::fmt::Display for EPath<Ix>
where
    Ix: Clone + Hash + Eq + PartialEq + Default + std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let sv: String = self
            .0
            .iter()
            //TODO  textwrap
            .map(|v| format!("{}", v))
            .intersperse(String::from(","))
            .fold(String::default(), |acc, v| format!("{}{}", acc, v));

        write!(f, "path [{}]", &sv)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct EmgEdgeItem<Ix>
where
    Ix: Clone + Hash + Eq + Ord + 'static + Default,
{
    pub id: StateVar<EdgeIndex<Ix>>,
    pub paths: StateAnchor<Dict<EPath<Ix>, EdgeItemNode>>, // with parent self
    pub layout: Layout<Ix>,
    pub other_styles: StateVar<Style>,
    // no self  first try
    pub node: StateAnchor<Dict<EPath<Ix>, EdgeItemNode>>, //TODO with self?  not with self?
}

impl<Ix> EmgEdgeItem<Ix>
where
    Ix: Clone + Hash + Eq + PartialEq + PartialOrd + Ord + Default + std::fmt::Display,
{
    #[must_use]
    pub fn get_edge_data(&self, key: &EPath<Ix>) -> Option<EdgeData> {
        self.node
            .get()
            .get(key)
            .and_then(|x| x.as_edge_data().cloned())
    }

    #[topo::nested]
    #[instrument(skip(id))]
    pub fn new_root<T: Into<f64> + std::fmt::Debug>(id: impl Into<Ix> + Clone, w: T, h: T) -> Self {
        // info!(target: "yak_events", "Commencing yak shaving for ");
        let id: StateVar<EdgeIndex<Ix>> =
            use_state(EdgeIndex::new(id.clone().into(), id.clone().into()));

        let layout = Layout::<Ix> {
            size: use_state(GenericWH {
                w: px(w).into(),
                h: px(h).into(),
            }),
            origin: use_state(GenericLoc::default()),
            align: use_state(GenericLoc::default()),
            path_styles: use_state(Dict::unit(EPath::<Ix>::default(), s())),
        };
        let other_styles_sv = use_state(s());

        // ─────────────────────────────────────────────────────────────────

        // TODO how to use Pre-acquired G_STATE_STORE Optimize performance
        let calculated_size = layout.size.watch().map(|g_wh: &GenericWH| {
            // println!("in layout size watch map");
            let (w, h) = g_wh.get_length_value();

            Size2::new(w, h)
        });
        //TODO make dyn
        let calculated_origin = StateAnchor::constant(Trans3::identity());
        let calculated_align = StateAnchor::constant(Trans3::identity());
        let coordinates_trans = StateAnchor::constant(Trans3::identity());
        let matrix = coordinates_trans.map(|x| x.to_homogeneous().into());
        // ────────────────────────────────────────────────────────────────────────────────
        let loc_styles = (&calculated_size, &matrix).map(move |size: &Size2, mat4: &Mat4| {
            let _enter = span!(Level::TRACE,
                        "-> [root] [ loc_styles ] recalculation..(&calculated_size, &matrix).map ",
                        %id)
            .entered();

            trace!("size: {}  , matrix: {}", size, CssTransform::from(*mat4));

            // TODO use  key 更新 s(),
            s().w(px(size.x)).h(px(size.y)).transform(*mat4)
        });
        let layout_calculated = LayoutCalculated {
            size: calculated_size,
            origin: calculated_origin,
            align: calculated_align,
            coordinates_trans,
            matrix,
            // • • • • •
            loc_styles,
        };
        // let path_styles_sv = use_state(s());

        let styles_string = (
            &layout.path_styles.watch(),
            &layout_calculated.loc_styles,
            &other_styles_sv.watch(),
        )
            .map(
                move |path_styles: &Dict<EPath<Ix>,Style>, loc_styles: &Style, other_styles: &Style| {
                    let _enter = span!(Level::TRACE,
                            "-> [ROOT styles ] recalculation..(&other_styles_watch, &loc_styles).map ",
                            %id)
                    .entered();

                    let ps = path_styles.values().fold(String::default(), |acc,v|{
                         format!("{}{}",acc,v.render())
                    });

                    format!(
                        "{}{}{}",
                        other_styles.render(),
                        ps,
                        loc_styles.render()
                    )
                },
            );

        let paths = StateAnchor::constant(Dict::unit(EPath::<Ix>::default(), EdgeItemNode::Empty));

        let node: StateAnchor<Dict<EPath<Ix>, EdgeItemNode>> = paths.map_(move |_k, _v| {
            EdgeItemNode::EdgeData(EdgeData {
                calculated: layout_calculated.clone(),
                styles_string: styles_string.clone(),
            })
        });

        Self {
            id,
            paths,
            layout,
            other_styles: other_styles_sv,
            node,
        }
    }

    #[topo::nested]
    pub fn new_child(
        eix: EdgeIndex<Ix>,
        paths: StateAnchor<Dict<EPath<Ix>, EdgeItemNode>>,
        size: impl Into<GenericWH>,
        origin: impl Into<GenericLoc>,
        align: impl Into<GenericLoc>,
    ) -> Self {
        let id = use_state(eix.clone());
        let _child_span = span!(Level::TRACE, " build new child ",id=%id).entered();
        let layout = Layout::<Ix> {
            size: use_state(size.into()),
            origin: use_state(origin.into()),
            align: use_state(align.into()),
            path_styles: use_state(Dict::unit(EPath::<Ix>::default(), s())),
        };

        let other_styles_sv = use_state(s());

        //TODO not paths: StateVar<Dict<EPath<Ix>,EdgeItemNode>>  use edgeIndex instead to Reduce memory
        let node: StateAnchor<Dict<EPath<Ix>, EdgeItemNode>> =
            paths.map_(move |path, path_edge_item_node| {
                let _child_span =
                    span!(Level::TRACE, "[ node recalculation ]:paths change ").entered();

                // TODO  make path specialization style at EmgEdgeItem dict path->style variable

                let layout_calculated = layout_calculating(id, path_edge_item_node, layout);

                let p = path.clone();

                let this_path_style_string_sa: StateAnchor<Option<String>> = layout
                    .path_styles
                    .watch()
                    .map(move |d: &Dict<EPath<Ix>, Style>| {
                        d.get(&p).map(seed_styles::Style::render)
                    });

                let styles_string = (
                    &this_path_style_string_sa,
                    &layout_calculated.loc_styles,
                    &other_styles_sv.watch(),
                )
                    .map(
                        move |path_styles_string: &Option<String>,
                              loc_styles: &Style,
                              other_styles: &Style| {
                            let _enter = span!(Level::TRACE,
                            "-> [ styles ] recalculation..(&other_styles_watch, &loc_styles).map ",
                            %id)
                            .entered();

                            format!(
                                "{}{}{}",
                                other_styles.render(),
                                path_styles_string.as_ref().unwrap_or(&String::default()),
                                loc_styles.render()
                            )
                        },
                    );

                EdgeItemNode::EdgeData(EdgeData {
                    calculated: layout_calculated,
                    styles_string,
                })
            });

        Self {
            id,
            paths,
            layout,
            other_styles: other_styles_sv,
            node,
        }
    }
}

impl<
        Ix: 'static
            + Clone
            + Hash
            + Eq
            + PartialEq
            + PartialOrd
            + Ord
            + std::default::Default
            + std::fmt::Display,
    > std::fmt::Display for EmgEdgeItem<Ix>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let x = format!(
            "id:{{\n{};\n}}\nlayout:{{\n{};\n}}\nother_styles:{{\n{};\n}}\nnode:{{\n{};\n}}",
            indented(&self.id),
            indented(&self.layout),
            indented(&self.other_styles),
            indented(DictDisplay(self.node.get()))
        );
        write!(f, "EdgeDataWithParent {{\n{}\n}}", indented(&x))
    }
}

// fn try_get_parent_calc_size(parent: &StateVar<Option<EdgeItemNode>>) -> Option<Size2> {
//     parent
//         .get()
//         .and_then(|ei| ei.as_edge_data().map(|ed| ed.current.calculated.size.get()))
// }
// fn get_parent_calc_size_to_string(parent: &StateVar<Option<EdgeItemNode>>) -> String {
//     match try_get_parent_calc_size(parent) {
//         Some(size) => {
//             format!("w:{} h:{}", &size.x, &size.y)
//         }
//         None => String::from("None"),
//     }
// }

#[derive(Display, From, Clone, Debug, PartialEq, Eq)]
pub enum EdgeItemNode {
    EdgeData(EdgeData),
    String(String), //TODO make can write
    Empty,
}

impl EdgeItemNode {
    #[must_use]
    pub const fn as_edge_data(&self) -> Option<&EdgeData> {
        if let Self::EdgeData(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

// #[topo::nested]
// pub fn edge_item_data_with_parent(
//     id: impl Into<String>,
//     parent_edge_item_sv: StateVar<Option<EdgeItemNode>>,
// ) -> EdgeItemNode {
//     EdgeItemNode::new_child(
//         id,
//         parent_edge_item_sv, //TODO maybe use Rc instead of clone?
//         size(px(10), px(10)),
//         origin2(pc(0), pc(0)),
//         align2(pc(0), pc(0)),
//     )
// }

impl Default for EdgeItemNode {
    fn default() -> Self {
        Self::Empty
    }
}

// TODO lifetime
#[derive(Clone)]
pub struct Css<T>(T)
where
    T: CssValueTrait + Clone + 'static;

impl<T: Clone + seed_styles::CssValueTrait> From<T> for Css<T> {
    fn from(v: T) -> Self {
        Self(v)
    }
}

pub fn size(w: impl Into<GenericSize>, h: impl Into<GenericSize>) -> GenericWH {
    GenericWH::new(w, h)
}
pub fn origin2(x: impl Into<GenericSize>, y: impl Into<GenericSize>) -> GenericLoc {
    GenericLoc::new(x, y, px(0))
}
pub fn align2(x: impl Into<GenericSize>, y: impl Into<GenericSize>) -> GenericLoc {
    GenericLoc::new(x, y, px(0))
}
//TODO lifetime
pub fn css<
    Use: CssValueTrait + Clone + 'static,
    Ix: Clone + Hash + Eq + Ord + 'static + Default,
>(
    v: Use,
) -> Box<dyn RefreshFor<EmgEdgeItem<Ix>>> {
    // pub fn css<Use: CssValueTrait + std::clone::Clone + 'static>(v: Use) -> Box<Css<Use>> {
    Box::new(Css(v))
}
// fn get_current_edge_data(edge_item_sv: StateVar<Option<EdgeItemNode>>) -> EdgeData {
//     edge_item_sv
//         .get()
//         .unwrap()
//         .as_current_edge_data()
//         .unwrap()
//         .clone()
// }
// fn get_edge_parent(edge_item_sv: StateVar<Option<EdgeItemNode>>) -> StateVar<Option<EdgeItemNode>> {
//     edge_item_sv.get().unwrap().as_edge_data().unwrap().parent
// }
#[cfg(test)]
mod tests {

    use super::*;
    use emg_refresh::RefreshUseFor;
    use im::vector;
    use seed_styles::CssWidth;
    use styles::pc;
    use tracing::info;

    use tracing_flame::FlameLayer;
    use tracing_subscriber::{fmt, prelude::*, EnvFilter};

    fn setup_global_subscriber() -> impl Drop {
        std::env::set_var("RUST_LOG", "trace");

        let _el = env_logger::try_init();

        let filter_layer = EnvFilter::try_from_default_env()
            .or_else(|_| EnvFilter::try_new("info"))
            .unwrap();

        let fmt_layer = fmt::Layer::default()
            .with_target(false)
            .with_test_writer()
            .with_span_events(
                tracing_subscriber::fmt::format::FmtSpan::ENTER
                    // |tracing_subscriber::fmt::format::FmtSpan::FULL
                    | tracing_subscriber::fmt::format::FmtSpan::CLOSE,
            );

        let (flame_layer, _guard) = FlameLayer::with_file("./tracing/tracing.folded").unwrap();

        let _s = tracing_subscriber::registry()
            .with(filter_layer)
            .with(fmt_layer)
            .with(flame_layer)
            .try_init();
        _guard
    }

    fn _init() {
        let _el = env_logger::try_init();

        let _subscriber = tracing_subscriber::fmt()
            .with_test_writer()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .with_span_events(
                tracing_subscriber::fmt::format::FmtSpan::ACTIVE
                    | tracing_subscriber::fmt::format::FmtSpan::ENTER
                    | tracing_subscriber::fmt::format::FmtSpan::CLOSE,
            )
            // .with_max_level(Level::TRACE)
            .try_init();

        // tracing::subscriber::set_global_default(subscriber)
        // .expect("setting default subscriber failed");
    }
    #[test]
    fn loc() {
        // init();
        let _xx = setup_global_subscriber();
        {
            let span = span!(Level::TRACE, "loc-test");
            let _guard = span.enter();

            info!("=========================================================");

            let css_width = CssWidth::from(px(100));
            let css_height = CssHeight::from(px(100));
            let mut root_e = EmgEdgeItem::new_root("root", 1920, 1080);
            let path_root: StateAnchor<Dict<EPath<&str>, EdgeItemNode>> =
                root_e.node.map(|x: &Dict<EPath<&str>, EdgeItemNode>| {
                    let _span =
                        span!(Level::TRACE, "[ root_e.node change, path_root rebuild ]").entered();

                    x.iter()
                        .map(|(k, v)| {
                            let mut nk = k.clone();
                            nk.0.push_back(EdgeIndex::new("root", "root"));
                            (nk, v.clone())
                        })
                        .collect()
                });

            let e1 = EmgEdgeItem::new_child(
                EdgeIndex::new("root", "e1"),
                path_root.clone(),
                size(px(50), px(50)),
                origin2(pc(0), pc(0)),
                align2(pc(50), pc(50)),
            );
            let path_e1: StateAnchor<Dict<EPath<&str>, EdgeItemNode>> = (&e1.id.watch(), &e1.node)
                .map(
                    |id: &EdgeIndex<&str>, path: &Dict<EPath<&str>, EdgeItemNode>| {
                        let _span = span!(
                            Level::TRACE,
                            "[ (&e1.id.watch(), &e1.node) change, path_e1 rebuild ]"
                        )
                        .entered();

                        path.iter()
                            .map(|(k, v)| {
                                let mut nk = k.clone();
                                nk.0.push_back(*id);
                                (nk, v.clone())
                            })
                            .collect()
                    },
                );

            let mut e2 = EmgEdgeItem::new_child(
                EdgeIndex::new("e1", "e2"),
                path_e1.clone(),
                size(px(10), px(10)),
                origin2(pc(100), pc(100)),
                align2(pc(100), pc(100)),
            );

            // debug!("refresh_use before {}", &ec);
            let _span = span!(Level::TRACE, "debug print e1");
            _span.in_scope(|| {
                trace!("refresh_use before {}", &e1);
            });
            info!("=========================================================");

            root_e.refresh_use(&vec![css(css_width)]);
            root_e.refresh_use(&Css(css_height));
            assert_eq!(
                e1.get_edge_data(&EPath(vector![EdgeIndex::new("root", "root")]))
                    .unwrap()
                    .calculated
                    .coordinates_trans
                    .get(),
                Trans3::new(50., 50., 0.)
            );
            info!("=========================================================");

            e2.refresh_use(&Css(CssWidth::from(px(20))));
            e2.refresh_use(&Css(CssHeight::from(px(20))));

            e2.id.set(EdgeIndex::new("xxx", "yyy"));
            trace!("refresh_use after {:#?}", &e2);
            info!("..=========================================================");
            assert_eq!(
                e2.get_edge_data(&EPath(vector![
                    EdgeIndex::new("root", "root"),
                    EdgeIndex::new("root", "e1")
                ]))
                .unwrap()
                .calculated
                .coordinates_trans
                .get(),
                Trans3::new(30., 30., 0.)
            );
            trace!(
                "{}",
                e2.get_edge_data(&EPath(vector![
                    EdgeIndex::new("root", "root"),
                    EdgeIndex::new("root", "e1")
                ]))
                .unwrap()
                .styles_string
                .get(),
            );
            info!("..=========================================================");
            // trace!("end----- {:#?}", &root_e);
            // trace!("end----- {:#?}", &e1);
            // trace!("end----- {:#?}", &e2);

            // ─────────────────────────────────────────────────────────────────
        }
    }
    #[test]
    fn it_works() {
        // init();
        let _xx = setup_global_subscriber();
        {
            let span = span!(Level::TRACE, "start");
            let _guard = span.enter();

            info!("--------------------=====================================");
            // vec![ CssWidth::from(px(100))].up
            info!("=========================================================");

            // let cc = Transform9::identity();

            let css_width = CssWidth::from(px(100));
            let css_height = CssHeight::from(px(100));
            let mut root_e = EmgEdgeItem::new_root("root", 1920, 1080);
            let path_root: StateAnchor<Dict<EPath<&str>, EdgeItemNode>> =
                root_e.node.map(|x: &Dict<EPath<&str>, EdgeItemNode>| {
                    let _span =
                        span!(Level::TRACE, "[ root_e.node change, path_root rebuild ]").entered();

                    x.iter()
                        .map(|(k, v)| {
                            let mut nk = k.clone();
                            nk.0.push_back(EdgeIndex::new("root", "root"));
                            (nk, v.clone())
                        })
                        .collect()
                });

            let mut e1 = EmgEdgeItem::new_child(
                EdgeIndex::new("root", "e1"),
                path_root.clone(),
                size(px(10), px(10)),
                origin2(pc(100), pc(100)),
                align2(pc(50), pc(20)),
            );
            let path_e1: StateAnchor<Dict<EPath<&str>, EdgeItemNode>> = (&e1.id.watch(), &e1.node)
                .map(
                    |id: &EdgeIndex<&str>, path: &Dict<EPath<&str>, EdgeItemNode>| {
                        let _span = span!(
                            Level::TRACE,
                            "[ (&e1.id.watch(), &e1.node) change, path_e1 rebuild ]"
                        )
                        .entered();

                        path.iter()
                            .map(|(k, v)| {
                                let mut nk = k.clone();
                                nk.0.push_back(*id);
                                (nk, v.clone())
                            })
                            .collect()
                    },
                );

            let mut e2 = EmgEdgeItem::new_child(
                EdgeIndex::new("e1", "e2"),
                path_e1.clone(),
                size(px(10), px(10)),
                origin2(pc(100), pc(100)),
                align2(pc(50), pc(20)),
            );

            // debug!("refresh_use before {}", &ec);
            let _span = span!(Level::TRACE, "debug print e1");
            _span.in_scope(|| {
                trace!("refresh_use before {}", &e1);
            });
            info!("=========================================================");

            assert_eq!(
                e1.node
                    .get()
                    .get(&EPath(vector![EdgeIndex::new("root", "root")]))
                    .and_then(|x| x.as_edge_data())
                    .unwrap()
                    .calculated
                    .coordinates_trans
                    .get(),
                Trans3::new(950.0, 206.0, 0.)
            );

            let xx = vec![css(css_width)];

            root_e.refresh_use(&xx);
            root_e.refresh_use(&xx);
            root_e.refresh_use(&xx);
            root_e.refresh_use(&xx);
            root_e.refresh_use(&xx);
            root_e.refresh_use(&xx);
            root_e.refresh_use(&xx);
            root_e.refresh_use(&xx);
            root_e.refresh_use(&xx);
            root_e.refresh_use(&xx);
            root_e.refresh_use(&xx);
            root_e.refresh_use(&xx);
            root_e.refresh_use(&xx);
            root_e.refresh_use(&xx);
            root_e.refresh_use(&xx);
            root_e.refresh_use(&xx);
            root_e.refresh_use(&xx);
            root_e.refresh_use(&xx);
            root_e.refresh_use(&xx);
            root_e.refresh_use(&xx);
            root_e.refresh_use(&xx);
            root_e.refresh_use(&xx);
            root_e.refresh_use(&xx);
            root_e.refresh_use(&xx);
            root_e.refresh_use(&xx);
            root_e.refresh_use(&xx);
            root_e.refresh_use(&xx);
            root_e.refresh_use(&xx);
            root_e.refresh_use(&xx);
            root_e.refresh_use(&xx);
            root_e.refresh_use(&xx);
            root_e.refresh_use(&xx);
            root_e.refresh_use(&xx);
            root_e.refresh_use(&xx);
            root_e.refresh_use(&xx);

            // trace!("refresh_use after css_width {}", &root_e);
            trace!("refresh_use after css_width {}", &e1);
            info!("=========================================================");

            root_e.refresh_use(&Css(css_height));
            assert_eq!(
                e1.node
                    .get()
                    .get(&EPath(vector![EdgeIndex::new("root", "root")]))
                    .and_then(|x| x.as_edge_data())
                    .unwrap()
                    .calculated
                    .coordinates_trans
                    .get(),
                Trans3::new(40., 10., 0.)
            );
            info!("=========================================================");

            e1.refresh_use(&Css(CssWidth::from(px(12))));
            e1.refresh_use(&Css(CssWidth::from(px(12))));
            e1.refresh_use(&Css(CssWidth::from(px(12))));
            e1.refresh_use(&Css(CssWidth::from(px(12))));
            e1.refresh_use(&Css(CssWidth::from(px(12))));
            e1.refresh_use(&Css(CssWidth::from(px(12))));
            e1.refresh_use(&Css(CssWidth::from(px(12))));
            e1.refresh_use(&Css(CssWidth::from(px(12))));
            e1.refresh_use(&Css(CssWidth::from(px(12))));
            e1.refresh_use(&Css(CssWidth::from(px(12))));
            e1.refresh_use(&Css(CssWidth::from(px(12))));
            e1.refresh_use(&Css(CssWidth::from(px(12))));
            e1.refresh_use(&Css(CssWidth::from(px(12))));
            e1.refresh_use(&Css(CssWidth::from(px(12))));
            e1.refresh_use(&Css(CssWidth::from(px(12))));
            e1.refresh_use(&Css(CssWidth::from(px(12))));
            e1.refresh_use(&Css(CssWidth::from(px(12))));
            e1.refresh_use(&Css(CssWidth::from(px(12))));
            e1.refresh_use(&Css(CssWidth::from(px(12))));
            e1.refresh_use(&Css(CssWidth::from(px(12))));
            e1.refresh_use(&Css(CssWidth::from(px(12))));
            e1.refresh_use(&Css(CssWidth::from(px(12))));
            e1.refresh_use(&Css(CssWidth::from(px(12))));
            e1.refresh_use(&Css(CssWidth::from(px(12))));
            e1.refresh_use(&Css(CssWidth::from(px(12))));
            e1.refresh_use(&Css(CssWidth::from(px(12))));
            e1.refresh_use(&Css(CssWidth::from(px(12))));
            e1.refresh_use(&Css(CssWidth::from(px(12))));
            e1.refresh_use(&Css(CssWidth::from(px(12))));
            e1.refresh_use(&Css(CssWidth::from(px(12))));
            assert_eq!(
                e1.node
                    .get()
                    .get(&EPath(vector![EdgeIndex::new("root", "root")]))
                    .and_then(|x| x.as_edge_data())
                    .unwrap()
                    .calculated
                    .size
                    .get(),
                Size2::new(12., 10.)
            );
            info!("=========================================================");

            assert_eq!(
            e1.node.get().get(&EPath(vector![EdgeIndex::new("root", "root")]))
                    .and_then(|x| x.as_edge_data())
                .unwrap()
                .styles_string
                .get(),
            "width: 12px;\nheight: 10px;\ntransform: matrix3d(1,0,0,0,0,1,0,0,0,0,1,0,38,10,0,1);\n"
            );
            trace!("refresh_use after {}", &e1);
            // ─────────────────────────────────────────────────────────────────
            // ────────────────────────────────────────────────────────────────────────────────
            info!("=========================================================");

            trace!("refresh_use after {}", &e2);
            info!("=========================================================");
            e2.refresh_use(&Css(CssHeight::from(px(50))));
            assert_eq!(
                e2.node
                    .get()
                    .get(&EPath(vector![
                        EdgeIndex::new("root", "root"),
                        EdgeIndex::new("root", "e1")
                    ]))
                    .and_then(|x| x.as_edge_data())
                    .unwrap()
                    .calculated
                    .coordinates_trans
                    .get(),
                Trans3::new(-4.0, -48.0, 0.0)
            );
            let _span1 = span!(Level::TRACE, "debug print 1");
            _span1.in_scope(|| {
                trace!("refresh_use after {}", &e2);
            });

            let _span2 = span!(Level::TRACE, "debug print 2");
            _span2.in_scope(|| {
                trace!("refresh_use after2 {}", &e2);
            });
            info!("=========================================================");
            e2.refresh_use(&Css(CssHeight::from(px(150))));

            e2.id.set(EdgeIndex::new("xxx", "yyy"));
            trace!("refresh_use after {:#?}", &e2);
            info!("..=========================================================");
            trace!(
                "{}",
                e2.node
                    .get()
                    .get(&EPath(vector![
                        EdgeIndex::new("root", "root"),
                        EdgeIndex::new("root", "e1")
                    ]))
                    .and_then(|x| x.as_edge_data())
                    .unwrap()
                    .styles_string
                    .get(),
            );
            info!("..=========================================================");
            // trace!("end----- {:#?}", &root_e);
            // trace!("end----- {:#?}", &e1);
            // trace!("end----- {:#?}", &e2);

            // ─────────────────────────────────────────────────────────────────
        }
    }

    // #[test]
    // fn change_parent() {
    //     init();
    //     let e = EmgEdgeItem::new_root(100, 100);
    //     let e2 = EmgEdgeItem::new_root(200, 200);
    //     let ec = EmgEdgeItem::new_child(
    //         "e1",
    //         use_state(Some(e.clone())),
    //         size(px(10), px(10)),
    //         origin2(pc(0), pc(0)),
    //         align2(pc(50), pc(50)),
    //     );
    //     let ec2 = EmgEdgeItem::new_child(
    //         "e2",
    //         use_state(Some(ec.clone())),
    //         size(px(10), px(10)),
    //         origin2(pc(0), pc(0)),
    //         align2(pc(100), pc(000)),
    //     );

    //     assert_eq!(
    //         ec.as_current_edge_data()
    //             .unwrap()
    //             .calculated
    //             .coordinates_trans
    //             .get(),
    //         Trans3::new(50.0, 50.0, 0.)
    //     );

    //     ec.as_edge_data_with_parent()
    //         .unwrap()
    //         .parent
    //         .set(Some(e2.clone()));
    //     assert_eq!(
    //         ec.as_current_edge_data()
    //             .unwrap()
    //             .calculated
    //             .coordinates_trans
    //             .get(),
    //         Trans3::new(100.0, 100.0, 0.)
    //     );
    //     // ─────────────────────────────────────────────────────────────────
    //     //local
    //     assert_eq!(
    //         ec2.as_current_edge_data()
    //             .unwrap()
    //             .calculated
    //             .coordinates_trans
    //             .get(),
    //         Trans3::new(10.0, 00.0, 0.)
    //     );

    //     ec2.as_edge_data_with_parent()
    //         .unwrap()
    //         .parent
    //         .set(Some(e.clone()));

    //     // local use root
    //     assert_eq!(
    //         ec2.as_current_edge_data()
    //             .unwrap()
    //             .calculated
    //             .coordinates_trans
    //             .get(),
    //         Trans3::new(100.0, 0.0, 0.)
    //     );
    // }
    // #[test]
    // #[should_panic]
    // fn change_child_p_to_none() {
    //     init();
    //     let e = EmgEdgeItem::new_root(100, 100);
    //     let e2 = EmgEdgeItem::new_root(200, 200);
    //     let ec = EmgEdgeItem::new_child(
    //         "e1",
    //         use_state(Some(e.clone())),
    //         size(px(10), px(10)),
    //         origin2(pc(0), pc(0)),
    //         align2(pc(50), pc(50)),
    //     );
    //     let ec2 = EmgEdgeItem::new_child(
    //         "e2",
    //         use_state(Some(ec.clone())),
    //         size(px(10), px(10)),
    //         origin2(pc(0), pc(0)),
    //         align2(pc(100), pc(0)),
    //     );

    //     assert_eq!(
    //         ec.as_current_edge_data()
    //             .unwrap()
    //             .calculated
    //             .coordinates_trans
    //             .get(),
    //         Trans3::new(50.0, 50.0, 0.)
    //     );

    //     ec.as_edge_data_with_parent().unwrap().parent.set(None);
    //     assert_eq!(
    //         ec.as_current_edge_data()
    //             .unwrap()
    //             .calculated
    //             .coordinates_trans
    //             .get(),
    //         Trans3::new(100.0, 100.0, 0.)
    //     );
    //     // ─────────────────────────────────────────────────────────────────
    //     //local
    //     assert_eq!(
    //         ec2.as_current_edge_data()
    //             .unwrap()
    //             .calculated
    //             .coordinates_trans
    //             .get(),
    //         Trans3::new(10.0, 00.0, 0.)
    //     );

    //     ec2.as_edge_data_with_parent().unwrap().parent.set(None);

    //     // local use root
    //     assert_eq!(
    //         ec2.as_current_edge_data()
    //             .unwrap()
    //             .calculated
    //             .coordinates_trans
    //             .get(),
    //         Trans3::new(100.0, 0.0, 0.)
    //     );
    // }
}
