#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::non_ascii_literal)]
#![allow(clippy::used_underscore_binding)]
//for display attr

// ────────────────────────────────────────────────────────────────────────────────
#![feature(specialization)]
#![feature(more_qualified_paths)]
// #![feature(min_specialization)]
#![feature(auto_traits)]
#![feature(negative_impls)]
#![feature(iter_intersperse)]
#![feature(let_chains)]
// ────────────────────────────────────────────────────────────────────────────────
#![feature(test)]
// ────────────────────────────────────────────────────────────────────────────────

// // 固定 type  T:xxx
// #![feature(trivial_bounds)]
// #![feature(negative_impls)]
// #![feature(auto_traits)]

use cassowary::{Constraint, Solver, Variable, WeightedRelation};
use ccsa::{CCSSEqExpression, CassowaryGeneralMap, CassowaryMap, ScopeViewVariable, CCSS};
use emg_hasher::CustomHasher;
use std::fmt::Write;
use std::{cell::RefCell, clone::Clone, cmp::Eq, hash::BuildHasherDefault, rc::Rc, time::Duration};

use calc::layout_calculating;
use derive_more::Display;
use derive_more::From;
use derive_more::Into;
// use derive_more::TryInto;
use emg::{Edge, NodeIndex};

use emg_common::{
    display::{DictDisplay, HashMapDisplay},
    im::{
        self,
        ordmap::{self, NodeDiffItem},
        vector, HashMap, HashSet, OrdSet,
    },
    na::{Affine3, Matrix4, Rotation3, Translation3, Vector2, Vector3},
    num_traits::cast,
    GenericSize, IdStr, LayoutOverride, NotNan, Precision, RectLTRB, TypeName, Vector, VectorDisp,
};
use emg_shaping::{EqShapingWithDebug, Shaping};
use emg_state::{
    state_lit::StateVarLit, state_store, topo, use_state, use_state_impl::Engine, Anchor,
    CloneStateAnchor, CloneStateVar, Dict, GStateStore, StateAnchor, StateMultiAnchor, StateVar,
};
use float_cmp::approx_eq;
use styles::{px, s, w, CssTransform, CssValueTrait, Style, UpdateStyle};
// use styles::Percent;
// use styles::ExactLength;
// use styles::CssWidth;
// use styles::CssHeight;
use styles::{CssHeightTrait, CssTransformTrait, CssWidthTrait};
//
// ────────────────────────────────────────────────────────────────────────────────

use crate::ccsa::svv_process::{eq_opt_sw_to_weighted_relation, svv_op_svvs_to_expr};
use indented::indented;
use tracing::{
    debug, debug_span, info, instrument, span, trace, trace_span, warn, warn_span, Level,
};

// ─────────────────────────────────────────────────────────────────────────────
mod calc;
mod css_calc_op;
mod epath;
mod impl_refresh;
mod parser;
// ────────────────────────────────────────────────────────────────────────────────
pub use epath::EPath;
pub use seed_styles as styles;
pub mod add_values;
pub mod animation;

pub use animation::AnimationE;
pub use emg::{node_index, EdgeIndex};
pub use emg_common;

pub mod ccsa;
pub type LayoutEndType = (Translation3<Precision>, Precision, Precision);
// ─────────────────────────────────────────────────────────────────────────────
pub mod ccsa_macro_prelude {

    pub use crate::ccsa;

    pub use emg_common as common;
}

// ────────────────────────────────────────────────────────────────────────────────

static CURRENT_PROP_WEIGHT: f64 = cassowary::strength::MEDIUM * 1.5;
static CHILD_PROP_WEIGHT: f64 = cassowary::strength::MEDIUM * 0.9;
pub const EDGES_POOL_SIZE: usize = 16;
pub const CHILDREN_POOL_SIZE: usize = 2;

// ────────────────────────────────────────────────────────────────────────────────

thread_local! {
    static G_CLOCK: StateVar<Duration> = use_state(||Duration::ZERO);
}

thread_local! {
    static G_ANIMA_RUNNING_STORE: StateVar<Vector<Anchor<bool>>> = use_state(Vector::new);
}
thread_local! {
    static G_AM_RUNING: StateAnchor<bool> = global_anima_running_build();
}
// ─────────────────────────────────────────────────────────────────────────────

pub fn global_anima_running_add(running: &StateAnchor<bool>) {
    G_ANIMA_RUNNING_STORE.with(|sv| sv.update(|v| v.push_back(running.get_anchor())));
}

#[must_use]
pub fn global_anima_running_sa() -> StateAnchor<bool> {
    G_AM_RUNING.with(std::clone::Clone::clone)
}
#[must_use]
pub fn global_anima_running() -> bool {
    G_AM_RUNING.with(emg_state::CloneStateAnchor::get)
}
#[must_use]
pub fn global_anima_running_build() -> StateAnchor<bool> {
    let watch: Anchor<Vector<bool>> = G_ANIMA_RUNNING_STORE.with(|am| {
        am.watch().anchor().then(|v: &Vector<Anchor<bool>>| {
            v.clone().into_iter().collect::<Anchor<Vector<bool>>>()
        })
    });
    watch.map(|list: &Vector<bool>| list.contains(&true)).into()
}
#[must_use]
pub fn global_clock() -> StateVar<Duration> {
    G_CLOCK.with(|c| *c)
}
pub fn global_clock_set(now: Duration) {
    G_CLOCK.with(|c| c.set(now));
}

// ────────────────────────────────────────────────────────────────────────────────
thread_local! {
    static G_WIDTH: StateVar<f64> = use_state(||0.);
}
#[must_use]
pub fn global_width() -> StateVar<f64> {
    G_WIDTH.with(|sv| *sv)
}
thread_local! {
    static G_HEIGHT: StateVar<f64> = use_state(||0.);
}
#[must_use]
pub fn global_height() -> StateVar<f64> {
    G_HEIGHT.with(|sv| *sv)
}

// ────────────────────────────────────────────────────────────────────────────────

#[derive(Display, Debug, PartialEq, PartialOrd, Copy, Clone, From, Into)]
struct Mat4(Matrix4<Precision>);

// type Mat4 = Matrix4<f64>;

// ────────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Display)]
//TODO use StateVar<StateAnchor> for can change? note: 了解 最终 和 cassowary 如何共同影响 尺寸
pub struct GenericSizeAnchor(StateAnchor<GenericSize>);

impl Default for GenericSizeAnchor {
    fn default() -> Self {
        Self(StateAnchor::constant(GenericSize::default()))
    }
}

impl std::ops::Deref for GenericSizeAnchor {
    type Target = StateAnchor<GenericSize>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
// ────────────────────────────────────────────────────────────────────────────────

impl ::core::ops::Mul<f64> for GenericSizeAnchor {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self {
        Self(self.0 * rhs)
    }
}
impl ::core::ops::Add for GenericSizeAnchor {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}
// ────────────────────────────────────────────────────────────────────────────────

pub auto trait NotStateAnchor {}
impl<T> !NotStateAnchor for StateAnchor<T> {}
pub auto trait NotStateVar {}
impl<T> !NotStateVar for StateVar<T> {}

impl<T> From<T> for GenericSizeAnchor
where
    T: NotStateAnchor + NotStateVar + Into<GenericSize>,
{
    fn from(v: T) -> Self {
        Self(StateAnchor::constant(v.into()))
        // Self(StateVarLit::new(v.into()).watch())
    }
}

impl<T> From<StateAnchor<T>> for GenericSizeAnchor
where
    T: NotStateAnchor + NotStateVar + Into<GenericSize> + Clone + 'static,
{
    fn from(v: StateAnchor<T>) -> Self {
        Self(v.map(|x| x.clone().into()))
    }
}
impl<T> From<StateVar<T>> for GenericSizeAnchor
where
    T: NotStateAnchor + NotStateVar + Into<GenericSize> + Clone + 'static,
{
    fn from(v: StateVar<T>) -> Self {
        // Self(v.watch().map(|x|x.clone().into()))
        Self(v.get_var_with(|v| v.watch().map(|x| x.clone().into()).into()))
    }
}

struct Transforms {
    loc: Translation3<f64>,
    scale: Vector3<f64>,
    rotate: Rotation3<f64>,
}
impl Default for Transforms {
    fn default() -> Self {
        Self {
            loc: Translation3::<f64>::identity(),
            scale: Vector3::<f64>::from_element(0.),
            rotate: Rotation3::<f64>::identity(),
        }
    }
}
#[derive(Debug, Clone)]
struct M4Data {
    m4: Affine3<f64>,
    m4_def: Affine3<f64>,
    layout_m4: Affine3<f64>,
    //TODO m4fDef:ED msg -> M4.Mat4
    world_inv: Affine3<f64>,
    layout_inv: Affine3<f64>,
    m4offset: Affine3<f64>,
}
impl Default for M4Data {
    fn default() -> Self {
        Self {
            m4: Affine3::<f64>::identity(),
            m4_def: Affine3::<f64>::identity(),
            layout_m4: Affine3::<f64>::identity(),
            //TODO m4fDef:ED msg -> M4.Mat4
            world_inv: Affine3::<f64>::identity(),
            layout_inv: Affine3::<f64>::identity(),
            m4offset: Affine3::<f64>::identity(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Layout {
    w: StateVar<GenericSizeAnchor>,
    h: StateVar<GenericSizeAnchor>,
    z: StateVar<StateAnchor<u64>>,
    origin_x: StateVar<GenericSizeAnchor>,
    origin_y: StateVar<GenericSizeAnchor>,
    origin_z: StateVar<GenericSizeAnchor>,
    align_x: StateVar<GenericSizeAnchor>,
    align_y: StateVar<GenericSizeAnchor>,
    align_z: StateVar<GenericSizeAnchor>,
    cassowary_constants: StateVar<StateAnchor<Vector<CCSS>>>,
    cassowary_selectors: StateVar<Vector<ScopeViewVariable>>,
    cassowary_generals: StateVar<CassowaryGeneralMap>,
}

impl Layout {
    // fn get(&self,prop:&str)->StateVar<GenericSizeAnchor>{
    //     match prop {
    //         "width" => self.w.clone(),
    //         "height" => self.h.clone(),
    //         "origin_x" => self.origin_x.clone(),
    //         "origin_y" => self.origin_y.clone(),
    //         "origin_z" => self.origin_z.clone(),
    //         "align_x" => self.align_x.clone(),
    //         "align_y" => self.align_y.clone(),
    //         "align_z" => self.align_z.clone(),
    //         _ => panic!("unknown prop {}",prop),
    //     }

    // }
    /// Set the layout's size.
    #[cfg(test)]
    fn set_size(&self, w: impl Into<GenericSizeAnchor>, h: impl Into<GenericSizeAnchor>) {
        self.w.set(w.into());
        self.h.set(h.into());
    }
    pub fn store_set_size(
        &self,
        store: &GStateStore,
        w: impl Into<GenericSizeAnchor>,
        h: impl Into<GenericSizeAnchor>,
    ) {
        self.store_set_w(store, w);
        self.store_set_h(store, h);
    }

    pub fn store_set_w(&self, store: &GStateStore, w: impl Into<GenericSizeAnchor>) {
        self.w.store_set(store, w.into());
    }
    pub fn store_set_h(&self, store: &GStateStore, h: impl Into<GenericSizeAnchor>) {
        self.h.store_set(store, h.into());
    }
}
impl Copy for Layout {}

impl std::fmt::Display for Layout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut members = String::new();
        let mut wh = String::new();
        writeln!(wh, "w:{}", self.w)?;
        writeln!(wh, "h:{}", self.h)?;
        writeln!(members, "size:{{\n{}\n}}", indented(wh))?;
        let mut origin = String::new();
        writeln!(origin, "x:{}", self.origin_x)?;
        writeln!(origin, "y:{}", self.origin_y)?;
        writeln!(origin, "z:{}", self.origin_z)?;
        writeln!(members, "origin:{{\n{}\n}}", indented(origin))?;
        let mut align = String::new();
        writeln!(align, "x:{}", self.align_x)?;
        writeln!(align, "y:{}", self.align_y)?;
        writeln!(align, "z:{}", self.align_z)?;
        writeln!(members, "align:{{\n{}\n}}", indented(align))?;

        write!(f, "Layout {{\n{}\n}}", indented(members))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutCalculated {
    // suggest_size: StateAnchor<Vector2<f64>>,
    size_constraints: StateAnchor<Vec<Constraint>>,
    cassowary_inherited_generals_sa: StateAnchor<Rc<CassowaryGeneralMap>>,
    cass_or_calc_size: StateAnchor<Vector2<Precision>>,
    origin: StateAnchor<Translation3<Precision>>,
    align: StateAnchor<Translation3<Precision>>,
    translation: StateAnchor<Translation3<Precision>>,
    coordinates_trans: StateAnchor<Translation3<Precision>>,
    cass_trans: StateAnchor<Translation3<Precision>>,
    matrix: StateAnchor<Mat4>,
    loc_styles: StateAnchor<Style>,
    //TODO what different with EdgeCtx:world?
    world: StateAnchor<Translation3<Precision>>,
}

impl std::fmt::Display for LayoutCalculated {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut members = String::new();
        writeln!(members, "cass_or_calc_size: {}", self.cass_or_calc_size)?;
        writeln!(members, "origin: {}", self.origin)?;
        writeln!(members, "align: {}", self.align)?;
        writeln!(members, "translation: {}", self.translation)?;
        writeln!(members, "coordinates_trans: {}", self.coordinates_trans)?;
        writeln!(members, "cass_trans: {}", self.cass_trans)?;
        writeln!(members, "matrix: {}", self.matrix)?;
        writeln!(members, "loc_styles: {}", self.loc_styles)?;
        writeln!(members, "world: {}", self.world)?;

        write!(f, "LayoutCalculated {{\n{}}}", indented(members))
    }
}

pub struct EdgeCtx {
    pub styles_end: StateAnchor<StylesDict>,
    pub layout_end: StateAnchor<LayoutEndType>,
    pub world: StateAnchor<Translation3<Precision>>,
    pub children_layout_override: StateAnchor<Option<LayoutOverride>>,
}

impl EdgeCtx {
    #[cfg(feature = "debug")]
    #[must_use]
    pub fn to_layout_override(&self, nix: NodeIndex) -> StateAnchor<LayoutOverride> {
        (
            &self.world,
            &self.layout_end,
            &self.children_layout_override,
        )
            .map(move |world, (_, w, h), children_layout_override| {
                let rect = RectLTRB::from_origin_size(world.vector.xy().into(), *w, *h);

                let _span =
                    debug_span!("LayoutOverride", ?nix, func = "to_layout_override").entered();

                children_layout_override.as_ref().cloned().map_or_else(
                    || {
                        debug!("rect:{:#?}", &rect);
                        LayoutOverride::new(rect)
                    },
                    |mut lo| {
                        debug!("lo:{:#?}", &lo);
                        debug!("rect:{:#?}", &rect);

                        lo.underlay(Some(nix.index().clone()), rect);
                        lo
                    },
                )
            })
    }

    #[cfg(not(feature = "debug"))]
    #[must_use]
    pub fn to_layout_override(&self) -> StateAnchor<LayoutOverride> {
        (
            &self.world,
            &self.layout_end,
            &self.children_layout_override,
        )
            .map(|world, (_, w, h), children_layout_override| {
                let rect = RectLTRB::from_origin_size(world.vector.xy().into(), *w, *h);
                children_layout_override.as_ref().cloned().map_or_else(
                    || {
                        let _span = debug_span!("LayoutOverride", func = "to_layout_override-def")
                            .entered();
                        debug!(target:"to_layout_override",?rect);
                        LayoutOverride::new(rect)
                    },
                    |mut lo| {
                        let _span =
                            debug_span!("LayoutOverride", func = "to_layout_override").entered();
                        debug!(target:"to_layout_override", "lo:{:#?}", &lo);
                        debug!(target:"to_layout_override", "rect:{:#?}", &rect);

                        lo.underlay(rect);
                        lo
                    },
                )
            })
    }
}

impl PartialEq for EdgeCtx {
    fn eq(&self, other: &Self) -> bool {
        self.styles_end == other.styles_end && self.layout_end == other.layout_end
    }
}

impl Clone for EdgeCtx {
    fn clone(&self) -> Self {
        Self {
            styles_end: self.styles_end.clone(),
            layout_end: self.layout_end.clone(),
            world: self.world.clone(),
            children_layout_override: self.children_layout_override.clone(),
            // _phantom_data: std::marker::PhantomData::<RenderCtx>,
        }
    }
}

impl std::fmt::Debug for EdgeCtx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EdgeCtx")
            .field("styles_end", &self.styles_end as &dyn std::fmt::Debug)
            .field("layout_end", &self.layout_end as &dyn std::fmt::Debug)
            .field("world", &self.world as &dyn std::fmt::Debug)
            .field(
                "children_layout",
                &self.children_layout_override as &dyn std::fmt::Debug,
            )
            .finish()
    }
}
pub struct EdgeData {
    path_layout: StateAnchor<Layout>,
    calculated: LayoutCalculated,
    cassowary_map: Rc<CassowaryMap>,
    children_vars_sa: StateAnchor<HashSet<Variable, BuildHasherDefault<CustomHasher>>>,
    cassowary_calculated_vars: StateAnchor<Dict<Variable, (NotNan<Precision>, IdStr)>>,
    cassowary_calculated_layout: StateAnchor<(Option<Precision>, Option<Precision>)>,
    pub styles_string: StateAnchor<String>,
    // pub info_string: StateAnchor<String>,
    pub ctx: EdgeCtx,

    opt_p_calculated: Option<LayoutCalculated>, //TODO check need ? use for what?
                                                // matrix: M4Data,
                                                // transforms_am: Transforms,
                                                // animations:
}

impl std::fmt::Debug for EdgeData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EdgeData")
            .field("path_layout", &self.path_layout)
            .field("calculated", &self.calculated)
            .field("cassowary_map", &self.cassowary_map)
            .field("children_vars_sa", &self.children_vars_sa)
            .field("cassowary_calculated_vars", &self.cassowary_calculated_vars)
            .field(
                "cassowary_calculated_layout",
                &self.cassowary_calculated_layout,
            )
            .field("styles_string", &self.styles_string)
            .field("ctx", &self.ctx)
            .field("opt_p_calculated", &self.opt_p_calculated)
            .finish()
    }
}

impl Clone for EdgeData {
    fn clone(&self) -> Self {
        Self {
            path_layout: self.path_layout.clone(),
            calculated: self.calculated.clone(),
            cassowary_map: self.cassowary_map.clone(),
            children_vars_sa: self.children_vars_sa.clone(),
            cassowary_calculated_vars: self.cassowary_calculated_vars.clone(),
            cassowary_calculated_layout: self.cassowary_calculated_layout.clone(),
            styles_string: self.styles_string.clone(),
            ctx: self.ctx.clone(),
            opt_p_calculated: self.opt_p_calculated.clone(),
        }
    }
}
impl Eq for EdgeData {}

impl PartialEq for EdgeData {
    fn eq(&self, other: &Self) -> bool {
        self.path_layout == other.path_layout
            && self.calculated == other.calculated
            && self.cassowary_map == other.cassowary_map
            && self.children_vars_sa == other.children_vars_sa
            && self.cassowary_calculated_vars == other.cassowary_calculated_vars
            && self.cassowary_calculated_layout == other.cassowary_calculated_layout
            && self.styles_string == other.styles_string
            && self.ctx == other.ctx
            && self.opt_p_calculated == other.opt_p_calculated
    }
}

impl EdgeData {
    #[must_use]
    pub fn styles_string(&self) -> String {
        self.styles_string.get()
    }
    #[must_use]
    pub fn store_styles_string(&self, store: &GStateStore) -> String {
        self.styles_string.store_get(store)
    }
    #[must_use]
    pub fn engine_styles_string(&self, engine: &mut Engine) -> String {
        engine.get(self.styles_string.anchor())
    }
}

impl std::fmt::Display for EdgeData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut members = String::new();
        writeln!(members, "calculated: {}", &self.calculated)?;
        writeln!(
            members,
            "styles_string: String {{\n{}\n}}",
            indented(&self.styles_string)
        )?;
        //TODO full this members
        writeln!(members, "more... ")?;

        write!(f, "EdgeData {{\n{}\n}}", indented(members))
    }
}

impl From<Mat4> for CssTransform {
    fn from(Mat4(matrix): Mat4) -> Self {
        Self::from(format!(
            "matrix3d({},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{})",
            &matrix.m11,
            &matrix.m21,
            &matrix.m31,
            &matrix.m41,
            &matrix.m12,
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
    Self: Into<T>,
    T: UpdateStyle<T>,
{
    fn update_style(self, style: &mut Style) {
        self.into().update_style(style);
    }
}

pub type GraphEdgesDict = Dict<EdgeIndex, Edge<EmgEdgeItem>>;
// use ahash::AHasher as CustomHasher;
// use rustc_hash::FxHasher as CustomHasher;

type PathVarMap<T> = HashMap<EPath, T, BuildHasherDefault<CustomHasher>>;
//TODO use GenericSizeAnchor for kv's -> v
pub type StylesDict =
    Dict<TypeName, StateAnchor<Rc<dyn EqShapingWithDebug<emg_native::WidgetState>>>>;

pub struct EmgEdgeItem {
    //TODO save g_store
    pub id: StateVar<StateAnchor<EdgeIndex>>, // dyn by Edge(source_nix , target_nix)
    pub paths: DictPathEiNodeSA,              // with parent self  // current not has current node
    pub layout: Layout,
    pub styles: StateVar<StylesDict>,
    path_styles: StateVar<PathVarMap<Style>>, //TODO check use
    path_layouts: StateVar<PathVarMap<Layout>>, // layout only for one path

    pub other_css_styles: StateVar<Style>,
    // no self  first try
    pub edge_nodes: DictPathEiNodeSA, //TODO with self?  not with self?  (current with self)
    store: Rc<RefCell<GStateStore>>,
}

impl std::fmt::Display for EmgEdgeItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut members = String::new();

        writeln!(members, "id: {}", self.id)?;

        writeln!(
            members,
            "paths: {}",
            DictDisplay("⚓<Dict>", self.paths.get())
        )?;
        writeln!(members, "layout: {}", self.layout)?;
        writeln!(members, "styles: ⚓{:?}", self.styles.get())?;

        writeln!(
            members,
            "path_styles: {}",
            HashMapDisplay("\u{2726}<HashMap>", self.path_styles.get())
        )?;
        writeln!(
            members,
            "path_layouts: {}",
            HashMapDisplay("\u{2726}<HashMap>", self.path_layouts.get())
        )?;

        writeln!(members, "other_css_styles:{}", self.other_css_styles,)?;

        writeln!(
            members,
            "edge_nodes: {}",
            DictDisplay("⚓<Dict>", self.edge_nodes.get())
        )?;

        write!(f, "EmgEdgeItem {{\n{}\n}}", indented(members))
    }
}

impl Clone for EmgEdgeItem {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            paths: self.paths.clone(),
            layout: self.layout,
            styles: self.styles,
            path_styles: self.path_styles,
            path_layouts: self.path_layouts,
            other_css_styles: self.other_css_styles,
            edge_nodes: self.edge_nodes.clone(),
            store: self.store.clone(),
        }
    }
}

impl std::fmt::Debug for EmgEdgeItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EmgEdgeItem")
            .field("id", &self.id)
            .field("paths", &self.paths)
            .field("layout", &self.layout)
            .field("styles", &self.styles)
            .field("path_styles", &self.path_styles)
            .field("path_layouts", &self.path_layouts)
            .field("other_css_styles", &self.other_css_styles)
            .field("edge_nodes", &self.edge_nodes)
            .finish()
    }
}
impl Eq for EmgEdgeItem {}

impl PartialEq for EmgEdgeItem {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.paths == other.paths
            && self.layout == other.layout
            && self.path_styles == other.path_styles
            && self.other_css_styles == other.other_css_styles
            && self.edge_nodes == other.edge_nodes
    }
}

pub type DictPathEiNodeSA = StateAnchor<Dict<EPath, EdgeItemNode>>; //NOTE: EdgeData or something

impl EmgEdgeItem {
    #[cfg(test)]
    fn set_size(&self, w: impl Into<GenericSizeAnchor>, h: impl Into<GenericSizeAnchor>) {
        self.layout.set_size(w, h);
    }
    // pub fn store_set_size(
    //     &self,
    //     store: &GStateStore,
    //     w: impl Into<GenericSizeAnchor>,
    //     h: impl Into<GenericSizeAnchor>,
    // ) {
    //     self.layout.store_set_size(store, w, h);
    // }

    #[cfg(test)]
    #[must_use]
    fn edge_data(&self, key: &EPath) -> Option<EdgeData> {
        //TODO not get(), use ref
        self.edge_nodes
            .get()
            .get(key)
            .and_then(EdgeItemNode::as_edge_data)
            .cloned()
    }

    // #[must_use]
    // pub fn store_edge_data(&self,store:&GStateStore, key: &EPath) -> Option<EdgeData> {
    //     self.node.store_get(store)
    //         .get(key)
    //         .and_then(EdgeItemNode::as_edge_data).cloned()

    // }
    pub fn store_edge_data_with<F: FnOnce(Option<&EdgeData>) -> R, R>(
        &self,
        store: &GStateStore,
        key: &EPath,
        func: F,
    ) -> R {
        #[cfg(debug_assertions)]
        {
            let oo = self
                .edge_nodes
                .store_get_with(store, std::clone::Clone::clone);
            trace!("edge_nodes: {:#?}", &oo);
        }

        self.edge_nodes.store_get_with(store, |o| {
            func(o.get(key).and_then(EdgeItemNode::as_edge_data))
        })
    }
}

impl EmgEdgeItem {
    pub fn build_path_layout(&self, func: impl FnOnce(Layout) -> (EPath, Layout)) {
        let (path, layout) = func(self.layout);
        self.path_layouts
            .set_with_once(move |pls_map| pls_map.update(path, layout));
    }

    // #[topo::nested]
    // #[instrument(skip(edges))]
    // pub fn default_in_topo(
    //     source_node_nix_sa: StateAnchor<Option<NodeIndex>>,
    //     target_node_nix_sa: StateAnchor<Option<NodeIndex>>,
    //     edges: StateAnchor<GraphEdgesDict>,
    // ) -> Self {
    //     Self::new_in_topo(
    //         source_node_nix_sa,
    //         target_node_nix_sa,
    //         edges,
    //         (GenericSize::default(), GenericSize::default()),
    //         (
    //             GenericSize::default(),
    //             GenericSize::default(),
    //             GenericSize::default(),
    //         ),
    //         (
    //             GenericSize::default(),
    //             GenericSize::default(),
    //             GenericSize::default(),
    //         ),
    //     )
    // }

    #[cfg(test)]
    #[topo::nested]
    #[instrument(skip(edges))]
    pub fn default_with_wh_in_topo<
        T: emg_common::num_traits::AsPrimitive<Precision> + std::fmt::Debug,
    >(
        source_node_nix_sa: StateAnchor<Option<NodeIndex>>,
        target_node_nix_sa: StateAnchor<Option<NodeIndex>>,
        edges: StateAnchor<GraphEdgesDict>,
        w: T,
        h: T,
    ) -> Self {
        Self::new_in_topo(
            source_node_nix_sa,
            target_node_nix_sa,
            edges,
            (px(w), px(h)),
            (
                GenericSize::default(),
                GenericSize::default(),
                GenericSize::default(),
            ),
            (
                GenericSize::default(),
                GenericSize::default(),
                GenericSize::default(),
            ),
        )
    }
    #[topo::nested]
    #[instrument(skip_all)]
    pub fn new_in_topo(
        source_node_nix_sa: StateAnchor<Option<NodeIndex>>,
        target_node_nix_sa: StateAnchor<Option<NodeIndex>>,
        edges: StateAnchor<GraphEdgesDict>,
        size: (impl Into<GenericSizeAnchor>, impl Into<GenericSizeAnchor>),
        origin: (
            impl Into<GenericSizeAnchor>,
            impl Into<GenericSizeAnchor>,
            impl Into<GenericSizeAnchor>,
        ),
        align: (
            impl Into<GenericSizeAnchor>,
            impl Into<GenericSizeAnchor>,
            impl Into<GenericSizeAnchor>,
        ),
    ) -> Self
    where
        (IdStr, NotNan<Precision>): PartialEq,
    {
        let id_sa: StateAnchor<EdgeIndex> =
            (&source_node_nix_sa, &target_node_nix_sa).map(|s, t| {
                let _g = span!(
                    Level::TRACE,
                    "[ id_sa recalculation ]:source_node_nix_sa/target_node_nix_sa change "
                )
                .entered();

                EdgeIndex::new(s.clone(), t.clone())
            });
        let id_sv = use_state(|| id_sa);
        let _child_span = trace_span!(" building new child ",id=?id_sv).entered();
        // ─────────────────────────────────────────────────────────────────

        let layout = Layout {
            w: use_state(|| size.0.into()),
            h: use_state(|| size.1.into()),
            z: use_state(|| StateAnchor::constant(0)),
            origin_x: use_state(|| origin.0.into()),
            origin_y: use_state(|| origin.1.into()),
            origin_z: use_state(|| origin.2.into()),
            align_x: use_state(|| align.0.into()),
            align_y: use_state(|| align.1.into()),
            align_z: use_state(|| align.2.into()),
            cassowary_constants: use_state(|| StateAnchor::constant(vector![])),
            cassowary_selectors: use_state(|| vector![]),
            cassowary_generals: use_state(CassowaryGeneralMap::new),
        };

        // let path_styles= use_state(||Dict::unit(EPath::::default(), s()));
        let path_styles: StateVar<PathVarMap<Style>> = use_state(PathVarMap::default);
        let path_layouts: StateVar<PathVarMap<Layout>> = use_state(PathVarMap::default);

        let other_css_styles_sv = use_state(s);
        let styles_sv = use_state(Dict::new);

        let opt_self_source_node_nix_sa_re_get:StateAnchor<Option<NodeIndex>> = id_sv.watch().then(|eid_sa_inner|{
            let _g = trace_span!( "[ source_node_nix_sa_re_get recalculation ]:id_sv change ").entered();

            eid_sa_inner.map(|i:&EdgeIndex|{

                let _g = span!(Level::TRACE, "[ source_node_nix_sa_re_get recalculation ]:eid_sa_inner change ",edge_index=?i).entered();

                i.source_nix().cloned()
            }).into()
        });

        let opt_self_target_node_nix_sa_re_get: StateAnchor<Option<NodeIndex>> =
            id_sv.watch().then(|eid_sa_inner| {
                eid_sa_inner
                    .map(|i: &EdgeIndex| i.target_nix().cloned())
                    .into()
            });

        let edges2 = edges.clone();

        let parent_paths: DictPathEiNodeSA =
            opt_self_source_node_nix_sa_re_get.then(move|opt_self_source_nix:&Option<NodeIndex>| {

                let _g = span!(Level::TRACE, "[ source_node_incoming_edge_dict_sa recalculation ]:source_node_nix_sa_re_get change ").entered();

                // if opt_self_source_nix.is_none(){
                //     //NOTE 如果 source nix  是没有 node index 那么他就是无上一级的
                //     Anchor::constant(Dict::<EPath, EdgeItemNode>::unit(EPath::::default(), EdgeItemNode::Empty))
                //     //TODO check why use unit? answer:need for `EdgeItemNode::Empty => path_ein_empty_node_builder`
                //     // Anchor::constant(Dict::<EPath, EdgeItemNode>::new())
                // }else{
                    opt_self_source_nix.clone().map_or_else(
                        ||Anchor::constant(Dict::<EPath, EdgeItemNode>::unit(EPath::default(), EdgeItemNode::Empty)),
                        |some_source_nix|{


                        edges.filter_map(EDGES_POOL_SIZE,move|someone_eix, e| {


                            debug!("********************** \n one_eix.target_node_ix: {:?} ?? opt_source_nix_clone:{:?}",someone_eix.target_nix(),&some_source_nix);
                            if   someone_eix.target_nix()? == &some_source_nix {

                                Some(e.item.edge_nodes.clone())

                            }else{
                                None
                            }

                        })
                        .anchor()
                        .then(|x:&Dict<EdgeIndex, DictPathEiNodeSA>|{

                            x.values().map(emg_state::StateAnchor::anchor)
                            .collect::<Anchor<Vector<_>>>()
                            .map(|v:&Vector<_>|{
                                let _g = trace_span!( "[  paths dict recalculation ]:vector paths change ").entered();
                                Dict::unions(v.clone())})
                        })
                    })

                // }



            });

        // NOTE children cassowary_map
        let children_nodes = opt_self_target_node_nix_sa_re_get.then(move |opt_self_target_nix| {
            // if opt_self_target_nix.is_none() {
            //     //NOTE 尾
            //     Anchor::constant(Dict::<EPath, EdgeItemNode>::default())
            // } else {
            // TODO  try  use node outgoing  find which is good speed? maybe make loop, because node in map/then will calculating
            // TODO ? let e = edges2.map(|e|e.get(Edge::default()));
            opt_self_target_nix.clone().map_or_else(
                || Anchor::constant(Dict::<EPath, EdgeItemNode>::default()),
                |self_target_nix| {
                    edges2
                        .filter_map(EDGES_POOL_SIZE, move |child_eix, v| {
                            //NOTE  edge source is self_target, this is children

                            if child_eix.source_nix()? == &self_target_nix {
                                Some(v.edge_nodes.clone())
                            } else {
                                None
                            }
                        })
                        .anchor()
                        .then(|x: &Dict<EdgeIndex, DictPathEiNodeSA>| {
                            x.values()
                                .map(emg_state::StateAnchor::anchor)
                                .collect::<Anchor<Vector<_>>>()
                                .map(|v: &Vector<_>| Dict::unions(v.clone()))
                        })
                },
            )
        });
        // ─────────────────────────────────────────────────────────────────

        //TODO not paths: StateVar<Dict<EPath,EdgeItemNode>>  use edgeIndex instead to Reduce memory
        let paths_clone = parent_paths.clone();
        let edge_nodes_sa:DictPathEiNodeSA = id_sv.watch().then(move|id_sa|{

            let paths_clone2 = paths_clone.clone();
            let children_nodes2 = children_nodes.clone();

            id_sa.then(move |eid:&EdgeIndex|{

                let children_nodes3 = children_nodes2.clone();

                let eid_clone = eid.clone();

                paths_clone2.map(move |p_node_as_paths:&Dict<EPath, EdgeItemNode>|{

                    p_node_as_paths.iter()
                        .map(|(parent_e_path, p_ei_node_v)| {
                            let mut p_ep_add_self = parent_e_path.clone();

                            //TODO node 可以自带 self nix ,下游不必每个子节点都重算

                            p_ep_add_self.push_back(eid_clone.clone());
                            (p_ep_add_self, p_ei_node_v.clone())
                        })
                        .collect::<Dict<EPath, EdgeItemNode>>()

                }).map_( 1,move |self_path:&EPath, p_path_edge_item_node:&EdgeItemNode| {

                    //@=====    each    path    edge    prosess     ============================================================================================================================================
                    //
                    //@ current var=>ix ix=>var cassowary_map
                    let current_cassowary_map = Rc::new(CassowaryMap::new());

                    let width_var  =current_cassowary_map.var("width").unwrap();
                    let height_var  =current_cassowary_map.var("height").unwrap();
                    let top_var  =current_cassowary_map.var("top").unwrap();
                    let left_var  =current_cassowary_map.var("left").unwrap();
                    let bottom_var  = current_cassowary_map.var("bottom").unwrap();
                    let right_var  = current_cassowary_map.var("right").unwrap();



                    let self_path2 =self_path.clone();
                    // let self_path3 =self_path.clone();
                    let self_path4 =self_path.clone();
                    let self_path5 =self_path.clone();
                    let self_path6 =self_path.clone();
                    let self_path7 =self_path.clone();

                    let _child_span =
                        span!(Level::TRACE, "[ node recalculation ]:paths change ").entered();

                        //TODO use Dict anchor Collection
                    let path_layout:StateAnchor<Layout> = path_layouts.watch().map(move|path_layouts_map:&PathVarMap< Layout>|{
                        // println!("--> id: {:?}", &id_sv);
                        trace!("--> finding path_layout in path_with_ed_node_builder------------------- len:{}",path_layouts_map.len());
                        // println!("--> layout:{:?}",pls_map.get(&path_clone));
                        //TODO path layout combine def layout
                        *path_layouts_map.get(&self_path2).unwrap_or(&layout)
                    });

                    //NOTE 约束
                    let ccss_list_sa = path_layout.then(|layout|{
                        layout.cassowary_constants.watch().then(|x|x.clone().into_anchor()).into_anchor()
                    });





                    let (opt_p_calculated,layout_calculated,layout_styles_string) =  match p_path_edge_item_node {
                        //NOTE 上一级节点: empty => 此节点是root
                        EdgeItemNode::Empty => path_ein_empty_node_builder(&path_layout, self_path,&current_cassowary_map,path_styles, other_css_styles_sv),
                        EdgeItemNode::EdgeData(ped)=> path_with_ed_node_builder(id_sv, ped, &path_layout, self_path, &current_cassowary_map,path_styles, other_css_styles_sv),
                        EdgeItemNode::String(_)  => {
                            todo!("parent is EdgeItemNode::String(_) not implemented yet");
                        }

                    };

                    //NOTE children cassowary_map
                    let (children_layout_override_sa_a,children_cass_maps_sa) = children_nodes3.filter_map(CHILDREN_POOL_SIZE,move |child_path,child_node|{
                        let _span = debug_span!("LayoutOverride",step=0, func = "will except_tail_match")
                                    .entered();
                                    // debug!("self_path3:{}",&self_path3);
                                    debug!("child_path:{:?}",&child_path);
                                    // debug!("match?:{}",&child_path.except_tail_match(&self_path3));

                        //NOTE remove this if ,because never not match
                        // if child_path.except_tail_match(&self_path3) {


                            if let (Some(nix), Some(ed)) = (child_path.last_target(),child_node.as_edge_data()) {
                                let _span = debug_span!("LayoutOverride",?nix,step=1, func = "will to_layout_override")
                                .entered();

                                Some( (ed.ctx.to_layout_override(
                                    #[cfg(feature = "debug")]
                                    nix.clone()
                                ).into_anchor(), nix.index().clone(),(ed.cassowary_map.clone(),ed.calculated.size_constraints.clone())))


                            } else {
                                //目前是没有 这种情况,但是如果有的话,看下是什么情况
                                unreachable!("not match (child_path.last_target(),child_node.as_edge_data())->{:?} ,{:?}",child_path.last_target(),child_node.as_edge_data());
                                // None
                            }
                        // }else{
                        //     panic!(" not child_path.except_tail_match(&self_path3)");
                        //     None
                        // }
                    })
                    .map(|x|{
                        #[cfg(feature = "debug")]
                        {
                                debug_span!("LayoutOverride",step=2,func = "EdgeItem new_in_top",info="to_layout_override之后 ... debug..").in_scope(||{
                                for (k,_) in x.iter(){
                                    debug!("child path---------:{:?}",k,);
                                }
                            });

                        }

                        let (children_as_layout_override_list, children_cass_maps) = x.values().cloned().fold((Vector::new(),Dict::new()),|(mut layout_override_vec,mut cass_dict),(layout_override,ix,cass_map)|{

                            #[cfg(feature = "debug")]
                            {
                                let ix2 =  ix.clone();

                                let layout_override = layout_override.map( move |x|{
                                    let ix3=  ix2.clone();

                                    (ix3,x.clone())
                                });
                                layout_override_vec.push_back(layout_override);

                            }
                            #[cfg(not(feature = "debug"))]
                            {
                                layout_override_vec.push_back(layout_override);
                            }

                            cass_dict.insert (ix,cass_map);
                            (layout_override_vec,cass_dict)
                        });
                        let children_layout_override = children_as_layout_override_list.into_iter().collect::<Anchor<Vector<_>>>().map(|los|{
                            let _span = debug_span!("LayoutOverride",step=3,func = "EdgeItem new_in_top",info="to_layout_override之后, children_layout_override end...").entered();

                            #[cfg(feature = "debug")]
                            {
                                for x in los.iter(){
                                    debug!(".... layout_override --- :{:#?}",x);
                                }

                                los.clone().into_iter().fold(Option::<LayoutOverride>::None,|acc,(ix,lo)|{
                                    debug!("ix:{:?}   acc:{:#?}",ix,&acc);
                                    debug!("lo:{:#?}",&lo);
                                        if let Some(old_lo) = acc {
                                            debug!("acc is some, will + ");
                                            let x = Some(old_lo + lo);
                                            debug!("comb---:{:#?}",&x);
                                            x

                                        }else{
                                        Some(lo)
                                        }
                                })

                            }

                            //TODO check same as use feature debug
                            #[cfg(not(feature = "debug"))]
                            {
                                los.clone().into_iter().reduce(|acc,lo|{
                                    // debug!("acc:{:#?}",&acc);
                                    // debug!("lo:{:#?}",&lo);
                                        acc + lo
                                })
                            }

                        });
                        (children_layout_override,children_cass_maps)


                    }).split();

                    let children_layout_override_sa = children_layout_override_sa_a.then(std::clone::Clone::clone);









                    //TODO 不要每一次变更 ccss_list ,都全部重新计算
                    //NOTE  [children_vars_sa] used for in child calc ,checkking if not has ,then the [var] is code added, not use add.
                    let (constant_sets_sa,children_vars_sa) = (&ccss_list_sa,&children_cass_maps_sa,&layout_calculated.cassowary_inherited_generals_sa).map(move | ccss_list,children_cass_maps,current_cassowary_inherited_generals|{

                            let _debug_span_ = warn_span!( "->[ constant_sets_sa calc then ] ").entered();

                            warn!("[constant_sets_sa] ccss_list:\n{}", VectorDisp(ccss_list.clone()));

                            let (constraint_sets_end,children_vars) = ccss_list.iter()
                            // .fold((OrdSet::<Constraint>::new(), HashSet::<Variable, BuildHasherDefault<CustomHasher>>::with_hasher(BuildHasherDefault::<CustomHasher>::default())),
                            .fold((OrdSet::<Constraint>::new(), HashSet::<Variable, BuildHasherDefault<CustomHasher>>::default() ),
                            |(  constraint_sets,mut children_vars0),CCSS{ svv_op_svvs,  eq_exprs, opt_sw }|{

                                if let (left_constraints,Some((left_expr,left_child_vars))) = svv_op_svvs_to_expr(svv_op_svvs,children_cass_maps,current_cassowary_inherited_generals){

                                    children_vars0 = children_vars0.union(left_child_vars);


                                    let (constraints2,_,children_vars2) = eq_exprs.iter().fold((constraint_sets.union(left_constraints),left_expr,children_vars0), |(mut constraints1,left_expr,children_vars1),CCSSEqExpression{ eq, expr }|{


                                        if let (right_constraints, Some((right_expr,right_child_vars))) = svv_op_svvs_to_expr(expr,children_cass_maps,current_cassowary_inherited_generals){

                                            let constraint = left_expr | eq_opt_sw_to_weighted_relation(*eq,opt_sw)| right_expr.clone();

                                            constraints1.insert(constraint);

                                            (constraints1.union(right_constraints),right_expr,children_vars1.union(right_child_vars))

                                        }else{

                                            (constraints1,left_expr,children_vars1)

                                        }

                                    });
                                    (constraints2,children_vars2)
                                }else{
                                    (constraint_sets,children_vars0)
                                }

                            });
                            warn!("[constant_sets_sa] \n {:#?} \n constant_sets:\n{:#?}", &self_path7,&constraint_sets_end);


                            // constraints_sa.into_iter().collect::<Anchor<Vector<Vec<Constraint>>>>()
                            // .map(move|size_constraints_vector|{
                            //     let mut x = constant_sets.clone();
                            //     x.extend(size_constraints_vector.clone().into_iter().flatten());
                            //     (x,children_vars.clone())
                            // })

                            (constraint_sets_end,children_vars)




                    }).split();

                    // let current_cassowary_inherited_generals_val_sa = layout_calculated.cassowary_inherited_generals_sa.map(|x|x.v_v);

                    let LayoutCalculated{cass_or_calc_size, origin, align ,..} = &layout_calculated;
                    //NOTE 层建议值 (层当前计算所得)
                    let current_calculated_prop_val_sa = ( cass_or_calc_size, origin, align ).map(|size, origin, align|{
                        let width = size.x;
                        let height = size.y;
                        let origin_x = origin.x;
                        let origin_y = origin.y;
                        let align_x = align.x;
                        let align_y = align.y;

                        //TODO real val because cassowary calc need suggestions???
                        //TODO bottom right use from bottom or from top??
                        let top=  0. as Precision;
                        let bottom = height;
                        let left = 0. as Precision;
                        let right = width;

                        //TODO 使用 variable => notnan f64 , 避免后续 计算 每次都要获取 variable.
                        im::ordmap!{
                            IdStr::new("width") => NotNan::new(width).unwrap(),
                            IdStr::new("height") => NotNan::new(height).unwrap(),
                            IdStr::new("origin_x") => NotNan::new(origin_x).unwrap(),
                            IdStr::new("origin_y") => NotNan::new(origin_y).unwrap(),
                            IdStr::new("align_x") => NotNan::new(align_x).unwrap(),
                            IdStr::new("align_y") => NotNan::new(align_y).unwrap(),
                            IdStr::new("top") => NotNan::new(top).unwrap(),
                            IdStr::new("bottom") => NotNan::new(bottom).unwrap(),
                            IdStr::new("left") => NotNan::new(left).unwrap(),
                            IdStr::new("right") => NotNan::new(right).unwrap()
                        }

                    });

                    // ────────────────────────────────────────────────────────────────────────────────
                    // let children_cass_maps_no_val_sa = children_cass_maps_sa.map_(|_ix,(map,..)|{
                    //     map.clone()
                    // });
                    let children_cass_size_constraints_sa = children_cass_maps_sa.then(|d|{
                        d.values().map(|(_,sa)|sa.get_anchor()).collect::<Vec<_>>().into_iter().collect::<Anchor<Vector<Vec<Constraint>>>>()
                    });
                    // let current_cassowary_map3 = current_cassowary_map.clone();

                    // let children_for_current_addition_constants_sa =  (&children_cass_maps_no_val_sa,&children_cass_size_constraints_sa).map(move |cass_maps,children_size_constraints|{
                    let children_for_current_addition_constants_sa =  children_cass_size_constraints_sa.map(move |children_size_constraints|{


                        let mut  res_exprs = OrdSet::new();

                        //NOTE add some each child custom  cassowary map to current constants map

                        // for (_,map) in cass_maps {

                        //     res_exprs.extend([
                        //         // current_cassowary_map3.var("width").unwrap() | WeightedRelation::GE(cassowary::strength::REQUIRED) | map.var("left").unwrap()+map.var("width").unwrap(),
                        //         // current_cassowary_map3.var("height").unwrap() | WeightedRelation::GE(cassowary::strength::REQUIRED) | map.var("top").unwrap() + map.var("height").unwrap(),

                        //         // current_cassowary_map3.var("width").unwrap() | WeightedRelation::GE(cassowary::strength::WEAK) | map.var("right").unwrap(),
                        //         // current_cassowary_map3.var("height").unwrap() | WeightedRelation::GE(cassowary::strength::WEAK) | map.var("bottom").unwrap(),

                        //     ]);


                        // }

                        res_exprs.extend(children_size_constraints.clone().into_iter().flatten());


                        res_exprs


                    });
                    // ────────────────────────────────────────────────────────────────────────────────

                    let mut last_observation_constants:OrdSet<Constraint>  =  OrdSet::new();
                    let mut last_observation_current_props:Dict<IdStr, NotNan<Precision>> =  Dict::new();
                    let mut last_observation_children_for_current_constants :OrdSet<Constraint>  =  OrdSet::new();
                    let mut last_current_cassowary_inherited_general_vals: Dict<Variable, f64> = Dict::new();
                    let mut last_current_cassowary_top_general_vals: Dict<Variable, f64> = Dict::new();
                    // let mut last_virtual_constraints: Dict<IdStr, [Constraint; 10]> = Dict::new();
                    // let mut last_top_virtual_constraints: Dict<IdStr, [Constraint; 10]> = Dict::new();

                    let current_cassowary_map2 = current_cassowary_map.clone();
                    let mut cass_solver = Solver::new();



                    //NOTE current default constraints
                    cass_solver.add_constraints([
                        //NOTE add for current cassowary
                        width_var | WeightedRelation::GE(cassowary::strength::REQUIRED) | 0.0,
                        height_var | WeightedRelation::GE(cassowary::strength::REQUIRED) | 0.0,
                        // width_var | WeightedRelation::EQ(cassowary::strength::REQUIRED) | right_var - left_var,
                        // height_var | WeightedRelation::EQ(cassowary::strength::REQUIRED) | bottom_var - top_var,

                        top_var | WeightedRelation::EQ(cassowary::strength::REQUIRED) | 0.0,
                        left_var | WeightedRelation::EQ(cassowary::strength::REQUIRED) | 0.0,
                        bottom_var | WeightedRelation::EQ(cassowary::strength::REQUIRED) | height_var,
                        right_var | WeightedRelation::EQ(cassowary::strength::REQUIRED) | width_var
                    ]).unwrap();

                    let calculated_changed_vars_sa  =
                        (&children_for_current_addition_constants_sa,&constant_sets_sa,&current_calculated_prop_val_sa,&layout_calculated.cassowary_inherited_generals_sa)
                        .map_mut( Dict::<Variable,NotNan<f64>>::new(),move |out,children_for_current_addition_constants,newest_constants,newest_current_prop_vals,current_cassowary_inherited_generals| {


                            let _debug_span_ = warn_span!( "->[ calculated_changed_vars_sa calc map_mut ] ").entered();
                            warn!("[calculated_changed_vars_sa] path:{:?} newest_current_prop_vals :{:?}",&self_path5,&newest_current_prop_vals);

                            let mut children_for_current_constants_did_update = false;


                            if children_for_current_addition_constants.is_empty() && !last_observation_children_for_current_constants.is_empty(){
                                for constant in last_observation_children_for_current_constants.iter(){
                                    cass_solver.remove_constraint(constant).unwrap();

                                }
                                last_observation_children_for_current_constants.clear();
                                children_for_current_constants_did_update=true;
                            }else{
                                for diff_item in last_observation_children_for_current_constants.diff(children_for_current_addition_constants){
                                    match diff_item{
                                        NodeDiffItem::Add(new) => {
                                            cass_solver.add_constraint(new.clone()).unwrap();
                                            children_for_current_constants_did_update = true;
                                        },
                                        NodeDiffItem::Update { old, new } => {
                                            cass_solver.remove_constraint(old).unwrap();
                                            cass_solver.add_constraint(new.clone()).unwrap();
                                            children_for_current_constants_did_update = true;

                                        },
                                        NodeDiffItem::Remove(old) => {
                                            cass_solver.remove_constraint(old).unwrap();
                                            children_for_current_constants_did_update = true;
                                        },
                                    }
                                }
                                last_observation_children_for_current_constants = children_for_current_addition_constants.clone();

                            }

                            let mut constants_did_update = false;

                            if newest_constants.is_empty() && !last_observation_constants.is_empty() {
                                for constant in last_observation_constants.iter() {
                                    cass_solver.remove_constraint(constant).unwrap();
                                }
                                last_observation_constants.clear();
                                constants_did_update = true;
                            }else{
                                for diff_item in last_observation_constants.diff(newest_constants){
                                    match diff_item {
                                        NodeDiffItem::Add(x) => {
                                            cass_solver.add_constraint(x.clone()).ok();//may duplicate constants
                                            constants_did_update = true;
                                        },
                                        NodeDiffItem::Update { old, new } => {
                                            cass_solver.remove_constraint(old).unwrap();
                                            cass_solver.add_constraint(new.clone()).unwrap();
                                            constants_did_update = true;
                                        },
                                        NodeDiffItem::Remove(old) => {
                                            cass_solver.remove_constraint(old).unwrap();
                                            constants_did_update = true;
                                        } ,
                                    };

                                };
                                if constants_did_update {
                                    last_observation_constants = newest_constants.clone();
                                }

                            }

                            // @ current cassowary Top general vals ────────────────────────────────────────────────────────────────────────────────
                            let mut general_top_vals_did_update = false;


                            //TODO optimization, no need all val.
                            for diff_item in last_current_cassowary_top_general_vals.diff(&current_cassowary_inherited_generals.top_v_v_suggest){

                                match diff_item {
                                    ordmap::DiffItem::Add(&var, &v) => {
                                        warn!("path:{:?} , cass_solver add v:{}",&self_path6,&v);
                                        cass_solver.add_edit_variable(var, cassowary::strength::STRONG*1000.0).ok();
                                        cass_solver.suggest_value(var, v ).unwrap();
                                        general_top_vals_did_update = true;

                                    },
                                    ordmap::DiffItem::Update { old:(&old_var,_old_v), new:(&var,&v) } => {
                                        //TODO check, remove .
                                        assert_eq!(old_var,var);
                                        cass_solver.add_edit_variable(var, cassowary::strength::STRONG*1000.0).ok();
                                        cass_solver.suggest_value(var, v).unwrap();
                                        general_top_vals_did_update = true;

                                    },
                                    ordmap::DiffItem::Remove(_, _) => {
                                        panic!("current Top general vals never remove (current now)")
                                    },
                                };

                            };
                            if general_top_vals_did_update{
                                last_current_cassowary_top_general_vals = current_cassowary_inherited_generals.top_v_v_suggest.clone();
                            }
                            // @ current cassowary inherited general vals ────────────────────────────────────────────────────────────────────────────────
                            let mut general_inherited_vals_did_update = false;


                            //TODO optimization, no need all val.
                            for diff_item in last_current_cassowary_inherited_general_vals.diff(&current_cassowary_inherited_generals.v_v_suggest){

                                match diff_item {
                                    ordmap::DiffItem::Add(&var, &v) => {
                                        warn!("path:{:?} , cass_solver add v:{}",&self_path6,&v);
                                        cass_solver.add_edit_variable(var, cassowary::strength::STRONG*1000.0).ok();
                                        cass_solver.suggest_value(var, v ).unwrap();
                                        general_inherited_vals_did_update = true;

                                    },
                                    ordmap::DiffItem::Update { old:(&old_var,_old_v), new:(&var,&v) } => {
                                        //TODO check, remove .
                                        assert_eq!(old_var,var);
                                        cass_solver.add_edit_variable(var, cassowary::strength::STRONG*1000.0).ok();
                                        cass_solver.suggest_value(var, v).unwrap();
                                        general_inherited_vals_did_update = true;

                                    },
                                    ordmap::DiffItem::Remove(_, _) => {
                                        panic!("current inherited general vals never remove (current now)")
                                    },
                                };

                            };
                            if general_inherited_vals_did_update{
                                last_current_cassowary_inherited_general_vals = current_cassowary_inherited_generals.v_v_suggest.clone();
                            }


                            // ────────────────────────────────────────────────────────────────────────────────
                            let mut prop_vals_did_update = false;


                            info!("current_cassowary_map2===== \n all= \n{:?}",&current_cassowary_map2.map);

                            for diff_item in last_observation_current_props.diff(newest_current_prop_vals){
                                info!("current_cassowary_map2 \n all= \n{:?}",&current_cassowary_map2.map);

                                match diff_item {
                                    ordmap::DiffItem::Add(prop, v) => {
                                        //TODO use option , not this
                                        match prop.as_str() {
                                            "width" | "height" | "bottom" | "right" if approx_eq!(Precision,v.into_inner(),0.0,(0.01,2)) => {
                                                    continue;
                                            }
                                            _ => {}
                                        }


                                        info!("current props  add (maybe first time)");
                                        // panic!("current_cassowary_map2:want:{:?} \n all= \n{:?}",&prop,&current_cassowary_map2.map);
                                        let var = current_cassowary_map2.var(&**prop).unwrap();
                                        cass_solver.add_edit_variable(var, CURRENT_PROP_WEIGHT).ok();
                                        cass_solver.suggest_value(var, *v).unwrap();
                                        prop_vals_did_update = true;

                                    },
                                    ordmap::DiffItem::Update { old:(old_prop,_old_v), new:(prop,v) } => {
                                        //TODO check, remove .
                                        assert_eq!(old_prop,prop);
                                        let var = current_cassowary_map2.var(&**prop).unwrap();
                                        cass_solver.add_edit_variable(var, CURRENT_PROP_WEIGHT).ok();
                                        cass_solver.suggest_value(var, *v).unwrap();
                                        prop_vals_did_update = true;

                                    },
                                    ordmap::DiffItem::Remove(_, _) => {
                                        panic!("current props never remove (current now)")
                                    },
                                };

                            };
                            if prop_vals_did_update {
                                last_observation_current_props = newest_current_prop_vals.clone();
                            }

                            // ────────────────────────────────────────────────────────────────────────────────
                            if constants_did_update || prop_vals_did_update || general_inherited_vals_did_update || general_top_vals_did_update || children_for_current_constants_did_update{
                                let changes = cass_solver.fetch_changes();
                                // warn!("cass solver change:{:#?}",&changes);



                                if !changes.is_empty() {
                                    *out =  changes.into();
                                    return true
                                }
                            }

                            false

                        });
                    // ────────────────────────────────────────────────────────────────────────────────
                    let current_cassowary_map3 = current_cassowary_map.clone();
                    let cassowary_calculated_vars =  (&children_cass_maps_sa,&calculated_changed_vars_sa).map_mut(Dict::<Variable, (NotNan<Precision>,IdStr)>::new(),move|out,children_cass_maps,changed_vars|{
                        let _debug_span_ = warn_span!( "->[ calculated_vars calc map_mut ] ").entered();

                        if !changed_vars.is_empty() {
                            // warn!("[calculated_vars] changed_vars======== \n{:?}",&changed_vars);

                            //TODO remove get id if release
                            for (var,v) in changed_vars.iter() {
                                let id_prop_str =   children_cass_maps.iter().find_map(|(id,(cassowary_map ,..))|{
                                    cassowary_map.prop(var).map(|prop|{
                                        let vv:IdStr = format!("{} |=> #{:?}[{}]",&self_path4, &id,&prop).into();
                                        vv
                                    })
                                }).or_else(||{
                                    current_cassowary_map3.prop(var).map(|prop|{
                                        let vv:IdStr = format!("{}[{}] ",&self_path4,&prop).into();
                                        vv
                                    })
                                }).unwrap_or_default();//TODO add genal vals

                                warn!("[calculated_vars] changed  prop:{:?}  v:{}",&id_prop_str,&v);


                                out.insert(*var,(NotNan::new(cast(v.into_inner()).unwrap()).unwrap(),id_prop_str));


                            }
                            // warn!("[calculated_vars] total  prop:\n{:#?} ",&out);

                            return true
                        }
                        false

                    });



                    //TODO check diff with [calculated], because calculated_vars may re suggestion some value.
                    //TODO replace current_cassowary_map4.var("width") use   width
                    let cassowary_calculated_layout = cassowary_calculated_vars.map(move |cassowary_vars|{
                        debug!("[calculated_vars] [calculated_cassowary_layout] total  prop:\n{:#?} ",&cassowary_vars);


                        let w  =cassowary_vars.get(&width_var).map(|x|x.0.into_inner() as Precision);
                        let h  =cassowary_vars.get(&height_var).map(|x|x.0.into_inner() as Precision);
                        // let top = cassowary_vars.get(&current_cassowary_map4.var("top").unwrap()).map(|x|x.0.into_inner());
                        // let left = cassowary_vars.get(&current_cassowary_map4.var("left").unwrap()).map(|x|x.0.into_inner());
                        // let bottom = cassowary_vars.get(&current_cassowary_map4.var("bottom").unwrap()).map(|x|x.0.into_inner());
                        // let right = cassowary_vars.get(&current_cassowary_map4.var("right").unwrap()).map(|x|x.0.into_inner());
                        (w,h)
                    });

                    let info_string:StateAnchor<String> = cassowary_calculated_layout.map(|(width,height,)|{

                        let w = width.unwrap_or_default();
                        let h = height.unwrap_or_default();

                        let info = format!("w:{w} h:{h} ");
                        info

                    });

                    let layout_end:StateAnchor<LayoutEndType> = (&layout_calculated.translation,&cassowary_calculated_layout).map(|trans,(w,h)|
                        // (
                        //     Translation3::new(NotNan::new(trans.x ).unwrap(),NotNan::new(trans.y ).unwrap(),NotNan::new(trans.z ).unwrap() ) ,
                        //     w.and_then(|w|NotNan::new(w).ok()).unwrap(),
                        //     h.and_then(|h|NotNan::new(h).ok()).unwrap()
                        // )
                        (*trans,w.expect("width must have"),h.expect("height must have"))
                    );


                    let styles_string:StateAnchor<String> = (&info_string,&layout_styles_string, &cassowary_calculated_layout).map(move |info,layout_styles,(w,h)|{

                        debug!("[calculated_vars] [info] total  info:\n{} ",&info);


                        if let (Some(w), Some(h)) = (w,h){
                            format!(
                                "{} {}",
                                layout_styles,
                                s().w(px(*w)).h(px(*h)).render()
                            )
                        }else{
                            String::new()
                        }

                    });




                    // • • • • •



                    // ────────────────────────────────────────────────────────────────────────────────

                    let world = layout_calculated.world.clone();

                    EdgeItemNode::EdgeData(Box::new(EdgeData {
                        path_layout,
                        calculated: layout_calculated,
                        cassowary_map: current_cassowary_map,
                        children_vars_sa,
                        cassowary_calculated_vars,
                        cassowary_calculated_layout,
                        styles_string,
                        ctx:EdgeCtx{
                            styles_end:styles_sv.watch(),//TODO make real path_styles_sv
                            layout_end,
                            world,
                            children_layout_override:children_layout_override_sa,
                            // _phantom_data:std::marker::PhantomData::<RenderCtx>
                        },
                        opt_p_calculated,

                    }))
                }).into()

            }).into()
        });

        Self {
            id: id_sv,
            paths: parent_paths,
            layout,
            path_styles,
            path_layouts,
            other_css_styles: other_css_styles_sv,
            styles: styles_sv,
            edge_nodes: edge_nodes_sa,
            store: state_store(),
        }
    }
}

fn path_with_ed_node_builder(
    id_sv: StateVar<StateAnchor<EdgeIndex>>,
    ped: &EdgeData,
    path_layout: &StateAnchor<Layout>,
    path: &EPath,
    current_cassowary_map: &Rc<CassowaryMap>,
    path_styles: StateVar<PathVarMap<Style>>,
    other_styles_sv: StateVar<Style>,
) -> (
    Option<LayoutCalculated>,
    LayoutCalculated,
    StateAnchor<String>,
) {
    // println!("run path_with_ed_node_builder ******************************************************************");

    let p_calculated = ped.calculated.clone();
    let path_clone2 = path.clone();

    let layout_calculated = layout_calculating(id_sv, ped, current_cassowary_map, path_layout);
    // let p = path.clone();
    let this_path_style_string_sa: StateAnchor<Option<String>> =
        path_styles.watch().map(move |d: &PathVarMap<Style>| {
            let _g = trace_span!(
                "[  this_path_style_string_sa recalculation ]:layout.path_styles change "
            )
            .entered();

            d.get(&path_clone2).map(seed_styles::Style::render)
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
                                "-> [ styles ] recalculation..(&other_styles_watch,&loc_styles &loc_styles).map",
                                )
                                .entered();

                                format!(
                                    "{}{}{}",
                                    other_styles.render(),
                                    path_styles_string.as_ref().unwrap_or(&String::default()),
                                    loc_styles.render()
                                )
                            },
                        );
    (Some(p_calculated), layout_calculated, styles_string)
}

fn path_ein_empty_node_builder(
    path_layout: &StateAnchor<Layout>,
    _path: &EPath,
    current_cassowary_map: &Rc<CassowaryMap>,

    path_styles: StateVar<PathVarMap<Style>>,
    other_styles_sv: StateVar<Style>,
) -> (
    Option<LayoutCalculated>,
    LayoutCalculated,
    StateAnchor<String>,
) {
    // println!("run path_ein_empty_node_builder ******************************************************************");

    // ─────────────────────────────────────────────────────────────────
    // let path_clone = path.clone();

    let w = path_layout.then(|l: &Layout| l.w.watch().into());
    let h = path_layout.then(|l: &Layout| l.h.watch().into());
    let current_cassowary_generals_sa = path_layout.then(|l| l.cassowary_generals.watch().into());

    // let origin_x = path_layout.then(|l:&Layout|l.origin_x.watch().into());
    // let origin_y = path_layout.then(|l:&Layout|l.origin_y.watch().into());
    // let align_x = path_layout.then(|l:&Layout|l.align_x.watch().into());
    // let align_y = path_layout.then(|l:&Layout|l.align_y.watch().into());
    // ─────────────────────────────────────────────────────────────────
    let sa_w = w.then(|w| w.get_anchor());
    let sa_h = h.then(|h| h.get_anchor());
    let width_var = current_cassowary_map.var("width").unwrap();
    let height_var = current_cassowary_map.var("height").unwrap();
    let top_var = current_cassowary_map.var("top").unwrap();
    let left_var = current_cassowary_map.var("left").unwrap();
    let bottom_var = current_cassowary_map.var("bottom").unwrap();
    let right_var = current_cassowary_map.var("right").unwrap();
    // ─────────────────────────────────────────────────────────────────

    let size_constraints = (&sa_w, &sa_h).map(move |w: &GenericSize, h: &GenericSize| {
        let size_constraints = vec![
            width_var | WeightedRelation::EQ(cassowary::strength::REQUIRED) | w.get_length_value(),
            height_var | WeightedRelation::EQ(cassowary::strength::REQUIRED) | h.get_length_value(),
            // • • • • •
            bottom_var
                | WeightedRelation::EQ(cassowary::strength::REQUIRED)
                | (top_var + height_var),
            right_var
                | WeightedRelation::EQ(cassowary::strength::REQUIRED)
                | (left_var + width_var),
            bottom_var | WeightedRelation::GE(cassowary::strength::REQUIRED) | top_var,
            right_var | WeightedRelation::GE(cassowary::strength::REQUIRED) | left_var,
            width_var | WeightedRelation::GE(cassowary::strength::REQUIRED) | 0.0,
            height_var | WeightedRelation::GE(cassowary::strength::REQUIRED) | 0.0,
            top_var | WeightedRelation::GE(cassowary::strength::WEAK) | 0.0,
            left_var | WeightedRelation::GE(cassowary::strength::WEAK) | 0.0,
        ];
        size_constraints
    });

    let current_cassowary_map3 = current_cassowary_map.clone();

    let current_cassowary_inherited_generals_sa =
        current_cassowary_generals_sa.map(move |self_generals| {
            let _span = trace_span!("build inherited cassowary_generals_map").entered();
            trace!("current_cassowary_generals + current_cassowary_map:----");
            trace!("-- current_cassowary_generals:{:#?}", &self_generals);
            trace!("-- current_cassowary_map:{:#?}", &current_cassowary_map3);
            let end =
                self_generals.clone().with_default_not_overwrite() + current_cassowary_map3.clone();
            trace!("-- end final map:{:#?}", &end);

            Rc::new(end)
        });

    //TODO 如果没有parent 那么 parent 就是 screen w h
    let cass_or_calc_size: StateAnchor<Vector2<Precision>> =
        (&sa_w, &sa_h).map(|w: &GenericSize, h: &GenericSize| {
            //TODO check editor display error
            Vector2::<Precision>::new(w.get_length_value(), h.get_length_value())
        });

    //TODO 审视是否要自定义定位
    let calculated_origin = StateAnchor::constant(Translation3::<Precision>::identity());
    let calculated_align = StateAnchor::constant(Translation3::<Precision>::identity());
    let coordinates_trans = StateAnchor::constant(Translation3::<Precision>::identity());
    let cass_trans = StateAnchor::constant(Translation3::<Precision>::identity());
    let calculated_translation =
        (&cass_trans, &coordinates_trans).map(|cass, defined| cass * defined);

    let matrix = cass_trans.map(|x| x.to_homogeneous().into());
    let loc_styles =
        (&cass_or_calc_size, &matrix).map(move |size: &Vector2<Precision>, mat4: &Mat4| {
            let _enter = span!(
                Level::TRACE,
                "-> [root] [ loc_styles ] recalculation..(&calculated_size, &matrix).map ",
            )
            .entered();

            trace!("size: {}  , matrix: {}", size, CssTransform::from(*mat4));

            // TODO use  key 更新 s(),
            s().w(px(size.x)).h(px(size.y)).transform(*mat4)
        });

    let world = calculated_translation.clone();
    let layout_calculated = LayoutCalculated {
        size_constraints,
        cassowary_inherited_generals_sa: current_cassowary_inherited_generals_sa,
        cass_or_calc_size,
        origin: calculated_origin,
        align: calculated_align,
        translation: calculated_translation,
        coordinates_trans,
        cass_trans,
        matrix,
        // • • • • •
        loc_styles,
        world,
    };
    let styles_string = (
        &path_styles.watch(),
        &layout_calculated.loc_styles,
        &other_styles_sv.watch(),
    )
        .map(
            move |path_styles: &PathVarMap<Style>, loc_styles: &Style, other_styles: &Style| {
                let _enter = span!(
                    Level::TRACE,
                    "-> [ROOT styles ] recalculation..(&other_styles_watch, &loc_styles).map ",
                )
                .entered();

                //NOTE fold because edge has no in , path_styles only one values. fold max run only once;
                let ps = path_styles
                    .values()
                    .fold(String::default(), |acc, v| format!("{}{}", acc, v.render()));

                format!("{}{}{}", other_styles.render(), ps, loc_styles.render())
            },
        );
    (None, layout_calculated, styles_string)
}

#[derive(Display, From)]
pub enum EdgeItemNode {
    EdgeData(Box<EdgeData>),
    String(String), //TODO make can write, in DictPathEiNodeSA it clone need RC?
    Empty,
}

impl Eq for EdgeItemNode {}

impl PartialEq for EdgeItemNode {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::EdgeData(l0), Self::EdgeData(r0)) => l0 == r0,
            (Self::String(l0), Self::String(r0)) => l0 == r0,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl std::fmt::Debug for EdgeItemNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EdgeData(arg0) => f.debug_tuple("EdgeData").field(arg0).finish(),
            Self::String(arg0) => f.debug_tuple("String").field(arg0).finish(),
            Self::Empty => write!(f, "Empty"),
        }
    }
}

impl Clone for EdgeItemNode {
    fn clone(&self) -> Self {
        match self {
            Self::EdgeData(arg0) => Self::EdgeData(arg0.clone()),
            Self::String(arg0) => Self::String(arg0.clone()),
            Self::Empty => Self::Empty,
        }
    }
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

    #[must_use]
    pub const fn as_string(&self) -> Option<&String> {
        if let Self::String(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Returns `true` if the edge item node is [`Empty`].
    ///
    /// [`Empty`]: EdgeItemNode::Empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }
}

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

// pub fn size(v: impl Into<GenericSize>) -> GenericSizeAnchor {
//     v.into().into()
// }
// pub fn origin2(x: impl Into<GenericSize>, y: impl Into<GenericSize>) -> GenericLoc {
//     GenericLoc::new(x, y, px(0))
// }
// pub fn align2(x: impl Into<GenericSize>, y: impl Into<GenericSize>) -> GenericLoc {
//     GenericLoc::new(x, y, px(0))
// }
//TODO lifetime
pub fn css<Use: CssValueTrait + Clone + 'static>(v: Use) -> Box<dyn Shaping<EmgEdgeItem>> {
    // pub fn css<Use: CssValueTrait + std::clone::Clone + 'static>(v: Use) -> Box<Css<Use>> {
    Box::new(Css(v))
}

#[cfg(test)]
pub mod tests {
    #![allow(clippy::too_many_lines)]
    use crate::{
        css, debug, debug_span, emg_common, epath, instrument, px, s, styles, topo, trace,
        use_state, Clone, CloneStateAnchor, CloneStateVar, Css, CssWidthTrait, Dict, EPath, Edge,
        EdgeIndex, EdgeItemNode, EmgEdgeItem, GraphEdgesDict, Level, Precision, Translation3,
        Vector2,
    };
    extern crate test;

    use emg::{edge_index, edge_index_no_source, node_index};
    use emg_common::{im::vector, num_traits::ToPrimitive};
    use emg_common::{parent, IdStr};
    use emg_shaping::ShapingUseDyn;
    use emg_state::StateVar;

    use styles::{bg_color, h, hsl, pc, width, CssBackgroundColorTrait, CssHeight, CssWidth};
    use tracing::{info, span, warn};

    use test::{black_box, Bencher};

    use emg_common::num_traits::cast;

    use color_eyre::eyre::Report;
    ///# Errors
    /// run twice
    pub fn tracing_init() -> Result<(), Report> {
        use tracing_subscriber::prelude::*;
        fn theme() -> color_eyre::config::Theme {
            use color_eyre::{config::Theme, owo_colors::style};

            Theme::dark().active_line(style().bright_yellow().bold())
            // ^ use `new` to derive from a blank theme, or `light` to derive from a light theme.
            // Now configure your theme (see the docs for all options):
            // .line_number(style().blue())
            // .help_info_suggestion(style().red())
        }
        // let error_layer =
        // tracing_subscriber::fmt::layer().with_filter(tracing::metadata::LevelFilter::ERROR);

        let tree_layer = tracing_tree::HierarchicalLayer::new(2)
            .with_indent_lines(true)
            .with_indent_amount(4)
            .with_targets(true)
            .with_filter(tracing_subscriber::EnvFilter::new(
                // "emg_layout=debug,emg_layout[build inherited cassowary_generals_map],emg_layout[LayoutOverride]=error",
                // "[GElement-shaping]=debug",
                // "error,[sa gel in map clone]=debug",
                // "emg_state=off,[anchors-dirty]=debug,cassowary=off",
                // ,
                "[StateVarProperty clone]=debug,[StateVarProperty drop]=debug,[manually_drop]=trace,[sv to svp]=debug,[clock.remove_after_fn]=debug",
                // emg_layout::animation::tests=off
                // "error",
            ));

        tracing_subscriber::registry()
            // .with(layout_override_layer)
            // .with(event_matching_layer)
            // .with(touch_layer)
            .with(tree_layer)
            .with(tracing_error::ErrorLayer::default())
            // .with(out_layer)
            .try_init()?;

        // color_eyre::install()
        color_eyre::config::HookBuilder::new()
            .theme(theme())
            .install()
    }

    #[test]
    fn f64_to_f32() {
        // let val = 1452089033.7674935_f64;
        #[allow(clippy::unreadable_literal)]
        let val = 12089033.7674935_f64;
        let x: f32 = cast(val).unwrap();
        println!("64- {val:?}");
        println!("32- {x:?}");

        let x: f32 = val.to_f32().unwrap();
        println!("64- {val:?}");
        println!("32- {x:?}");

        #[allow(clippy::cast_possible_truncation)]
        let x: f32 = (f64::trunc(val * 100.0f64) / 100.0f64) as f32;
        println!("64- {val:?}");
        println!("32- {x:?}");

        // let after = f64::trunc(before  * 100.0) / 100.0; // or f32::trunc
    }

    #[test]
    fn loc() {
        // init();
        let _xx = tracing_init();
        {
            let span = span!(Level::TRACE, "loc-test");
            let _guard = span.enter();

            info!("=========================================================");

            let css_width = width(px(100));
            let css_height = h(px(100));
            let e_dict_sv: StateVar<GraphEdgesDict> = use_state(Dict::new);

            let root_e_source = use_state(|| None);
            let root_e_target = use_state(|| Some(node_index("root")));
            let mut root_e = EmgEdgeItem::default_with_wh_in_topo(
                root_e_source.watch(),
                root_e_target.watch(),
                e_dict_sv.watch(),
                1920,
                1080,
            );
            e_dict_sv.set_with_once(|d| {
                let mut nd = d.clone();
                nd.insert(
                    EdgeIndex::new(None, node_index("root")),
                    Edge::new(root_e_source, root_e_target, root_e.clone()),
                );
                nd
            });

            let e1_source = use_state(|| Some(node_index("root")));
            let e1_target = use_state(|| Some(node_index("1")));
            let e1 = EmgEdgeItem::new_in_topo(
                e1_source.watch(),
                e1_target.watch(),
                e_dict_sv.watch(),
                (px(50), px(50)),
                (pc(0), pc(0), pc(0)),
                (pc(50), pc(50), pc(50)),
            );

            e_dict_sv.set_with_once(|d| {
                let mut nd = d.clone();
                nd.insert(
                    edge_index("root", "1"),
                    Edge::new(e1_source, e1_target, e1.clone()),
                );
                nd
            });

            let e2_source = use_state(|| Some(node_index("1")));
            let e2_target = use_state(|| Some(node_index("2")));
            let mut e2 = EmgEdgeItem::new_in_topo(
                e2_source.watch(),
                e2_target.watch(),
                e_dict_sv.watch(),
                (px(10), px(10)),
                (pc(100), pc(100), pc(100)),
                (pc(100), pc(100), pc(100)),
            );
            e_dict_sv.set_with_once(|d| {
                let mut nd = d.clone();
                nd.insert(
                    edge_index("1", "2"),
                    Edge::new(e2_source, e2_target, e2.clone()),
                );
                nd
            });

            // debug!("shaping_use before {}", &ec);
            let _span = span!(Level::TRACE, "debug print e1");
            _span.in_scope(|| {
                trace!("loc shaping_use before {}", &e1);
            });
            info!("l2 =========================================================");

            root_e.shaping_use_dyn(&vec![css(css_width)]);
            // root_e.shaping_use(&css(css_width.clone()));
            root_e.shaping_use_dyn(&Css(css_height));
            assert_eq!(
                e1.edge_data(&EPath(vector![
                    edge_index_no_source("root"),
                    edge_index("root", "1")
                ]))
                .unwrap()
                .calculated
                .coordinates_trans
                .get(),
                Translation3::<Precision>::new(50., 50., 0.)
            );
            info!("=========================================================");

            e2.shaping_use_dyn(&Css(CssWidth::from(px(20))));
            e2.shaping_use_dyn(&Css(CssHeight::from(px(20))));
            e2.shaping_use_dyn(&Css(bg_color(hsl(40, 70, 30))));

            trace!("shaping_use after {:#?}", &e2);
            info!("l3 =========================================================");
            assert_eq!(
                e2.edge_data(&EPath(vector![
                    edge_index_no_source("root"),
                    edge_index("root", "1"),
                    edge_index("1", "2")
                ]))
                .unwrap()
                .calculated
                .coordinates_trans
                .get(),
                Translation3::<Precision>::new(30., 30., 0.)
            );
            trace!(
                "{}",
                e2.edge_data(&EPath(vector![
                    edge_index_no_source("root"),
                    edge_index("root", "1"),
                    edge_index("1", "2")
                ]))
                .unwrap()
                .styles_string
                .get(),
            );
            #[cfg(feature = "insta")]
            insta::assert_debug_snapshot!("loc-root_e", &root_e);
            #[cfg(feature = "insta")]
            insta::assert_debug_snapshot!("loc-e1", &e1);
            #[cfg(feature = "insta")]
            insta::assert_debug_snapshot!("loc-e2", &e2);

            info!("..=========================================================");
        }
    }
    #[bench]
    #[topo::nested]
    fn it_works_bench(b: &mut Bencher) {
        b.iter(|| {
            it_works_for_bench();
            black_box(());
        });
    }

    fn it_works_for_bench() {
        info!("--------------------=====================================");
        // vec![ CssWidth::from(px(100))].up
        info!("=========================================================");

        // let cc = Affine3<f64>::identity();
        let _ff = s().width(pc(11)).bg_color(hsl(40, 70, 30));
        let css_width = width(px(100));
        let css_height = h(px(100));

        let e_dict_sv: StateVar<GraphEdgesDict> = use_state(Dict::new);

        let root_e_source = use_state(|| None);
        let root_e_target = use_state(|| Some(node_index("root")));
        let mut root_e = EmgEdgeItem::default_with_wh_in_topo(
            root_e_source.watch(),
            root_e_target.watch(),
            e_dict_sv.watch(),
            1920,
            1080,
        );
        e_dict_sv.set_with_once(|d| {
            let mut nd = d.clone();
            nd.insert(
                EdgeIndex::new(None, Some(node_index("root"))),
                Edge::new(root_e_source, root_e_target, root_e.clone()),
            );
            nd
        });

        let e1_source = use_state(|| Some(node_index("root")));
        let e1_target = use_state(|| Some(node_index("1")));
        let mut e1 = EmgEdgeItem::new_in_topo(
            e1_source.watch(),
            e1_target.watch(),
            e_dict_sv.watch(),
            (px(10), px(10)),
            (pc(100), pc(100), pc(100)),
            (pc(50), pc(20), pc(20)),
        );
        e_dict_sv.set_with_once(|d| {
            let mut nd = d.clone();
            nd.insert(
                edge_index("root", "1"),
                Edge::new(e1_source, e1_target, e1.clone()),
            );
            nd
        });

        let e2_source = use_state(|| Some(node_index("1")));
        let e2_target = use_state(|| Some(node_index("2")));
        let mut e2 = EmgEdgeItem::new_in_topo(
            e2_source.watch(),
            e2_target.watch(),
            e_dict_sv.watch(),
            (px(10), px(10)),
            (pc(100), pc(100), pc(100)),
            (pc(50), pc(20), pc(20)),
        );
        e_dict_sv.set_with_once(|d| {
            let mut nd = d.clone();
            nd.insert(
                edge_index("1", "2"),
                Edge::new(e2_source, e2_target, e2.clone()),
            );
            nd
        });

        // debug!("shaping_use before {}", &ec);
        let _span = span!(Level::TRACE, "debug print e1");
        _span.in_scope(|| {
            trace!("shaping_use before {}", &e1);
        });
        info!("l1 =========================================================");

        assert_eq!(
            e1.edge_nodes
                .get()
                .get(&EPath(vector![
                    edge_index_no_source("root"),
                    edge_index("root", "1")
                ]))
                .and_then(EdgeItemNode::as_edge_data)
                .unwrap()
                .calculated
                .coordinates_trans
                .get(),
            Translation3::<Precision>::new(950.0, 206.0, 0.)
        );

        let xx = vec![css_width];
        // let xx = vec![css(css_width)];

        root_e.shaping_use_dyn(&xx);
        root_e.shaping_use_dyn(&xx);
        root_e.shaping_use_dyn(&xx);
        root_e.shaping_use_dyn(&xx);
        root_e.shaping_use_dyn(&xx);
        root_e.shaping_use_dyn(&xx);
        root_e.shaping_use_dyn(&xx);
        root_e.shaping_use_dyn(&xx);
        root_e.shaping_use_dyn(&xx);
        root_e.shaping_use_dyn(&xx);
        root_e.shaping_use_dyn(&xx);
        root_e.shaping_use_dyn(&xx);
        root_e.shaping_use_dyn(&xx);
        root_e.shaping_use_dyn(&xx);
        root_e.shaping_use_dyn(&xx);
        root_e.shaping_use_dyn(&xx);
        root_e.shaping_use_dyn(&xx);
        root_e.shaping_use_dyn(&xx);
        root_e.shaping_use_dyn(&xx);
        root_e.shaping_use_dyn(&xx);
        root_e.shaping_use_dyn(&xx);
        root_e.shaping_use_dyn(&xx);
        root_e.shaping_use_dyn(&xx);
        root_e.shaping_use_dyn(&xx);
        root_e.shaping_use_dyn(&xx);
        root_e.shaping_use_dyn(&xx);
        root_e.shaping_use_dyn(&xx);
        root_e.shaping_use_dyn(&xx);
        root_e.shaping_use_dyn(&xx);
        root_e.shaping_use_dyn(&xx);
        root_e.shaping_use_dyn(&xx);
        root_e.shaping_use_dyn(&xx);
        root_e.shaping_use_dyn(&xx);
        root_e.shaping_use_dyn(&xx);
        root_e.shaping_use_dyn(&xx);

        // trace!("shaping_use after css_width {}", &root_e);
        trace!("shaping_use after css_width {}", &e1);
        info!("=========================================================");

        // root_e.shaping_use(&Css(css_height.clone()));
        let tempcss = use_state(|| css_height);
        root_e.shaping_use_dyn(&tempcss);
        assert_eq!(
            e1.edge_nodes
                .get()
                .get(&EPath(vector![
                    edge_index_no_source("root"),
                    edge_index("root", "1")
                ]))
                .and_then(EdgeItemNode::as_edge_data)
                .unwrap()
                .calculated
                .coordinates_trans
                .get(),
            Translation3::<Precision>::new(40., 10., 0.)
        );
        info!("=========================================================");
        tempcss.set(h(px(1111)));
        assert_ne!(
            e1.edge_nodes
                .get()
                .get(&EPath(vector![
                    edge_index_no_source("root"),
                    edge_index("root", "1")
                ]))
                .and_then(EdgeItemNode::as_edge_data)
                .unwrap()
                .calculated
                .coordinates_trans
                .get(),
            Translation3::<Precision>::new(40., 10., 0.)
        );
        tempcss.set(h(px(100)));
        assert_eq!(
            e1.edge_nodes
                .get()
                .get(&EPath(vector![
                    edge_index_no_source("root"),
                    edge_index("root", "1")
                ]))
                .and_then(EdgeItemNode::as_edge_data)
                .unwrap()
                .calculated
                .coordinates_trans
                .get(),
            Translation3::<Precision>::new(40., 10., 0.)
        );

        info!("=========================================================");

        e1.shaping_use_dyn(&Css(CssWidth::from(px(12))));
        e1.shaping_use_dyn(&Css(CssWidth::from(px(12))));
        e1.shaping_use_dyn(&Css(CssWidth::from(px(12))));
        e1.shaping_use_dyn(&Css(CssWidth::from(px(12))));
        e1.shaping_use_dyn(&Css(CssWidth::from(px(12))));
        e1.shaping_use_dyn(&Css(CssWidth::from(px(12))));
        e1.shaping_use_dyn(&Css(CssWidth::from(px(12))));
        e1.shaping_use_dyn(&Css(CssWidth::from(px(12))));
        e1.shaping_use_dyn(&Css(CssWidth::from(px(12))));
        e1.shaping_use_dyn(&Css(CssWidth::from(px(12))));
        e1.shaping_use_dyn(&Css(CssWidth::from(px(12))));
        e1.shaping_use_dyn(&Css(CssWidth::from(px(12))));
        e1.shaping_use_dyn(&Css(CssWidth::from(px(12))));
        e1.shaping_use_dyn(&Css(CssWidth::from(px(12))));
        e1.shaping_use_dyn(&Css(CssWidth::from(px(12))));
        e1.shaping_use_dyn(&Css(CssWidth::from(px(12))));
        e1.shaping_use_dyn(&Css(CssWidth::from(px(12))));
        e1.shaping_use_dyn(&Css(CssWidth::from(px(12))));
        e1.shaping_use_dyn(&Css(CssWidth::from(px(12))));
        e1.shaping_use_dyn(&Css(CssWidth::from(px(12))));
        e1.shaping_use_dyn(&Css(CssWidth::from(px(12))));
        e1.shaping_use_dyn(&Css(CssWidth::from(px(12))));
        e1.shaping_use_dyn(&Css(CssWidth::from(px(12))));
        e1.shaping_use_dyn(&Css(CssWidth::from(px(12))));
        e1.shaping_use_dyn(&Css(CssWidth::from(px(12))));
        e1.shaping_use_dyn(&Css(CssWidth::from(px(12))));
        e1.shaping_use_dyn(&Css(CssWidth::from(px(12))));
        e1.shaping_use_dyn(&Css(CssWidth::from(px(12))));
        e1.shaping_use_dyn(&Css(CssWidth::from(px(12))));
        e1.shaping_use_dyn(&Css(CssWidth::from(px(12))));
        assert_eq!(
            e1.edge_nodes
                .get()
                .get(&EPath(vector![
                    edge_index_no_source("root"),
                    edge_index("root", "1")
                ]))
                .and_then(EdgeItemNode::as_edge_data)
                .unwrap()
                .calculated
                .cass_or_calc_size
                .get(),
            Vector2::<Precision>::new(12., 10.)
        );
        info!("=========================================================");
        assert_eq!(
                e1.edge_nodes.get().get(&EPath(vector![edge_index_no_source("root"), edge_index("root", "1")]))
                    .and_then(EdgeItemNode::as_edge_data)
                .unwrap()
                .styles_string
                .get() ,
            "transform: matrix3d(1,0,0,0,0,1,0,0,0,0,1,0,38,10,0,1);\n width: 12px;\nheight: 10px;\n"
            );
        trace!("shaping_use after {}", &e1);
        info!("=========================================================");

        trace!("shaping_use after {}", &e2);
        info!("l1351 =========================================================");
        e2.shaping_use_dyn(&Css(CssHeight::from(px(50))));
        assert_eq!(
            e2.edge_nodes
                .get()
                .get(&EPath(vector![
                    edge_index_no_source("root"),
                    edge_index("root", "1"),
                    edge_index("1", "2")
                ]))
                .and_then(EdgeItemNode::as_edge_data)
                .unwrap()
                .calculated
                .coordinates_trans
                .get(),
            Translation3::<Precision>::new(-4.0, -48.0, 0.0)
        );
        e2.set_size(px(100), px(100));
        assert_eq!(
            e2.edge_nodes
                .get()
                .get(&EPath(vector![
                    edge_index_no_source("root"),
                    edge_index("root", "1"),
                    edge_index("1", "2")
                ]))
                .and_then(EdgeItemNode::as_edge_data)
                .unwrap()
                .calculated
                .cass_or_calc_size
                .get(),
            Vector2::<Precision>::new(100.0, 100.0)
        );
        let _span1 = span!(Level::TRACE, "debug print 1");
        _span1.in_scope(|| {
            trace!("shaping_use after {}", &e2);
        });

        let _span2 = span!(Level::TRACE, "debug print 2");
        _span2.in_scope(|| {
            trace!("shaping_use after2 {}", &e2);
        });
        info!("=========================================================");
        e2.shaping_use_dyn(&Css(CssHeight::from(px(150))));

        trace!("shaping_use after {:#?}", &e2);
        info!("..=========================================================");
        trace!(
            "{}",
            e2.edge_nodes
                .get()
                .get(&EPath(vector![
                    edge_index_no_source("root"),
                    edge_index("root", "1"),
                    edge_index("1", "2")
                ]))
                .and_then(EdgeItemNode::as_edge_data)
                .unwrap()
                .styles_string
                .get(),
        );
        info!("..=========================================================");
    }
    #[test]
    fn test_edge() {
        let f = width(parent!(CssHeight) + pc(100));
        println!("{f}");
    }

    #[test]
    #[instrument]
    fn it_works() {
        let _xx = tracing_init();

        let _span = debug_span!("start").entered();

        info!("--------------------=====================================");
        // vec![ CssWidth::from(px(100))].up
        info!("=========================================================");

        // let cc = Affine3<f64>::identity();
        let _build_test = s().width(pc(11)).bg_color(hsl(40, 70, 30));
        let css_width = width(px(100));
        let css_height = h(px(100));

        let e_dict_sv: StateVar<GraphEdgesDict> = use_state(Dict::new);

        let root_e_source = use_state(|| None);
        let root_e_target = use_state(|| Some(node_index("root")));
        let mut root_e = EmgEdgeItem::default_with_wh_in_topo(
            root_e_source.watch(),
            root_e_target.watch(),
            e_dict_sv.watch(),
            1920,
            1080,
        );
        e_dict_sv.set_with_once(|d| {
            let mut nd = d.clone();
            nd.insert(
                EdgeIndex::new(None, Some(node_index("root"))),
                Edge::new(root_e_source, root_e_target, root_e.clone()),
            );
            nd
        });

        let e1_source = use_state(|| Some(node_index("root")));
        let e1_target = use_state(|| Some(node_index("1")));
        let mut e1 = EmgEdgeItem::new_in_topo(
            e1_source.watch(),
            e1_target.watch(),
            e_dict_sv.watch(),
            (
                (parent!(CssHeight) + pc(100)),
                (parent!(CssHeight) + pc(100)),
            ),
            (pc(100), pc(100), pc(100)),
            (
                (parent!(CssWidth) * 0.5),
                (parent!(CssHeight) * 0.2),
                pc(20),
            ),
        );
        e_dict_sv.set_with_once(|d| {
            let mut nd = d.clone();
            nd.insert(
                edge_index("root", "1"),
                Edge::new(e1_source, e1_target, e1.clone()),
            );
            nd
        });

        let e2_source = use_state(|| Some(node_index("1")));
        let e2_target = use_state(|| Some(node_index("2")));
        let mut e2 = EmgEdgeItem::new_in_topo(
            e2_source.watch(),
            e2_target.watch(),
            e_dict_sv.watch(),
            (px(10), px(10)),
            (pc(100), pc(100), pc(100)),
            (pc(50), pc(20), pc(20)),
        );
        e_dict_sv.set_with_once(|d| {
            let mut nd = d.clone();
            nd.insert(
                edge_index("1", "2"),
                Edge::new(e2_source, e2_target, e2.clone()),
            );
            nd
        });

        // debug!("shaping_use before {}", &ec);
        let _span = span!(Level::TRACE, "debug print e1");
        _span.in_scope(|| {
            trace!("shaping_use before {}", &e1);
        });
        info!("l1 =========================================================");
        debug!("calculated 1 =========================================================");
        // debug!(
        //     "==== {}",
        //     e1.edge_nodes
        //         .get()
        //         .get(&EPath(vector![
        //             edge_index_no_source("root"),
        //             edge_index("root", "1")
        //         ]))
        //         .and_then(EdgeItemNode::as_edge_data)
        //         .unwrap()
        //         .calculated
        // );
        assert_eq!(
            e1.edge_nodes
                .get()
                .get(&EPath(vector![
                    edge_index_no_source("root"),
                    edge_index("root", "1")
                ]))
                .and_then(EdgeItemNode::as_edge_data)
                .unwrap()
                .calculated
                .coordinates_trans
                .get(),
            Translation3::<Precision>::new(-2040.0, -1944.0, 0.)
        );
        debug!("calculated 2 =========================================================");
        debug!(
            "{}",
            e1.edge_nodes
                .get()
                .get(&EPath(vector![
                    edge_index_no_source("root"),
                    edge_index("root", "1")
                ]))
                .and_then(EdgeItemNode::as_edge_data)
                .unwrap()
                .calculated
        );

        let xx = vec![css_width];
        // let xx = vec![css(css_width)];

        root_e.shaping_use_dyn(&xx);

        warn!("calculated 3 =========================================================");
        warn!(
            "======= {}",
            e1.edge_nodes
                .get()
                .get(&EPath(vector![
                    edge_index_no_source("root"),
                    edge_index("root", "1")
                ]))
                .and_then(EdgeItemNode::as_edge_data)
                .unwrap()
                .calculated
        );

        root_e.shaping_use_dyn(&xx);
        root_e.shaping_use_dyn(&xx);
        root_e.shaping_use_dyn(&xx);
        root_e.shaping_use_dyn(&xx);
        root_e.shaping_use_dyn(&xx);
        root_e.shaping_use_dyn(&xx);
        root_e.shaping_use_dyn(&xx);
        root_e.shaping_use_dyn(&xx);
        root_e.shaping_use_dyn(&xx);
        root_e.shaping_use_dyn(&xx);
        root_e.shaping_use_dyn(&xx);
        root_e.shaping_use_dyn(&xx);
        root_e.shaping_use_dyn(&xx);
        root_e.shaping_use_dyn(&xx);
        root_e.shaping_use_dyn(&xx);
        root_e.shaping_use_dyn(&xx);
        root_e.shaping_use_dyn(&xx);
        root_e.shaping_use_dyn(&xx);
        root_e.shaping_use_dyn(&xx);
        root_e.shaping_use_dyn(&xx);
        root_e.shaping_use_dyn(&xx);
        root_e.shaping_use_dyn(&xx);
        root_e.shaping_use_dyn(&xx);
        root_e.shaping_use_dyn(&xx);
        root_e.shaping_use_dyn(&xx);
        root_e.shaping_use_dyn(&xx);
        root_e.shaping_use_dyn(&xx);
        root_e.shaping_use_dyn(&xx);
        root_e.shaping_use_dyn(&xx);
        root_e.shaping_use_dyn(&xx);
        root_e.shaping_use_dyn(&xx);
        root_e.shaping_use_dyn(&xx);
        root_e.shaping_use_dyn(&xx);
        root_e.shaping_use_dyn(&xx);

        // trace!("shaping_use after css_width {}", &root_e);
        trace!("shaping_use after css_width {}", &e1);
        info!("=========================================================");

        // root_e.shaping_use(&Css(css_height.clone()));
        let tempcss = use_state(|| css_height);
        root_e.shaping_use_dyn(&tempcss);

        warn!(
            "calculated 4 root h w 100 ========================================================="
        );
        warn!(
            "{}",
            e1.edge_nodes
                .get()
                .get(&EPath(vector![
                    edge_index_no_source("root"),
                    edge_index("root", "1")
                ]))
                .and_then(EdgeItemNode::as_edge_data)
                .unwrap()
                .calculated
        );

        assert_eq!(
            e1.edge_nodes
                .get()
                .get(&EPath(vector![
                    edge_index_no_source("root"),
                    edge_index("root", "1")
                ]))
                .and_then(EdgeItemNode::as_edge_data)
                .unwrap()
                .calculated
                .coordinates_trans
                .get(),
            Translation3::<Precision>::new(-150.0, -180.0, 0.)
        );
        info!("=========================================================");
        tempcss.set(h(px(1111)));
        let _ff = e1
            .edge_nodes
            .get()
            .get(&EPath(vector![
                edge_index_no_source("root"),
                edge_index("root", "1")
            ]))
            .and_then(EdgeItemNode::as_edge_data)
            .unwrap()
            .styles_string
            .get();
        assert_ne!(
            e1.edge_nodes
                .get()
                .get(&EPath(vector![
                    edge_index_no_source("root"),
                    edge_index("root", "1")
                ]))
                .and_then(EdgeItemNode::as_edge_data)
                .unwrap()
                .calculated
                .coordinates_trans
                .get(),
            Translation3::<Precision>::new(40., 10., 0.)
        );
        tempcss.set(h(px(100)));
        assert_eq!(
            e1.edge_nodes
                .get()
                .get(&EPath(vector![
                    edge_index_no_source("root"),
                    edge_index("root", "1")
                ]))
                .and_then(EdgeItemNode::as_edge_data)
                .unwrap()
                .calculated
                .coordinates_trans
                .get(),
            Translation3::<Precision>::new(-150.0, -180.0, 0.)
        );

        info!("=========================================================");

        e1.shaping_use_dyn(&Css(CssWidth::from(px(12))));
        e1.shaping_use_dyn(&Css(CssWidth::from(px(12))));
        e1.shaping_use_dyn(&Css(CssWidth::from(px(12))));
        e1.shaping_use_dyn(&Css(CssWidth::from(px(12))));
        e1.shaping_use_dyn(&Css(CssWidth::from(px(12))));
        e1.shaping_use_dyn(&Css(CssWidth::from(px(12))));
        e1.shaping_use_dyn(&Css(CssWidth::from(px(12))));
        e1.shaping_use_dyn(&Css(CssWidth::from(px(12))));
        e1.shaping_use_dyn(&Css(CssWidth::from(px(12))));
        e1.shaping_use_dyn(&Css(CssWidth::from(px(12))));
        e1.shaping_use_dyn(&Css(CssWidth::from(px(12))));
        e1.shaping_use_dyn(&Css(CssWidth::from(px(12))));
        e1.shaping_use_dyn(&Css(CssWidth::from(px(12))));
        e1.shaping_use_dyn(&Css(CssWidth::from(px(12))));
        e1.shaping_use_dyn(&Css(CssWidth::from(px(12))));
        e1.shaping_use_dyn(&Css(CssWidth::from(px(12))));
        e1.shaping_use_dyn(&Css(CssWidth::from(px(12))));
        e1.shaping_use_dyn(&Css(CssWidth::from(px(12))));
        e1.shaping_use_dyn(&Css(CssWidth::from(px(12))));
        e1.shaping_use_dyn(&Css(CssWidth::from(px(12))));
        e1.shaping_use_dyn(&Css(CssWidth::from(px(12))));
        e1.shaping_use_dyn(&Css(CssWidth::from(px(12))));
        e1.shaping_use_dyn(&Css(CssWidth::from(px(12))));
        e1.shaping_use_dyn(&Css(CssWidth::from(px(12))));
        e1.shaping_use_dyn(&Css(CssWidth::from(px(12))));
        e1.shaping_use_dyn(&Css(CssWidth::from(px(12))));
        e1.shaping_use_dyn(&Css(CssWidth::from(px(12))));
        e1.shaping_use_dyn(&Css(CssWidth::from(px(12))));
        e1.shaping_use_dyn(&Css(CssWidth::from(px(12))));
        e1.shaping_use_dyn(&Css(CssWidth::from(px(12))));

        assert_eq!(
            e1.edge_nodes
                .get()
                .get(&EPath(vector![
                    edge_index_no_source("root"),
                    edge_index("root", "1")
                ]))
                .and_then(EdgeItemNode::as_edge_data)
                .unwrap()
                .calculated
                .cass_or_calc_size
                .get(),
            Vector2::<Precision>::new(12., 200.)
        );
        info!("=========================================================");
        assert_eq!(
                e1.edge_nodes.get().get(&EPath(vector![edge_index_no_source("root"), edge_index("root", "1")]))
                    .and_then(EdgeItemNode::as_edge_data)
                .unwrap()
                .styles_string
                .get() ,
            "transform: matrix3d(1,0,0,0,0,1,0,0,0,0,1,0,38,-180,0,1);\n width: 12px;\nheight: 200px;\n"
            );
        trace!("shaping_use after {}", &e1);
        info!("=========================================================");

        trace!("shaping_use after {}", &e2);
        info!("l1351 =========================================================");
        e2.shaping_use_dyn(&Css(CssHeight::from(px(50))));
        assert_eq!(
            e2.edge_nodes
                .get()
                .get(&EPath(vector![
                    edge_index_no_source("root"),
                    edge_index("root", "1"),
                    edge_index("1", "2")
                ]))
                .and_then(EdgeItemNode::as_edge_data)
                .unwrap()
                .calculated
                .coordinates_trans
                .get(),
            Translation3::<Precision>::new(-4.0, -10.0, 0.0)
        );
        e2.set_size(px(100), px(100));
        assert_eq!(
            e2.edge_nodes
                .get()
                .get(&EPath(vector![
                    edge_index_no_source("root"),
                    edge_index("root", "1"),
                    edge_index("1", "2")
                ]))
                .and_then(EdgeItemNode::as_edge_data)
                .unwrap()
                .calculated
                .cass_or_calc_size
                .get(),
            Vector2::<Precision>::new(100.0, 100.0)
        );
        let _span1 = span!(Level::TRACE, "debug print 1");
        _span1.in_scope(|| {
            trace!("shaping_use after {}", &e2);
        });

        let _span2 = span!(Level::TRACE, "debug print 2");
        _span2.in_scope(|| {
            trace!("shaping_use after2 {}", &e2);
        });
        info!("=========================================================");
        e2.shaping_use_dyn(&Css(CssHeight::from(px(150))));

        trace!("shaping_use after {:#?}", &e2);
        info!("..=========================================================");
        trace!(
            "{}",
            e2.edge_nodes
                .get()
                .get(&EPath(vector![
                    edge_index_no_source("root"),
                    edge_index("root", "1"),
                    edge_index("1", "2")
                ]))
                .and_then(EdgeItemNode::as_edge_data)
                .unwrap()
                .styles_string
                .get(),
        );
        #[cfg(feature = "insta")]
        insta::assert_debug_snapshot!("it_works-root_e", &root_e);
        #[cfg(feature = "insta")]
        insta::assert_debug_snapshot!("it_works-e1", &e1);
        #[cfg(feature = "insta")]
        insta::assert_debug_snapshot!("it_works-e2", &e2);
        info!("..=========================================================");
    }

    #[test]
    fn change_parent() {
        let _xx = tracing_init();
        {
            let _g = span!(Level::TRACE, "change_parent").entered();

            let e_dict_sv: StateVar<GraphEdgesDict> = use_state(Dict::new);

            let root_e_source = use_state(|| None);
            let root_e_target = use_state(|| Some(node_index("root")));
            let root_e = EmgEdgeItem::default_with_wh_in_topo(
                root_e_source.watch(),
                root_e_target.watch(),
                e_dict_sv.watch(),
                100,
                100,
            );
            e_dict_sv.set_with_once(|d| {
                let mut nd = d.clone();
                nd.insert(
                    EdgeIndex::new(None, Some(node_index("root"))),
                    Edge::new(root_e_source, root_e_target, root_e.clone()),
                );
                nd
            });

            let s_root_e2_source = use_state(|| None);
            let root_e2_target = use_state(|| Some(node_index("root2")));
            let root_e2 = EmgEdgeItem::default_with_wh_in_topo(
                s_root_e2_source.watch(),
                root_e2_target.watch(),
                e_dict_sv.watch(),
                200,
                200,
            );
            e_dict_sv.set_with_once(|d| {
                let mut nd = d.clone();
                nd.insert(
                    EdgeIndex::new(None, Some(node_index("root2"))),
                    Edge::new(s_root_e2_source, root_e2_target, root_e2.clone()),
                );
                nd
            });
            // ---------------------------------------

            let e1_source = use_state(|| Some(node_index("root")));
            let e1_target = use_state(|| Some(node_index("1")));
            let e1 = EmgEdgeItem::new_in_topo(
                e1_source.watch(),
                e1_target.watch(),
                e_dict_sv.watch(),
                (px(10), pc(10)),
                (pc(0), pc(0), pc(0)),
                (pc(50), pc(50), pc(50)),
            );
            e_dict_sv.set_with_once(|d| {
                let mut nd = d.clone();
                nd.insert(
                    edge_index("root", "1"),
                    Edge::new(e1_source, e1_target, e1.clone()),
                );
                nd
            });

            //-------------------------------------

            let e2_source = use_state(|| Some(node_index("1")));
            let e2_target = use_state(|| Some(node_index("2")));
            let e2 = EmgEdgeItem::new_in_topo(
                e2_source.watch(),
                e2_target.watch(),
                e_dict_sv.watch(),
                (px(10), px(10)),
                (pc(0), pc(0), pc(0)),
                (pc(100), pc(000), pc(000)),
            );
            e_dict_sv.set_with_once(|d| {
                let mut nd = d.clone();
                nd.insert(
                    edge_index("1", "2"),
                    Edge::new(e2_source, e2_target, e2.clone()),
                );
                nd
            });

            trace!("---e1 {}", &e1);

            assert_eq!(
                e1.edge_data(&epath!("root"=>"1"))
                    .unwrap()
                    .calculated
                    .coordinates_trans
                    .get(),
                Translation3::<Precision>::new(50.0, 50.0, 0.)
            );

            e1_source.set(Some(node_index("root2")));
            trace!("---new root2:e1 {}", &e1);

            assert_eq!(
                e1.edge_data(&epath!("root2"=>"1"))
                    .unwrap()
                    .calculated
                    .coordinates_trans
                    .get(),
                Translation3::<Precision>::new(100., 100., 0.)
            );
            info!("..=========================================================");
            trace!("new root_e2:e1 {}", &e1);
            info!("--------------------------------------------------");
            trace!("new root_e2:e2 {}", &e2);
            info!("..=========================================================");
            //local
            assert_eq!(e2.id.get(), edge_index("1", "2"));
            assert_eq!(
                e2.edge_data(&epath!("root2"=>"1"=>"2"))
                    .unwrap()
                    .calculated
                    .coordinates_trans
                    .get(),
                Translation3::<Precision>::new(10.0, 00.0, 0.)
            );
            info!("================= e_dict_sv: {:#?}", &e_dict_sv);

            // e2.id
            //     .set_with(|id| id.clone().use_incoming(node_index("root")));
            // parent_for_e2_sa.set(root_e.clone());
            // local use root
            e2_source.set(Some(node_index("root")));
            assert_eq!(e2.id.get(), edge_index("root", "2"));
            assert_eq!(
                e2.edge_data(&epath!("root"=>"2"))
                    .unwrap()
                    .calculated
                    .coordinates_trans
                    .get(),
                Translation3::<Precision>::new(100.0, 0.0, 0.)
            );
            trace!("re new root:e2 {}", &e2);
            info!("l1525 =========================================================");
        }
    }
    // #[test]
    // #[should_panic]
    // fn change_child_p_to_none() {
    //     init();
    //     let e = EmgEdgeItem::new_root(100, 100);
    //     let e2 = EmgEdgeItem::new_root(200, 200);
    //     let ec = EmgEdgeItem::new_child(
    //         "e1",
    //         use_state(||Some(e.clone())),
    //         size(px(10), px(10)),
    //         origin2(pc(0), pc(0)),
    //         align2(pc(50), pc(50)),
    //     );
    //     let ec2 = EmgEdgeItem::new_child(
    //         "e2",
    //         use_state(||Some(ec.clone())),
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
    //         Translation3<f64>::new(50.0, 50.0, 0.)
    //     );

    //     ec.as_edge_data_with_parent().unwrap().parent.set(None);
    //     assert_eq!(
    //         ec.as_current_edge_data()
    //             .unwrap()
    //             .calculated
    //             .coordinates_trans
    //             .get(),
    //         Translation3<f64>::new(100.0, 100.0, 0.)
    //     );
    //     // ─────────────────────────────────────────────────────────────────
    //     //local
    //     assert_eq!(
    //         ec2.as_current_edge_data()
    //             .unwrap()
    //             .calculated
    //             .coordinates_trans
    //             .get(),
    //         Translation3<f64>::new(10.0, 00.0, 0.)
    //     );

    //     ec2.as_edge_data_with_parent().unwrap().parent.set(None);

    //     // local use root
    //     assert_eq!(
    //         ec2.as_current_edge_data()
    //             .unwrap()
    //             .calculated
    //             .coordinates_trans
    //             .get(),
    //         Translation3<f64>::new(100.0, 0.0, 0.)
    //     );
    // }
}
