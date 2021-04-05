#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::used_underscore_binding)]
// // 固定 type  T:xxx
// #![feature(trivial_bounds)]
// #![feature(negative_impls)]
// #![feature(auto_traits)]

use std::{
    any::{Any, TypeId},
    ops::Deref,
    rc::Rc,
};

use calc::layout_calculating;
use derive_more::Display;
use derive_more::From;
use derive_more::Into;
use derive_more::TryInto;
use emg_refresh::{RefreshFor, RefreshUseFor};
use emg_state::{
    topo, use_state, CloneStateAnchor, CloneStateVar, StateAnchor, StateMultiAnchor, StateVar,
};
use na::{
    Affine3, Isometry3, Matrix4, Point3, Rotation3, Similarity3, Translation3, Vector2, Vector3,
};
use nalgebra as na;
pub use seed_styles as styles;
use styles::{
    pc, px, s, CssHeight, CssTransform, CssValueTrait, CssWidth, ExactLength, Percent, Style,
    UpdateStyle,
};
use styles::{CssHeightTrait, CssTransformTrait, CssWidthTrait};

//
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
#[derive(Display, Clone, Debug, From, TryInto, PartialEq, PartialOrd)]
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
impl Default for Layout {
    #[topo::nested]
    fn default() -> Self {
        Self {
            size: use_state(GenericWH::default()),
            origin: use_state(GenericLoc::default()),
            align: use_state(GenericLoc::default()),
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
#[derive(Display, Debug, Clone, PartialEq, PartialOrd)]
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
// #[display(
//     fmt = "Layout {{\nsize:{{\n{};\n}}\norigin:{{\n{};\n}}\nalign:{{\n{};\n}}\n}}",
//     "indented( size)",
//     "indented( origin)",
//     "indented( align)"
// )]
pub struct Layout {
    size: StateVar<GenericWH>,
    origin: StateVar<GenericLoc>,
    align: StateVar<GenericLoc>,
}
impl std::fmt::Display for Layout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let x = format!(
            "size:{{\n{};\n}}\norigin:{{\n{};\n}}\nalign:{{\n{};\n}}",
            indented(&self.size),
            indented(&self.origin),
            indented(&self.align)
        );
        write!(f, "Layout {{\n{}\n}}", indented(&x))
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
    id: StateVar<String>,
    layout: Layout,
    other_styles: StateVar<Style>,
    calculated: LayoutCalculated,
    styles_string: StateAnchor<String>, // matrix: M4Data,
                                        // transforms_am: Transforms,
                                        // animations:
}
impl std::fmt::Display for EdgeData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let x = format!(
            "id:{{\n{};\n}}\nlayout:{{\n{};\n}}\nother_styles:{{\n{};\n}}\ncalculated:{{\n{};\n}}",
            indented(&self.id),
            indented(&self.layout),
            indented(&self.other_styles),
            indented(&self.calculated)
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

#[derive(Display, Clone, Debug, PartialEq)]
#[display(
    fmt = "\n@E={{\n    parent-calc-size:{}\n    current:{{\n{};\n}}}}",
    "indented(get_parent_calc_size_to_string(parent))",
    "indented(indented(current))"
)]
pub struct EdgeDataWithParent {
    pub parent: StateVar<Option<EdgeItem>>,
    pub current: EdgeData,
}

fn try_get_parent_calc_size(parent:&StateVar<Option<EdgeItem>>)->Option<Size2>{
    parent.get().and_then(|ei|{ei.as_edge_data_with_parent().map(|ed|ed.current.calculated.size.get())})
}
fn get_parent_calc_size_to_string(parent:&StateVar<Option<EdgeItem>>)->String{
    match try_get_parent_calc_size(parent){
        Some(size) => {format!("w:{} h:{}",&size.x,&size.y)}
        None => {String::from("None")}
    }
}

#[derive(Display, From, Clone, Debug, PartialEq)]
pub enum EdgeItem {
    EdgeDataWithParent(EdgeDataWithParent),
    String(String),//TODO make can write
    Empty,
}

impl EdgeItem {
    #[must_use]
    pub const fn as_edge_data_with_parent(&self) -> Option<&EdgeDataWithParent> {
        if let Self::EdgeDataWithParent(v) = self {
            Some(v)
        } else {
            None
        }
    }

    #[must_use]
    pub fn as_current_edge_data(&self) -> Option<&EdgeData> {
        self.as_edge_data_with_parent().map(|v| &v.current)
    }

    #[topo::nested]
    #[instrument]
    pub fn new_root<T: Into<f64> + std::fmt::Debug>(w: T, h: T) -> Self {
        // info!(target: "yak_events", "Commencing yak shaving for ");
        let id: StateVar<String> = use_state(String::from("root"));

        let layout = Layout {
            size: use_state(GenericWH {
                w: px(w).into(),
                h: px(h).into(),
            }),
            origin: use_state(GenericLoc::default()),
            align: use_state(GenericLoc::default()),
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
                        %id).entered();
                    
                trace!("size: {}  , matrix: {}",size, CssTransform::from(*mat4));
                
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
        let styles_string = (&layout_calculated.loc_styles, &other_styles_sv.watch()).map(
           move |loc_styles: &Style, other_styles: &Style| {
                let _enter = span!(Level::TRACE,
                        "-> [root] [ styles ] recalculation..(&other_styles_watch, &loc_styles).map ",
                        %id).entered();
                // other 覆盖叠加到 loc
                loc_styles
                    .clone()
                    .custom_style(other_styles.clone())
                    .render()
            },
        );

        Self::EdgeDataWithParent(EdgeDataWithParent {
            parent: use_state(None),
            current: EdgeData {
                id,
                layout,
                other_styles: other_styles_sv,
                calculated: layout_calculated,
                styles_string,
            },
        })
    }

    #[topo::nested]
    pub fn new_child(
        id_like: impl Into<String>,
        parent_edge_item_sv: StateVar<Option<Self>>,
        size: impl Into<GenericWH>,
        origin: impl Into<GenericLoc>,
        align: impl Into<GenericLoc>,
    ) -> Self {
        
        let id_string:String = id_like.into();
        let id: StateVar<String> = use_state(id_string.clone());
        let _child_span = span!(Level::TRACE, " build new child ",id=%id_string).entered();
        let layout = Layout {
            size: use_state(size.into()),
            origin: use_state(origin.into()),
            align: use_state(align.into()),
        };
        let other_styles_sv = use_state(s());

        let layout_calculated = layout_calculating(id, &parent_edge_item_sv, &layout);


        let styles_string = (&layout_calculated.loc_styles, &other_styles_sv.watch()).map(
           move |loc_styles: &Style, other_styles: &Style| {
                let _enter = span!(Level::TRACE,
                        "-> [ styles ] recalculation..(&other_styles_watch, &loc_styles).map ",
                        %id).entered();
                // other 覆盖叠加到 loc
                loc_styles
                    .clone()
                    .custom_style(other_styles.clone())
                    .render()
            },
        );

        Self::EdgeDataWithParent(EdgeDataWithParent {
            parent: parent_edge_item_sv,
            current: EdgeData {
                id,
                layout,
                other_styles: other_styles_sv,
                calculated: layout_calculated,
                styles_string,
            },
        })
    }
}

#[topo::nested]
pub fn edge_item_data_with_parent(id: impl Into<String>, parent_edge_item_sv: StateVar<Option<EdgeItem>>) -> EdgeItem {
    EdgeItem::new_child(
        id,
        parent_edge_item_sv, //TODO maybe use Rc instead of clone?
        size(px(10), px(10)),
        origin2(pc(0), pc(0)),
        align2(pc(0), pc(0)),
    )
}

impl Default for EdgeItem {
    fn default() -> Self {
        Self::Empty
    }
}

// TODO lifetime
#[derive(Clone)]
pub struct Css<T>(T)
where
    T: CssValueTrait + Clone + 'static;

impl<T: std::clone::Clone + seed_styles::CssValueTrait> From<T> for Css<T> {
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
pub fn css<Use: CssValueTrait + Clone + 'static>(v: Use) -> Box<dyn RefreshFor<EdgeItem> >{
// pub fn css<Use: CssValueTrait + std::clone::Clone + 'static>(v: Use) -> Box<Css<Use>> {
    Box::new(Css(v))
}
fn get_current_edge_data(edge_item_sv: StateVar<Option<EdgeItem>>) -> EdgeData {
    edge_item_sv
        .get()
        .unwrap()
        .as_current_edge_data()
        .unwrap()
        .clone()
}
fn get_edge_parent(edge_item_sv: StateVar<Option<EdgeItem>>) -> StateVar<Option<EdgeItem>> {
    edge_item_sv
        .get()
        .unwrap()
        .as_edge_data_with_parent()
        .unwrap()
        .parent
}
#[cfg(test)]
mod tests {

    use seed_styles::CssWidth;
    use styles::CssWidthTrait;
    use tracing::{debug, info};

    use super::*;

    use tracing_flame::FlameLayer;
    use tracing_subscriber::{EnvFilter, fmt, prelude::*};

    fn setup_global_subscriber() ->impl Drop {
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
                    | tracing_subscriber::fmt::format::FmtSpan::CLOSE,);

        let (flame_layer, _guard) = FlameLayer::with_file("./tracing.folded").unwrap();

            // let subscriber = Registry::default().with(flame_layer);
        // tracing::subscriber::set_global_default(subscriber).expect("Could not set global default");

        let _s = tracing_subscriber::registry()
            .with(filter_layer)
            .with(fmt_layer)
            .with(flame_layer)
            
            .try_init();
        _guard
    }
    
    fn init() {

        let _el = env_logger::try_init();
      

        let _subscriber = tracing_subscriber::fmt()
            .with_test_writer()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .with_span_events(
                tracing_subscriber::fmt::format::FmtSpan::ACTIVE
                |tracing_subscriber::fmt::format::FmtSpan::ENTER
                    | tracing_subscriber::fmt::format::FmtSpan::CLOSE,
            )
            // .with_max_level(Level::TRACE)
            .try_init();
            
            

        // tracing::subscriber::set_global_default(subscriber)
        // .expect("setting default subscriber failed");

        

    }
    #[test]
    fn it_works() {
        // init();
         let _xx =  setup_global_subscriber();
{
        let span = span!(Level::TRACE, "start");
        let _guard = span.enter();

        info!("--------------------=====================================");
        let p = px(11);
        let s = s();
        let ff = s.w(p);
        // vec![ CssWidth::from(px(100))].up
        info!("=========================================================");

        // let cc = Transform9::identity();

        let css_width = CssWidth::from(px(100));
        let css_height = CssHeight::from(px(100));
        let mut e = EdgeItem::new_root(1920, 1080);
        let mut c1 = EdgeItem::new_child(
            "c1",
            use_state(Some(e.clone())),
            size(px(10), px(10)),
            origin2(pc(100), pc(100)),
            align2(pc(50), pc(20)),
        );
        let mut c2 = EdgeItem::new_child(
            "c2",
            use_state(Some(c1.clone())),
            size(px(10), px(10)),
            origin2(pc(100), pc(100)),
            align2(pc(50), pc(20)),
        );

        // debug!("refresh_use before {}", &ec);
        let _span = span!(Level::TRACE, "debug print ec");
        _span.in_scope(|| {
            trace!("refresh_use before {}", &c1);
        });
        // info!("=========================================================");

        assert_eq!(
            c1.as_current_edge_data()
                .unwrap()
                .calculated
                .coordinates_trans
                .get(),
            Trans3::new(950.0, 206.0, 0.)
        );

        let xx = vec![css(css_width)];

        e.refresh_use(&xx);
        // e.refresh_use(&css(css_width));

        debug!("refresh_use after css_width {}", &c1);
        // info!("=========================================================");

        e.refresh_use(&Css(css_height));
        assert_eq!(
            c1.as_current_edge_data()
                .unwrap()
                .calculated
                .coordinates_trans
                .get(),
            Trans3::new(40., 10., 0.)
        );

        // // debug!("refresh_use after coordinates_trans {}", ec.as_current_edge_data()
        // //         .unwrap()
        // //         .calculated
        // //         .get()
        // //         .coordinates_trans
        // //         .get());

        c1.refresh_use(&Css(CssWidth::from(px(12))));
        assert_eq!(
            c1.as_current_edge_data().unwrap().calculated.size.get(),
            Size2::new(12., 10.)
        );
      
        assert_eq!(
            c1.as_current_edge_data()
                .unwrap()
                .styles_string
                .get(),
            "width: 12px;\nheight: 10px;\ntransform: matrix3d(1,0,0,0,0,1,0,0,0,0,1,0,38,10,0,1);\n"
        );
        // debug!("refresh_use after {}", &c1);
        // ─────────────────────────────────────────────────────────────────
// ────────────────────────────────────────────────────────────────────────────────

        // debug!("refresh_use after {:#?}", &ec2);
        // info!("=========================================================");
        c2.refresh_use(&Css(CssHeight::from(px(50))));
        assert_eq!(
            c2.as_current_edge_data()
                .unwrap()
                .calculated
                .coordinates_trans
                .get(),
            Trans3::new(-4.0, -48.0, 0.0)
        );
        let _span1 = span!(Level::TRACE, "debug print 1");
        _span1.in_scope(|| {
        debug!("refresh_use after {}", &c2);
        });

         let _span2 = span!(Level::TRACE, "debug print 2");
        _span2.in_scope(|| {
        debug!("refresh_use after2 {}", &c2);
        });
        // info!("=========================================================");
        // ec2.refresh_use(&css(CssHeight::from(px(150))));

        // ec2.as_current_edge_data()
        //     .unwrap()
        //     .id
        //     .set(String::from("xxxxxxxx"));
        // debug!("refresh_use after {:#?}", &ec2);
        // info!("..=========================================================");
        // debug!(
        //     "{}",
        //     ec2.as_current_edge_data()
        //         .unwrap()
        //         .calculated
        //         .get()
        //         .style_string
        //         .get()
        // )

        // ─────────────────────────────────────────────────────────────────
        }
    }

    #[test]
    fn change_parent() {
        init();
        let e = EdgeItem::new_root(100, 100);
        let e2 = EdgeItem::new_root(200, 200);
        let ec = EdgeItem::new_child(
            "c1",
            use_state(Some(e.clone())),
            size(px(10), px(10)),
            origin2(pc(0), pc(0)),
            align2(pc(50), pc(50)),
        );
        let ec2 = EdgeItem::new_child(
            "c2",
            use_state(Some(ec.clone())),
            size(px(10), px(10)),
            origin2(pc(0), pc(0)),
            align2(pc(100), pc(000)),
        );

        assert_eq!(
            ec.as_current_edge_data()
                .unwrap()
                .calculated
                .coordinates_trans
                .get(),
            Trans3::new(50.0, 50.0, 0.)
        );

        ec.as_edge_data_with_parent()
            .unwrap()
            .parent
            .set(Some(e2.clone()));
        assert_eq!(
            ec.as_current_edge_data()
                .unwrap()
                .calculated
                .coordinates_trans
                .get(),
            Trans3::new(100.0, 100.0, 0.)
        );
        // ─────────────────────────────────────────────────────────────────
        //local
        assert_eq!(
            ec2.as_current_edge_data()
                .unwrap()
                .calculated
                .coordinates_trans
                .get(),
            Trans3::new(10.0, 00.0, 0.)
        );

        ec2.as_edge_data_with_parent()
            .unwrap()
            .parent
            .set(Some(e.clone()));

        // local use root
        assert_eq!(
            ec2.as_current_edge_data()
                .unwrap()
                .calculated
                .coordinates_trans
                .get(),
            Trans3::new(100.0, 0.0, 0.)
        );
    }
    #[test]
    #[should_panic]
    fn change_child_p_to_none() {
        init();
        let e = EdgeItem::new_root(100, 100);
        let e2 = EdgeItem::new_root(200, 200);
        let ec = EdgeItem::new_child(
            "c1",
            use_state(Some(e.clone())),
            size(px(10), px(10)),
            origin2(pc(0), pc(0)),
            align2(pc(50), pc(50)),
        );
        let ec2 = EdgeItem::new_child(
            "c2",
            use_state(Some(ec.clone())),
            size(px(10), px(10)),
            origin2(pc(0), pc(0)),
            align2(pc(100), pc(0)),
        );

        assert_eq!(
            ec.as_current_edge_data()
                .unwrap()
                .calculated
                .coordinates_trans
                .get(),
            Trans3::new(50.0, 50.0, 0.)
        );

        ec.as_edge_data_with_parent().unwrap().parent.set(None);
        assert_eq!(
            ec.as_current_edge_data()
                .unwrap()
                .calculated
                .coordinates_trans
                .get(),
            Trans3::new(100.0, 100.0, 0.)
        );
        // ─────────────────────────────────────────────────────────────────
        //local
        assert_eq!(
            ec2.as_current_edge_data()
                .unwrap()
                .calculated
                .coordinates_trans
                .get(),
            Trans3::new(10.0, 00.0, 0.)
        );

        ec2.as_edge_data_with_parent().unwrap().parent.set(None);

        // local use root
        assert_eq!(
            ec2.as_current_edge_data()
                .unwrap()
                .calculated
                .coordinates_trans
                .get(),
            Trans3::new(100.0, 0.0, 0.)
        );
    }
}
