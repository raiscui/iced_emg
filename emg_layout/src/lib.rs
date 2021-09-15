#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::non_ascii_literal)]
#![allow(clippy::used_underscore_binding)]//for display attr

// ────────────────────────────────────────────────────────────────────────────────
#![feature(iter_intersperse)]
#![feature(min_specialization)]
#![feature(auto_traits)]
#![feature(negative_impls)]
// ────────────────────────────────────────────────────────────────────────────────
#![feature(test)]
// ────────────────────────────────────────────────────────────────────────────────




// // 固定 type  T:xxx
// #![feature(trivial_bounds)]
// #![feature(negative_impls)]
// #![feature(auto_traits)]
use std::{cell::RefCell, clone::Clone, cmp::{Eq, Ord}, collections::HashMap, hash::{BuildHasherDefault, Hash}, rc::Rc};

use add_values::{AlignX, OriginX, OriginY};
use calc::layout_calculating;
use derive_more::Display;
use derive_more::From;
use derive_more::Into;
// use derive_more::TryInto;
use emg_core::{GenericSize, TypeCheck};
use emg::{Edge, EdgeIndex, NodeIndex, };
use emg_refresh::RefreshFor;
use emg_state::{Anchor, CloneStateAnchor, CloneStateVar, Dict, GStateStore, StateAnchor, StateMultiAnchor, StateVar, state_store, topo, use_state, use_state_impl::Engine};
use emg_core::Vector;
use emg_core::vector;
use na::{Affine3, Isometry3, Matrix4, Point3, Rotation3, Similarity3, Translation3, Vector2, Vector3};
use nalgebra as na;
pub use seed_styles as styles;
use styles::{CssHeight, CssTransform, CssValueTrait, CssWidth, Style, UpdateStyle, px, s};
// use styles::Percent;
// use styles::ExactLength;
// use styles::CssWidth;
// use styles::CssHeight;
use styles::{CssHeightTrait, CssTransformTrait, CssWidthTrait};
//
// ────────────────────────────────────────────────────────────────────────────────

use indented::indented;
use tracing::{span, trace_span,error,instrument, trace, Level};
// ────────────────────────────────────────────────────────────────────────────────

mod calc;
mod impl_refresh;
pub mod animation;
pub mod add_values;
pub use animation::AnimationE;

// ────────────────────────────────────────────────────────────────────────────────
thread_local! {
    static G_WIDTH: StateVar<f64> = use_state(0.);
}
#[must_use] pub fn global_width() -> StateVar<f64> {
    G_WIDTH.with(|sv| *sv)
}
thread_local! {
    static G_HEIGHT: StateVar<f64> = use_state(0.);
}
#[must_use] pub fn global_height() -> StateVar<f64> {
    G_HEIGHT.with(|sv| *sv)
}

// ────────────────────────────────────────────────────────────────────────────────

#[derive(Display, Debug, PartialEq, PartialOrd, Copy, Clone, From, Into)]
struct Mat4(Matrix4<f64>);

// type Mat4 = Matrix4<f64>;

// ────────────────────────────────────────────────────────────────────────────────

#[derive(Debug,Clone,PartialEq,Eq,Display)]
pub struct GenericSizeAnchor(StateAnchor<GenericSize>);

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
        Self(self.0*rhs)
    }
}
impl ::core::ops::Add for GenericSizeAnchor {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
       Self( self.0+rhs.0)
    }
}
// ────────────────────────────────────────────────────────────────────────────────
pub auto trait NotStateAnchor {}
impl<T> !NotStateAnchor for StateAnchor<T> {}
pub auto trait NotStateVar {}
impl<T> !NotStateVar for StateVar<T> {}

// impl<T> NotStateAnchor for T{}

impl<T> From<T> for GenericSizeAnchor
where T : NotStateAnchor+NotStateVar+ Into<GenericSize> + Clone+'static{
     fn from(v: T) -> Self {
       Self(StateAnchor::constant( v.into()))
    }
}

impl<T> From<StateAnchor<T>> for GenericSizeAnchor
//TODO sure not use "NotStateAnchor+NotStateVar+ " ?
where T :  Into<GenericSize> + Clone+'static{
    fn from(v: StateAnchor<T>) -> Self {
        
        Self(v.map(|x|x.clone().into()))
    }
}
impl<T> From<StateVar<T>> for GenericSizeAnchor
//TODO sure not use "NotStateAnchor+NotStateVar+ " ?
where T :  Into<GenericSize> + Clone+'static{
    fn from(v: StateVar<T>) -> Self {
        
        // Self(v.watch().map(|x|x.clone().into()))
        Self(v.get_var_with(|v|v.watch().map(|x|x.clone().into()).into()))
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

#[derive(Debug, Clone)]
struct EdgeDataOutput {
    loc_styles: StateAnchor<Style>,
    other_styles: StateVar<Style>,
    styles: StateAnchor<Style>,
    style_string: StateAnchor<String>,
}


// #[derive(Display, Debug, Clone, PartialEq, PartialOrd, Eq)]
// // #[derive(Debug, Clone, Into)]
// // #[into(owned, ref, ref_mut)]
// #[display(fmt = "(w:{},h:{})", w, h)]
// pub struct GenericWH {
//     w: GenericSize,
//     h: GenericSize,
// }
// impl Default for GenericWH {
//     fn default() -> Self {
//         Self {
//             w: px(0).into(),
//             h: px(0).into(),
//         }
//     }
// }
// impl GenericWH {
//     pub fn new(w: impl Into<GenericSize>, h: impl Into<GenericSize>) -> Self {
//         Self {
//             w: w.into(),
//             h: h.into(),
//         }
//     }

//     #[must_use]
//     pub fn get_length_value(&self) -> (f64, f64) {
//         (
//             self.w
//                 .try_get_length_value()
//                 .expect("root size w get failed, expected Length Px struct"),
//             self.h
//                 .try_get_length_value()
//                 .expect("root size h get failed, expected Length Px struct"),
//         )
//     }

    
// }


// impl Default for GenericLoc {
//     fn default() -> Self {
//         Self {
//             x: px(0).into(),
//             y: px(0).into(),
//             z: px(0).into(),
//         }
//     }
// }

// #[derive(Display, Debug, Clone, PartialEq, PartialOrd)]
// // #[derive(Debug, Clone, Into)]
// // #[into(owned, ref, ref_mut)]
// #[display(fmt = "(x:{},y:{},z:{})", x, y, z)]
// pub struct GenericLoc {
//     x: GenericSize,
//     y: GenericSize,
//     z: GenericSize,
// }

// impl GenericLoc {
//     pub fn new(
//         x: impl Into<GenericSize>,
//         y: impl Into<GenericSize>,
//         z: impl Into<GenericSize>,
//     ) -> Self {
//         Self {
//             x: x.into(),
//             y: y.into(),
//             z: z.into(),
//         }
//     }

//     #[must_use]
//     pub fn get_length_value(&self) -> (f64, f64, f64) {
//         (
//             self.x
//                 .try_get_length_value()
//                 .expect("root size get failed, expected Length Px struct"),
//             self.y
//                 .try_get_length_value()
//                 .expect("root size get failed, expected Length Px struct"),
//             self.z
//                 .try_get_length_value()
//                 .expect("root size get failed, expected Length Px struct"),
//         )
//     }
// }
#[derive(Debug, Clone, PartialEq)]
pub struct Layout
{
    w:StateVar<GenericSizeAnchor>,
    h:StateVar<GenericSizeAnchor>,
    z:StateVar<StateAnchor<u64>>,
    origin_x: StateVar<GenericSizeAnchor>,
    origin_y: StateVar<GenericSizeAnchor>,
    origin_z: StateVar<GenericSizeAnchor>,
    align_x: StateVar<GenericSizeAnchor>,
    align_y: StateVar<GenericSizeAnchor>,
    align_z: StateVar<GenericSizeAnchor>,
}

impl Layout
{
    /// Set the layout's size.
    #[cfg(test)]
    fn set_size(&self,
         w: impl Into<GenericSizeAnchor>,
        h: impl Into<GenericSizeAnchor>,) {
            self.w.set(w.into());
            self.h.set(h.into());
    }
    pub fn store_set_size(&self,store: &GStateStore,
        w: impl Into<GenericSizeAnchor>,
        h: impl Into<GenericSizeAnchor>,) {
        self.store_set_w(store,w);
        self.store_set_h(store,h);
    }

    pub fn store_set_w(&self, store: &GStateStore,w:impl Into<GenericSizeAnchor>) {
        self.w.store_set(store, w.into());

    }
    pub fn store_set_h(&self, store: &GStateStore,h:impl Into<GenericSizeAnchor>) {
        self.h.store_set(store, h.into());

    }


}
impl Copy for Layout 
{
}

impl std::fmt::Display for Layout
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let x = format!(
            "size:{{\nw:{},h:{};\n}}\norigin:{{\nx:{},y:{},z:{};\n}}\nalign:{{\nx:{},y:{},z:{}}}",
            indented(&self.w),
            indented(&self.h),
            indented(&self.origin_x),
            indented(&self.origin_y),
            indented(&self.origin_z),
            indented(&self.align_x),
            indented(&self.align_y),
            indented(&self.align_z),
            // 
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
struct PathVarMapDisplay<K, V>(PathVarMap<K, V>) where K: std::clone::Clone + std::hash::Hash + std::cmp::Eq + std::default::Default;

impl<K, V> std::fmt::Display for PathVarMapDisplay<K, V>
where
    K: std::fmt::Display + Ord,
    V: std::fmt::Display,
     K: std::clone::Clone + std::hash::Hash + std::cmp::Eq + std::default::Default
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let sv: String = self
            .0
            .iter()
            .map(|(k, v)| format!("{} :\n{}\n,\n", k, indented(v)))
            .fold(String::default(), |acc, v| format!("{}{}", acc, v));

        write!(f, "PathVarMap {{\n{}\n}}", indented(&sv))
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
    size: StateAnchor<Vector2<f64>>,
    origin: StateAnchor<Translation3<f64>>,
    align: StateAnchor<Translation3<f64>>,
    coordinates_trans: StateAnchor<Translation3<f64>>,
    matrix: StateAnchor<Mat4>,
    loc_styles: StateAnchor<Style>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EdgeData {
    calculated: LayoutCalculated,
    styles_string: StateAnchor<String>, 
    opt_p_calculated:Option<LayoutCalculated>,//TODO check need ? use for what?
    // matrix: M4Data,
                                        // transforms_am: Transforms,
                                        // animations:
}

impl EdgeData {
    #[must_use] pub fn styles_string(&self) -> String {
        self.styles_string.get()
    }
    #[must_use] pub fn store_styles_string(&self,store: &GStateStore) -> String {
        self.styles_string.store_get(store)
    }
    #[must_use] pub fn engine_styles_string(&self,engine: &mut Engine) -> String {
        engine.get(self.styles_string.anchor())
    }
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

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Hash, Debug, Default)]
pub struct EPath<Ix: Clone + Hash + Eq + PartialEq + Default>(Vector<EdgeIndex<Ix>>);

impl<Ix: Clone + Hash + Eq + PartialEq + Default> std::ops::Deref for EPath<Ix> {
    type Target = Vector<EdgeIndex<Ix>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<Ix: Clone + Hash + Eq + PartialEq + Default> std::ops::DerefMut for EPath<Ix> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<Ix: Clone + Hash + Eq + PartialEq + Default> EPath<Ix> {
    #[must_use] pub fn new(vec:Vector<EdgeIndex<Ix>>)->Self{
        Self(vec)
    }


    pub fn add_build(&self, target_nix:NodeIndex<Ix>)-> Self {
        let last = self.last().and_then(|e|e.target_nix().as_ref());
        let mut new_e = self.clone();
        new_e.push_back(EdgeIndex::new(last.cloned(),target_nix));
        new_e
        
    }
}

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

pub type GraphEdgesDict<Ix> = Dict<EdgeIndex<Ix>, Edge<EmgEdgeItem<Ix>, Ix>>;
// use ahash::AHasher as CustomHasher;
use rustc_hash::FxHasher as CustomHasher;

// type PathVarMap<Ix,T> = Dict<EPath<Ix>,T>;
// type PathVarMap<Ix,T> = indexmap::IndexMap <EPath<Ix>,T,BuildHasherDefault<CustomHasher>>;
type PathVarMap<Ix,T> = HashMap<EPath<Ix>,T,BuildHasherDefault<CustomHasher>>;
#[derive(Clone, Debug)]
pub struct EmgEdgeItem<Ix>
where
    // Ix: Clone + Hash + Eq + Ord + 'static + Default,
    Ix: Clone + Hash + Eq + Default + PartialOrd + std::cmp::Ord + 'static,
{
    //TODO save g_store
    pub id:StateVar< StateAnchor<EdgeIndex<Ix>>>,// dyn by Edge(source_nix , target_nix)
    pub paths:DictPathEiNodeSA<Ix>, // with parent self
    pub layout: Layout,
    path_styles: StateVar<PathVarMap<Ix, Style>>, //TODO check use
    path_layouts:StateVar<PathVarMap<Ix, Layout>>,

    pub other_styles: StateVar<Style>,
    // no self  first try
    pub node:DictPathEiNodeSA<Ix>, //TODO with self?  not with self?
    store:Rc<RefCell<GStateStore>>
}


impl<Ix> PartialEq for EmgEdgeItem<Ix> 
where 
Ix: Clone + Hash + Eq + Default + PartialOrd + std::cmp::Ord + 'static,

{
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.paths == other.paths && self.layout == other.layout && self.path_styles == other.path_styles
        && self.other_styles == other.other_styles && self.node == other.node
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
            "id:{{\n{};\n}}\npaths:{{\n{};\n}}\nlayout:{{\n{};\n}}\npath_styles:{{\n{};\n}}\nother_styles:{{\n{};\n}}\nnode:{{\n{};\n}}",
            indented(&self.id),
            indented(DictDisplay(self.paths.get())),
            indented(&self.layout),
            indented(PathVarMapDisplay(self.path_styles.get())),
            indented(&self.other_styles),
            indented(DictDisplay(self.node.get())),
        );
        write!(f, "EdgeDataWithParent {{\n{}\n}}", indented(&x))
    }
}
pub type DictPathEiNodeSA<Ix> = StateAnchor<Dict<EPath<Ix>, EdgeItemNode>>;


impl<Ix> EmgEdgeItem<Ix>
where
    Ix: Clone + Hash + Ord + Default 
{
    #[cfg(test)]
    fn set_size(&self,
        w: impl Into<GenericSizeAnchor>,
        h: impl Into<GenericSizeAnchor>,)
        {
        self.layout.set_size(w,h)
    }
    pub fn store_set_size(&self,store: &GStateStore,  
        w: impl Into<GenericSizeAnchor>,
        h: impl Into<GenericSizeAnchor>,){
        self.layout.store_set_size(store,w,h);
    }
    
    #[cfg(test)]
    #[must_use]
    fn edge_data(&self, key: &EPath<Ix>) -> Option<EdgeData> {
      
        self.node
            .get()
            .get(key)
            .and_then(EdgeItemNode::as_edge_data).cloned()
    }

    // #[must_use]
    // pub fn store_edge_data(&self,store:&GStateStore, key: &EPath<Ix>) -> Option<EdgeData> {
    //     self.node.store_get(store)
    //         .get(key)
    //         .and_then(EdgeItemNode::as_edge_data).cloned()
            
    // }
    pub fn store_edge_data_with<F: FnOnce(Option<&EdgeData>)->R,R>(&self,store:&GStateStore, key: &EPath<Ix>,func:F) -> R {
        self.node.store_get_with(store,|o|{
           func(o
            .get(key)
            .and_then(EdgeItemNode::as_edge_data) )
        })
       
            
    }
    pub fn engine_edge_data_with<F: FnOnce(Option<&EdgeData>)->R,R>(&self,engine:&mut Engine, key: &EPath<Ix>,func:F) -> R {
        self.node.engine_get_with(engine,|o|{
           func(o
            .get(key)
            .and_then(EdgeItemNode::as_edge_data) )
        })
       
            
    }
   
}

impl<Ix> EmgEdgeItem<Ix>
where
    Ix: Clone + Hash + Eq + PartialEq + PartialOrd + Ord + Default + std::fmt::Display,
{

    pub fn build_path_layout(&self,func:impl FnOnce(Layout)->(EPath<Ix>, Layout)){
        let (path,layout)   = func(self.layout);
        self.path_layouts.set_with_once(move|pls_map|{
            let mut new_pls_map = pls_map.clone();
            new_pls_map.insert(path, layout);
            new_pls_map
        } );
    }
  

    #[topo::nested]
    #[instrument(skip(edges))]
    pub fn default_in_topo(
        source_node_nix_sa: StateAnchor<Option<NodeIndex<Ix>>>,
        target_node_nix_sa: StateAnchor<Option<NodeIndex<Ix>>>,
        edges: StateAnchor<GraphEdgesDict<Ix>>,

         ) -> Self  where Ix:std::fmt::Debug{

        Self::new_in_topo(source_node_nix_sa, target_node_nix_sa, edges,    
            (GenericSize::default().into(), GenericSize::default().into()), 
            (GenericSize::default().into(),GenericSize::default().into(),GenericSize::default().into()),
            (GenericSize::default().into(),GenericSize::default().into(),GenericSize::default().into()),)

         }

    #[topo::nested]
    #[instrument(skip(edges))]
    pub fn default_with_wh_in_topo<T: Into<f64> + std::fmt::Debug>(
        source_node_nix_sa: StateAnchor<Option<NodeIndex<Ix>>>,
        target_node_nix_sa: StateAnchor<Option<NodeIndex<Ix>>>,
        edges: StateAnchor<GraphEdgesDict<Ix>>,
         w: T, h: T) -> Self  where Ix:std::fmt::Debug{

        Self::new_in_topo(source_node_nix_sa, target_node_nix_sa, edges,    
            (px(w).into(),  px(h).into()), 
            (GenericSize::default().into(),GenericSize::default().into(),GenericSize::default().into()),
            (GenericSize::default().into(),GenericSize::default().into(),GenericSize::default().into()),)

    }
    


    #[topo::nested]
    pub fn new_in_topo(

        source_node_nix_sa: StateAnchor<Option<NodeIndex<Ix>>>,
        target_node_nix_sa: StateAnchor<Option<NodeIndex<Ix>>>,
        edges: StateAnchor<GraphEdgesDict<Ix>>,
        size:  (GenericSizeAnchor,GenericSizeAnchor),
        origin: (GenericSizeAnchor,GenericSizeAnchor, GenericSizeAnchor),
        align:  (GenericSizeAnchor,GenericSizeAnchor,GenericSizeAnchor),
    ) -> Self 
    where Ix:std::fmt::Debug

    {
        let id_sa:StateAnchor <EdgeIndex<Ix> > =( &source_node_nix_sa,&target_node_nix_sa).map(|s,t| {
            let _g = span!(Level::TRACE, "[ id_sa recalculation ]:source_node_nix_sa/target_node_nix_sa change ").entered();
            
            EdgeIndex::new(s.clone(),t.clone())
        });
        let id_sv = use_state(id_sa);
        let _child_span = trace_span!(" building new child ",id=%id_sv).entered();
        // ─────────────────────────────────────────────────────────────────

        let layout = Layout{
            w: use_state(size.0),
            h: use_state(size.1),
            z: use_state(StateAnchor::constant(0)),
            origin_x: use_state(origin.0),
            origin_y: use_state(origin.1),
            origin_z: use_state(origin.2),
            align_x: use_state(align.0),
            align_y: use_state(align.1),
            align_z: use_state(align.2),
        };
        // let path_styles= use_state(Dict::unit(EPath::<Ix>::default(), s()));
        let path_styles:StateVar<PathVarMap<Ix, Style>> = use_state(PathVarMap::default());
        let path_layouts:StateVar<PathVarMap<Ix, Layout>> = use_state(PathVarMap::default());
      

        let other_styles_sv = use_state(s());

        let opt_source_node_nix_sa_re_get:StateAnchor<Option<NodeIndex<Ix>>> = id_sv.watch().then(|eid_sa_inner|{
            let _g = trace_span!( "[ source_node_nix_sa_re_get recalculation ]:id_sv change ").entered();

            eid_sa_inner.map(|i:&EdgeIndex<Ix>|{
                
                let _g = span!(Level::TRACE, "[ source_node_nix_sa_re_get recalculation ]:eid_sa_inner change ",edge_index=%i).entered();
                
                i.source_nix().clone()
            }).into()
        });

        let paths: DictPathEiNodeSA<Ix> = 
            opt_source_node_nix_sa_re_get.then(move|opt_source_nix:&Option<NodeIndex<Ix>>| {

                let _g = span!(Level::TRACE, "[ source_node_incoming_edge_dict_sa recalculation ]:source_node_nix_sa_re_get change ").entered();

                if opt_source_nix.is_none(){
                    //NOTE 如果 source nix  是没有 node index 那么他就是无上一级的
                    Anchor::constant(Dict::<EPath<Ix>, EdgeItemNode>::unit(EPath::<Ix>::default(), EdgeItemNode::Empty))
                }else{
                    let opt_source_nix_clone = opt_source_nix.clone();
                    edges.filter_map(move|someone_eix, e| {
                        
                        println!("********************** \n one_eix.target_node_ix: {:?} ?? opt_source_nix_clone:{:?}",someone_eix.target_nix(),&opt_source_nix_clone);
                        if   someone_eix.target_nix() == &opt_source_nix_clone {

                            Some(e.item.node.clone())

                        }else{
                            None
                        }
                        
                    })
                    .anchor()
                    .then(|x:&Dict<EdgeIndex<Ix>, DictPathEiNodeSA<Ix>>|{
                        x.values().map(emg_state::StateAnchor::anchor).collect::<Anchor<Vector<_>>>()
                        .map(|v:&Vector<_>|{
                            let _g = trace_span!( "[  paths dict recalculation ]:vector paths change ").entered();
                            Dict::unions(v.clone())})
                    })
                }
                       

                
            });

       

        //TODO not paths: StateVar<Dict<EPath<Ix>,EdgeItemNode>>  use edgeIndex instead to Reduce memory
        let paths_clone = paths.clone();
        let node:DictPathEiNodeSA<Ix> = id_sv.watch().then(move|id_sa|{
            
            let paths_clone2 = paths_clone.clone();
                
            id_sa.then(move |eid:&EdgeIndex<Ix>|{

                let eid_clone = eid.clone();
                
                paths_clone2.map(move |p_node_as_paths:&Dict<EPath<Ix>, EdgeItemNode>|{
                    
                    p_node_as_paths.iter()
                        .map(|(parent_e_node_k, p_ei_node_v)| {
                            let mut nk = parent_e_node_k.clone();
                            
                            //TODO node 可以自带 self nix ,下游不必每个子节点都重算

                            nk.0.push_back(eid_clone.clone());
                            (nk, p_ei_node_v.clone())
                        })
                        .collect::<Dict<EPath<Ix>, EdgeItemNode>>()

                }).map_( move |path:&EPath<Ix>, path_edge_item_node:&EdgeItemNode| {

                    let _child_span =
                        span!(Level::TRACE, "[ node recalculation ]:paths change ").entered();
                        
                       
                    let (opt_p_calculated,layout_calculated,styles_string) =  match path_edge_item_node {
                        //NOTE 上一级节点: empty => 此节点是root
                        EdgeItemNode::Empty => path_ein_empty_node_builder(&layout, path,path_layouts,path_styles, other_styles_sv),
                        EdgeItemNode::EdgeData(ped)=> path_with_ed_node_builder(id_sv, ped, &layout, path,path_layouts, path_styles, other_styles_sv),
                        EdgeItemNode::String(_)  => {
                            todo!("parent is EdgeItemNode::String(_) not implemented yet");
                        }
                                
                    };
                    EdgeItemNode::EdgeData(EdgeData {
                        calculated: layout_calculated,
                        styles_string,
                        opt_p_calculated
                    })
                }).into()

            }).into()
        });
            

        Self {
            id: id_sv,
            paths,
            layout,
            path_styles,
            path_layouts,
            other_styles: other_styles_sv,
            node,
            store:state_store()
        }
    }
}


fn path_with_ed_node_builder<Ix>(
    id_sv: StateVar<StateAnchor<EdgeIndex<Ix>>>, 
    ped: &EdgeData,
     layout: &Layout,
      path: &EPath<Ix>, 
      path_layouts:StateVar<PathVarMap<Ix, Layout>>,
      path_styles: StateVar<PathVarMap<Ix, Style>>,
      other_styles_sv: StateVar<Style>) -> (Option<LayoutCalculated>, LayoutCalculated, StateAnchor<String>) 
where
Ix: std::clone::Clone + std::hash::Hash + std::default::Default + std::cmp::Ord +std::fmt::Display+'static+ std::fmt::Debug
{
    // println!("run path_with_ed_node_builder ******************************************************************");


    let p_calculated = ped.calculated.clone();
    let path_clone = path.clone();
    let path_clone2 = path.clone();
    let layout_c = *layout;
    //TODO use Dict anchor Collection
    let path_layout:StateAnchor<Layout> = path_layouts.watch().map(move|pls_map:&PathVarMap<Ix, Layout>|{
        // println!("--> id: {:?}", &id_sv);
        trace!("--> find path_layout in path_with_ed_node_builder------------------- len:{}",pls_map.len());
        // println!("--> layout:{:?}",pls_map.get(&path_clone));
        *pls_map.get(&path_clone).unwrap_or(&layout_c)
    });
    let layout_calculated = layout_calculating(id_sv, ped, path_layout);
    // let p = path.clone();
    let this_path_style_string_sa: StateAnchor<Option<String>> = 
                        path_styles
                        .watch()
                        .map(move|d: &PathVarMap<Ix, Style>| {
                            let _g = trace_span!( "[  this_path_style_string_sa recalculation ]:layout.path_styles change ").entered();
    
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
    (Some(p_calculated),layout_calculated,styles_string)
}

fn path_ein_empty_node_builder<Ix:'static>(
    layout: &Layout,
    path: &EPath<Ix>, 
    path_layouts:StateVar<PathVarMap<Ix, Layout>>,
    path_styles:StateVar<PathVarMap<Ix, Style>>, other_styles_sv: StateVar<Style>) -> (Option<LayoutCalculated>,LayoutCalculated, StateAnchor<String>)
 where 
    Ix: std::clone::Clone + std::hash::Hash + std::default::Default + std::cmp::Ord 
    {
        // println!("run path_ein_empty_node_builder ******************************************************************");

        // ─────────────────────────────────────────────────────────────────
        let layout_c = *layout;
        let path_clone = path.clone();
        let path_layout:StateAnchor<Layout> = path_layouts.watch().map(move|pls_map:&PathVarMap<Ix, Layout>|{
            // println!("--> id: {:?}", &id_sv);
            trace!("--> find path_layout--in root ----------------- len:{}",pls_map.len());
            // println!("--> layout:{:?}",pls_map.get(&path_clone));
            debug_assert!(pls_map.len()<=1_usize);
            
            *pls_map.get(&path_clone).unwrap_or(&layout_c)
        });
        let w = path_layout.then(|l:&Layout|l.w.watch().into());
        let h = path_layout.then(|l:&Layout|l.h.watch().into());
        // let origin_x = path_layout.then(|l:&Layout|l.origin_x.watch().into());
        // let origin_y = path_layout.then(|l:&Layout|l.origin_y.watch().into());
        // let align_x = path_layout.then(|l:&Layout|l.align_x.watch().into());
        // let align_y = path_layout.then(|l:&Layout|l.align_y.watch().into());
        // ─────────────────────────────────────────────────────────────────

        //TODO 如果没有parent 那么 parent 就是 screen w h
    let calculated_size:StateAnchor<Vector2<f64>> = (&w,&h).then(|sa_w: &GenericSizeAnchor,sa_h: &GenericSizeAnchor| {
            (&**sa_w,&**sa_h).map(|w:&GenericSize,h:&GenericSize|->Vector2<f64>{
                //TODO check editor display error 
                Vector2::<f64>::from_vec(vec![w.get_length_value(), h.get_length_value()])
                // Vector2::<f64>::new(w.get_length_value(), h.get_length_value())
            }).into()    
            
        });

        //TODO 审视是否要自定义定位
    let calculated_origin = StateAnchor::constant(Translation3::<f64>::identity());
    let calculated_align = StateAnchor::constant(Translation3::<f64>::identity());
    let coordinates_trans = StateAnchor::constant(Translation3::<f64>::identity());
    let matrix = coordinates_trans.map(|x| x.to_homogeneous().into());
    let loc_styles = (&calculated_size, &matrix).map(move |size: &Vector2<f64>, mat4: &Mat4| {
            let _enter = span!(Level::TRACE,
                        "-> [root] [ loc_styles ] recalculation..(&calculated_size, &matrix).map ",
                        )
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
    let styles_string = (
            &path_styles.watch(),
            &layout_calculated.loc_styles,
            &other_styles_sv.watch(),
        )
        .map(
            move |path_styles: &PathVarMap<Ix,Style>, loc_styles: &Style, other_styles: &Style| {
                let _enter = span!(Level::TRACE,
                        "-> [ROOT styles ] recalculation..(&other_styles_watch, &loc_styles).map ",
                        )
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
    (None,layout_calculated,styles_string)
}



#[derive(Display, From, Clone, Debug, PartialEq, Eq)]
pub enum EdgeItemNode {
    EdgeData(EdgeData),
    String(String), //TODO make can write, in DictPathEiNodeSA it clone need RC?
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
pub fn css<
Ix: Clone + Hash + Eq + Ord + 'static + Default,
    Use: CssValueTrait + Clone + 'static,
>(
    v: Use,
) -> Box<dyn RefreshFor<EmgEdgeItem<Ix>>> {
    // pub fn css<Use: CssValueTrait + std::clone::Clone + 'static>(v: Use) -> Box<Css<Use>> {
    Box::new(Css(v))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::too_many_lines)]
    use crate::*;
    extern crate test;


    use emg::{edge_index, edge_index_no_source, node_index};
    use emg_core::parent;
    use emg_refresh::{RefreshForUse, RefreshWhoNoWarper};
    use emg_state::StateVar;
    use emg_core::vector;
 
    use styles::{CssBackgroundColorTrait,CssWidth, CssHeight, h, hsl, pc, width};
    use tracing::{debug, info, span, warn};

    use tracing_flame::FlameLayer;
    use tracing_subscriber::{fmt, prelude::*, EnvFilter};
    use test::{black_box, Bencher};

    fn setup_global_subscriber() -> impl Drop {
        std::env::set_var("RUST_LOG", "trace");
        std::env::set_var("RUST_LOG", "warn");

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

            let css_width = width(px(100));
            let css_height = h(px(100));
            let e_dict_sv:StateVar<GraphEdgesDict<String>> = use_state(Dict::new());


            let root_e_source =use_state( None);
            let root_e_target = use_state(Some(node_index("root")));
            let mut root_e = EmgEdgeItem::<String>::default_with_wh_in_topo(root_e_source.watch(), root_e_target.watch(),e_dict_sv.watch(),1920, 1080);
            e_dict_sv.set_with(|d|{
                let mut nd = d .clone();
                nd.insert(EdgeIndex::new(None,node_index("root")), Edge::new(root_e_source, root_e_target, root_e.clone()));
                nd
            });
                

            let e1_source =use_state( Some(node_index("root")));
            let e1_target = use_state(Some(node_index("1")));
            let e1 = EmgEdgeItem::<String>::new_in_topo(
                    e1_source.watch(),
                    e1_target.watch(),
                e_dict_sv.watch(),
                (px(50).into(), px(50).into()),
                 (pc(0).into(), pc(0).into(), pc(0).into()),
                  (pc(50).into(), pc(50).into(), pc(50).into()),
            );

            e_dict_sv.set_with(|d|{
                let mut nd = d .clone();
                nd.insert(edge_index("root","1"), Edge::new(e1_source, e1_target, e1.clone()));
                nd
            });

            
            let e2_source =use_state( Some(node_index("1")));
            let e2_target = use_state(Some(node_index("2")));
            let mut e2 = EmgEdgeItem::<String>::new_in_topo(
                e2_source.watch(),
                    e2_target.watch(),
                  e_dict_sv.watch(),
                (px(10).into(), px(10).into()),
                 (pc(100).into(), pc(100).into(), pc(100).into()), 
                 (pc(100).into(), pc(100).into(), pc(100).into()),
            );
            e_dict_sv.set_with(|d|{
                let mut nd = d .clone();
                nd.insert(edge_index("1","2"), Edge::new(e2_source, e2_target, e2.clone()));
                nd
            });


            // debug!("refresh_use before {}", &ec);
            let _span = span!(Level::TRACE, "debug print e1");
            _span.in_scope(|| {
                trace!("loc refresh_use before {}", &e1);
            });
            info!("l2 =========================================================");
            
            root_e.refresh_for_use(&vec![css(css_width)]);
            // root_e.refresh_use(&css(css_width.clone()));
            root_e.refresh_for_use(&Css(css_height));
            assert_eq!(
                e1.edge_data(&EPath(vector![edge_index_no_source("root"), edge_index("root", "1")]))
                    .unwrap()
                    .calculated
                    .coordinates_trans
                    .get(),
                Translation3::<f64>::new(50., 50., 0.)
            );
            info!("=========================================================");

            e2.refresh_for_use(&Css(CssWidth::from(px(20))));
            e2.refresh_for_use(&Css(CssHeight::from(px(20))));

            trace!("refresh_use after {:#?}", &e2);
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
                Translation3::<f64>::new(30., 30., 0.)
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
            insta::assert_debug_snapshot!("loc-root_e", &root_e);
            insta::assert_debug_snapshot!("loc-e1", &e1);
            insta::assert_debug_snapshot!("loc-e2", &e2);
            
            info!("..=========================================================");
        }
    }
    #[bench]
    #[topo::nested]
    fn it_works_bench(b: &mut Bencher){
        b.iter(|| {
           black_box( it_works_for_bench());
        });
    }
    
    fn it_works_for_bench() {
           

            info!("--------------------=====================================");
            // vec![ CssWidth::from(px(100))].up
            info!("=========================================================");

            // let cc = Affine3<f64>::identity();
            let _ff = s().width(pc(11)).bg_color(hsl(40,70,30));
            let css_width = width(px(100));
            let css_height = h(px(100));

            let e_dict_sv:StateVar<GraphEdgesDict<&str>> = use_state(Dict::new());

            let root_e_source =use_state( None);
            let root_e_target = use_state(Some(node_index("root")));
            let mut root_e = EmgEdgeItem::default_with_wh_in_topo(root_e_source.watch(), root_e_target.watch(),e_dict_sv.watch(),1920, 1080);
            e_dict_sv.set_with(|d|{
                let mut nd = d .clone();
                nd.insert(EdgeIndex::new(None,Some(node_index("root"))), Edge::new(root_e_source, root_e_target, root_e.clone()));
                nd
            });



            let e1_source =use_state( Some(node_index("root")));
            let e1_target = use_state(Some(node_index("1")));
            let mut e1 = EmgEdgeItem::new_in_topo(
                    e1_source.watch(),
                    e1_target.watch(),
                e_dict_sv.watch(),
                (px(10).into(), px(10).into()), 
                (pc(100).into(), pc(100).into(), pc(100).into()), 
                (pc(50).into(), pc(20).into(), pc(20).into()),
            );
            e_dict_sv.set_with(|d|{
                let mut nd = d .clone();
                nd.insert(edge_index("root","1"), Edge::new(e1_source, e1_target, e1.clone()));
                nd
            });

            let e2_source =use_state( Some(node_index("1")));
            let e2_target = use_state(Some(node_index("2")));
            let mut e2 = EmgEdgeItem::new_in_topo(
                e2_source.watch(),
                    e2_target.watch(),
                  e_dict_sv.watch(),
            (px(10).into(), px(10).into()), 
            (pc(100).into(), pc(100).into(), pc(100).into()), 
            (pc(50).into(), pc(20).into(), pc(20).into()),
            );
            e_dict_sv.set_with(|d|{
                let mut nd = d .clone();
                nd.insert(edge_index("1","2"), Edge::new(e2_source, e2_target, e2.clone()));
                nd
            });
          


            // debug!("refresh_use before {}", &ec);
            let _span = span!(Level::TRACE, "debug print e1");
            _span.in_scope(|| {
                trace!("refresh_use before {}", &e1);
            });
            info!("l1 =========================================================");

            assert_eq!(
                e1.node
                    .get()
                    .get(&EPath(vector![edge_index_no_source("root"),edge_index("root","1")]))
                    .and_then(EdgeItemNode::as_edge_data)
                    .unwrap()
                    .calculated
                    .coordinates_trans
                    .get(),
                Translation3::<f64>::new(950.0, 206.0, 0.)
            );


            let xx = vec![css_width];
            // let xx = vec![css(css_width)];

            root_e.refresh_for_use(&xx);
            root_e.refresh_for_use(&xx);
            root_e.refresh_for_use(&xx);
            root_e.refresh_for_use(&xx);
            root_e.refresh_for_use(&xx);
            root_e.refresh_for_use(&xx);
            root_e.refresh_for_use(&xx);
            root_e.refresh_for_use(&xx);
            root_e.refresh_for_use(&xx);
            root_e.refresh_for_use(&xx);
            root_e.refresh_for_use(&xx);
            root_e.refresh_for_use(&xx);
            root_e.refresh_for_use(&xx);
            root_e.refresh_for_use(&xx);
            root_e.refresh_for_use(&xx);
            root_e.refresh_for_use(&xx);
            root_e.refresh_for_use(&xx);
            root_e.refresh_for_use(&xx);
            root_e.refresh_for_use(&xx);
            root_e.refresh_for_use(&xx);
            root_e.refresh_for_use(&xx);
            root_e.refresh_for_use(&xx);
            root_e.refresh_for_use(&xx);
            root_e.refresh_for_use(&xx);
            root_e.refresh_for_use(&xx);
            root_e.refresh_for_use(&xx);
            root_e.refresh_for_use(&xx);
            root_e.refresh_for_use(&xx);
            root_e.refresh_for_use(&xx);
            root_e.refresh_for_use(&xx);
            root_e.refresh_for_use(&xx);
            root_e.refresh_for_use(&xx);
            root_e.refresh_for_use(&xx);
            root_e.refresh_for_use(&xx);
            root_e.refresh_for_use(&xx);

            // trace!("refresh_use after css_width {}", &root_e);
            trace!("refresh_use after css_width {}", &e1);
            info!("=========================================================");

            // root_e.refresh_use(&Css(css_height.clone()));
            let tempcss= use_state(css_height);
            root_e.refresh_for_use(&tempcss);
            assert_eq!(
                e1.node
                    .get()
                    .get(&EPath(vector![edge_index_no_source("root"), edge_index("root", "1")]))
                    .and_then(EdgeItemNode::as_edge_data)
                    .unwrap()
                    .calculated
                    .coordinates_trans
                    .get(),
                Translation3::<f64>::new(40., 10., 0.)
            );
            info!("=========================================================");
            tempcss.set(h(px(1111)));
            assert_ne!(
                e1.node
                    .get()
                    .get(&EPath(vector![edge_index_no_source("root"), edge_index("root", "1")]))
                    .and_then(EdgeItemNode::as_edge_data)
                    .unwrap()
                    .calculated
                    .coordinates_trans
                    .get(),
                Translation3::<f64>::new(40., 10., 0.)
            );
            tempcss.set(h(px(100)));
            assert_eq!(
                e1.node
                    .get()
                    .get(&EPath(vector![edge_index_no_source("root"), edge_index("root", "1")]))
                    .and_then(EdgeItemNode::as_edge_data)
                    .unwrap()
                    .calculated
                    .coordinates_trans
                    .get(),
                Translation3::<f64>::new(40., 10., 0.)
            );

            info!("=========================================================");

            e1.refresh_for_use(&Css(CssWidth::from(px(12))));
            e1.refresh_for_use(&Css(CssWidth::from(px(12))));
            e1.refresh_for_use(&Css(CssWidth::from(px(12))));
            e1.refresh_for_use(&Css(CssWidth::from(px(12))));
            e1.refresh_for_use(&Css(CssWidth::from(px(12))));
            e1.refresh_for_use(&Css(CssWidth::from(px(12))));
            e1.refresh_for_use(&Css(CssWidth::from(px(12))));
            e1.refresh_for_use(&Css(CssWidth::from(px(12))));
            e1.refresh_for_use(&Css(CssWidth::from(px(12))));
            e1.refresh_for_use(&Css(CssWidth::from(px(12))));
            e1.refresh_for_use(&Css(CssWidth::from(px(12))));
            e1.refresh_for_use(&Css(CssWidth::from(px(12))));
            e1.refresh_for_use(&Css(CssWidth::from(px(12))));
            e1.refresh_for_use(&Css(CssWidth::from(px(12))));
            e1.refresh_for_use(&Css(CssWidth::from(px(12))));
            e1.refresh_for_use(&Css(CssWidth::from(px(12))));
            e1.refresh_for_use(&Css(CssWidth::from(px(12))));
            e1.refresh_for_use(&Css(CssWidth::from(px(12))));
            e1.refresh_for_use(&Css(CssWidth::from(px(12))));
            e1.refresh_for_use(&Css(CssWidth::from(px(12))));
            e1.refresh_for_use(&Css(CssWidth::from(px(12))));
            e1.refresh_for_use(&Css(CssWidth::from(px(12))));
            e1.refresh_for_use(&Css(CssWidth::from(px(12))));
            e1.refresh_for_use(&Css(CssWidth::from(px(12))));
            e1.refresh_for_use(&Css(CssWidth::from(px(12))));
            e1.refresh_for_use(&Css(CssWidth::from(px(12))));
            e1.refresh_for_use(&Css(CssWidth::from(px(12))));
            e1.refresh_for_use(&Css(CssWidth::from(px(12))));
            e1.refresh_for_use(&Css(CssWidth::from(px(12))));
            e1.refresh_for_use(&Css(CssWidth::from(px(12))));
            assert_eq!(
                e1.node
                    .get()
                    .get(&EPath(vector![edge_index_no_source("root"), edge_index("root", "1")]))
                    .and_then(EdgeItemNode::as_edge_data)
                    .unwrap()
                    .calculated
                    .size
                    .get(),
                Vector2::<f64>::new(12., 10.)
            );
            info!("=========================================================");
            assert_eq!(
                e1.node.get().get(&EPath(vector![edge_index_no_source("root"), edge_index("root", "1")]))
                    .and_then(EdgeItemNode::as_edge_data)
                .unwrap()
                .styles_string
                .get() ,
            "width: 12px;\nheight: 10px;\ntransform: matrix3d(1,0,0,0,0,1,0,0,0,0,1,0,38,10,0,1);\n"
            );
            trace!("refresh_use after {}", &e1);
            info!("=========================================================");

            trace!("refresh_use after {}", &e2);
            info!("l1351 =========================================================");
            e2.refresh_for_use(&Css(CssHeight::from(px(50))));
            assert_eq!(
                e2.node
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
                Translation3::<f64>::new(-4.0, -48.0, 0.0)
            );
            e2.set_size(px(100), px(100));
            assert_eq!(
                e2.node
                    .get()
                    .get(&EPath(vector![
                        edge_index_no_source("root"),
                        edge_index("root", "1"),
                        edge_index("1", "2")
                    ]))
                    .and_then(EdgeItemNode::as_edge_data)
                    .unwrap()
                    .calculated
                    .size
                    .get(),
                    Vector2::<f64>::new(100.0, 100.0)
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
            e2.refresh_for_use(&Css(CssHeight::from(px(150))));

            trace!("refresh_use after {:#?}", &e2);
            info!("..=========================================================");
            trace!(
                "{}",
                e2.node
                    .get()
                    .get(&EPath(vector![
                        edge_index_no_source( "root"),
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
    fn it_works() {
        let _xx = setup_global_subscriber();
        {
            let span = span!(Level::TRACE, "start");
            let _guard = span.enter();

            info!("--------------------=====================================");
            // vec![ CssWidth::from(px(100))].up
            info!("=========================================================");

            // let cc = Affine3<f64>::identity();
            let _build_test = s().width(pc(11)).bg_color(hsl(40,70,30));
            let css_width = width(px(100));
            let css_height = h(px(100));

            let e_dict_sv:StateVar<GraphEdgesDict<&str>> = use_state(Dict::new());

            let root_e_source =use_state( None);
            let root_e_target = use_state(Some(node_index("root")));
            let mut root_e = EmgEdgeItem::default_with_wh_in_topo(root_e_source.watch(), root_e_target.watch(),e_dict_sv.watch(),1920, 1080);
            e_dict_sv.set_with(|d|{
                let mut nd = d .clone();
                nd.insert(EdgeIndex::new(None,Some(node_index("root"))), Edge::new(root_e_source, root_e_target, root_e.clone()));
                nd
            });



            let e1_source =use_state( Some(node_index("root")));
            let e1_target = use_state(Some(node_index("1")));
            let mut e1 = EmgEdgeItem::new_in_topo(
                    e1_source.watch(),
                    e1_target.watch(),
                e_dict_sv.watch(),
                ((parent!(CssHeight)+ pc(100)).into(), (parent!(CssHeight)+ pc(100)).into()), 
                (pc(100).into(), pc(100).into(), pc(100).into()), 
                ((parent!(CssWidth)*0.5).into(), (parent!(CssHeight)*0.2).into(), pc(20).into()),
            );
            e_dict_sv.set_with(|d|{
                let mut nd = d .clone();
                nd.insert(edge_index("root","1"), Edge::new(e1_source, e1_target, e1.clone()));
                nd
            });

            let e2_source =use_state( Some(node_index("1")));
            let e2_target = use_state(Some(node_index("2")));
            let mut e2 = EmgEdgeItem::new_in_topo(
                e2_source.watch(),
                    e2_target.watch(),
                  e_dict_sv.watch(),
            (px(10).into(), px(10).into()), 
            (pc(100).into(), pc(100).into(), pc(100).into()), 
            (pc(50).into(), pc(20).into(), pc(20).into()),
            );
            e_dict_sv.set_with(|d|{
                let mut nd = d .clone();
                nd.insert(edge_index("1","2"), Edge::new(e2_source, e2_target, e2.clone()));
                nd
            });
          


            // debug!("refresh_use before {}", &ec);
            let _span = span!(Level::TRACE, "debug print e1");
            _span.in_scope(|| {
                trace!("refresh_use before {}", &e1);
            });
            info!("l1 =========================================================");
            warn!("calculated 1 =========================================================");
            warn!("{}",e1.node
            .get()
            .get(&EPath(vector![edge_index_no_source("root"),edge_index("root","1")]))
            .and_then(EdgeItemNode::as_edge_data)
            .unwrap()
            .calculated);
            assert_eq!(
                e1.node
                    .get()
                    .get(&EPath(vector![edge_index_no_source("root"),edge_index("root","1")]))
                    .and_then(EdgeItemNode::as_edge_data)
                    .unwrap()
                    .calculated
                    .coordinates_trans
                    .get(),
                Translation3::<f64>::new(-2040.0, -1944.0, 0.)
            );
            warn!("calculated 2 =========================================================");
            warn!("{}",e1.node
            .get()
            .get(&EPath(vector![edge_index_no_source("root"),edge_index("root","1")]))
            .and_then(EdgeItemNode::as_edge_data)
            .unwrap()
            .calculated);

            let xx = vec![css_width];
            // let xx = vec![css(css_width)];

            root_e.refresh_for_use(&xx);

            warn!("calculated 3 =========================================================");
            warn!("{}",e1.node
            .get()
            .get(&EPath(vector![edge_index_no_source("root"),edge_index("root","1")]))
            .and_then(EdgeItemNode::as_edge_data)
            .unwrap()
            .calculated);

            root_e.refresh_for_use(&xx);
            root_e.refresh_for_use(&xx);
            root_e.refresh_for_use(&xx);
            root_e.refresh_for_use(&xx);
            root_e.refresh_for_use(&xx);
            root_e.refresh_for_use(&xx);
            root_e.refresh_for_use(&xx);
            root_e.refresh_for_use(&xx);
            root_e.refresh_for_use(&xx);
            root_e.refresh_for_use(&xx);
            root_e.refresh_for_use(&xx);
            root_e.refresh_for_use(&xx);
            root_e.refresh_for_use(&xx);
            root_e.refresh_for_use(&xx);
            root_e.refresh_for_use(&xx);
            root_e.refresh_for_use(&xx);
            root_e.refresh_for_use(&xx);
            root_e.refresh_for_use(&xx);
            root_e.refresh_for_use(&xx);
            root_e.refresh_for_use(&xx);
            root_e.refresh_for_use(&xx);
            root_e.refresh_for_use(&xx);
            root_e.refresh_for_use(&xx);
            root_e.refresh_for_use(&xx);
            root_e.refresh_for_use(&xx);
            root_e.refresh_for_use(&xx);
            root_e.refresh_for_use(&xx);
            root_e.refresh_for_use(&xx);
            root_e.refresh_for_use(&xx);
            root_e.refresh_for_use(&xx);
            root_e.refresh_for_use(&xx);
            root_e.refresh_for_use(&xx);
            root_e.refresh_for_use(&xx);
            root_e.refresh_for_use(&xx);

            // trace!("refresh_use after css_width {}", &root_e);
            trace!("refresh_use after css_width {}", &e1);
            info!("=========================================================");

            // root_e.refresh_use(&Css(css_height.clone()));
            let tempcss= use_state(css_height);
            root_e.refresh_for_use(&tempcss);
            
            warn!("calculated 4 root h w 100 =========================================================");
            warn!("{}",e1.node
            .get()
            .get(&EPath(vector![edge_index_no_source("root"),edge_index("root","1")]))
            .and_then(EdgeItemNode::as_edge_data)
            .unwrap()
            .calculated);

            assert_eq!(
                e1.node
                    .get()
                    .get(&EPath(vector![edge_index_no_source("root"), edge_index("root", "1")]))
                    .and_then(EdgeItemNode::as_edge_data)
                    .unwrap()
                    .calculated
                    .coordinates_trans
                    .get(),
                Translation3::<f64>::new(-150.0, -180.0, 0.)
            );
            info!("=========================================================");
            tempcss.set(h(px(1111)));
            let _ff =  e1.node
            .get()
            .get(&EPath(vector![edge_index_no_source("root"), edge_index("root", "1")]))
            .and_then(EdgeItemNode::as_edge_data)
                    .unwrap()
                    .styles_string
                    .get();
            assert_ne!(
                e1.node
                    .get()
                    .get(&EPath(vector![edge_index_no_source("root"), edge_index("root", "1")]))
                    .and_then(EdgeItemNode::as_edge_data)
                    .unwrap()
                    .calculated
                    .coordinates_trans
                    .get(),
                Translation3::<f64>::new(40., 10., 0.)
            );
            tempcss.set(h(px(100)));
            assert_eq!(
                e1.node
                    .get()
                    .get(&EPath(vector![edge_index_no_source("root"), edge_index("root", "1")]))
                    .and_then(EdgeItemNode::as_edge_data)
                    .unwrap()
                    .calculated
                    .coordinates_trans
                    .get(),
                Translation3::<f64>::new(-150.0, -180.0, 0.)
            );

            info!("=========================================================");

            e1.refresh_for_use(&Css(CssWidth::from(px(12))));
            e1.refresh_for_use(&Css(CssWidth::from(px(12))));
            e1.refresh_for_use(&Css(CssWidth::from(px(12))));
            e1.refresh_for_use(&Css(CssWidth::from(px(12))));
            e1.refresh_for_use(&Css(CssWidth::from(px(12))));
            e1.refresh_for_use(&Css(CssWidth::from(px(12))));
            e1.refresh_for_use(&Css(CssWidth::from(px(12))));
            e1.refresh_for_use(&Css(CssWidth::from(px(12))));
            e1.refresh_for_use(&Css(CssWidth::from(px(12))));
            e1.refresh_for_use(&Css(CssWidth::from(px(12))));
            e1.refresh_for_use(&Css(CssWidth::from(px(12))));
            e1.refresh_for_use(&Css(CssWidth::from(px(12))));
            e1.refresh_for_use(&Css(CssWidth::from(px(12))));
            e1.refresh_for_use(&Css(CssWidth::from(px(12))));
            e1.refresh_for_use(&Css(CssWidth::from(px(12))));
            e1.refresh_for_use(&Css(CssWidth::from(px(12))));
            e1.refresh_for_use(&Css(CssWidth::from(px(12))));
            e1.refresh_for_use(&Css(CssWidth::from(px(12))));
            e1.refresh_for_use(&Css(CssWidth::from(px(12))));
            e1.refresh_for_use(&Css(CssWidth::from(px(12))));
            e1.refresh_for_use(&Css(CssWidth::from(px(12))));
            e1.refresh_for_use(&Css(CssWidth::from(px(12))));
            e1.refresh_for_use(&Css(CssWidth::from(px(12))));
            e1.refresh_for_use(&Css(CssWidth::from(px(12))));
            e1.refresh_for_use(&Css(CssWidth::from(px(12))));
            e1.refresh_for_use(&Css(CssWidth::from(px(12))));
            e1.refresh_for_use(&Css(CssWidth::from(px(12))));
            e1.refresh_for_use(&Css(CssWidth::from(px(12))));
            e1.refresh_for_use(&Css(CssWidth::from(px(12))));
            e1.refresh_for_use(&Css(CssWidth::from(px(12))));
            
            assert_eq!(
                e1.node
                    .get()
                    .get(&EPath(vector![edge_index_no_source("root"), edge_index("root", "1")]))
                    .and_then(EdgeItemNode::as_edge_data)
                    .unwrap()
                    .calculated
                    .size
                    .get(),
                Vector2::<f64>::new(12., 200.)
            );
            info!("=========================================================");
            assert_eq!(
                e1.node.get().get(&EPath(vector![edge_index_no_source("root"), edge_index("root", "1")]))
                    .and_then(EdgeItemNode::as_edge_data)
                .unwrap()
                .styles_string
                .get() ,
            "width: 12px;\nheight: 200px;\ntransform: matrix3d(1,0,0,0,0,1,0,0,0,0,1,0,38,-180,0,1);\n"
            );
            trace!("refresh_use after {}", &e1);
            info!("=========================================================");

            trace!("refresh_use after {}", &e2);
            info!("l1351 =========================================================");
            e2.refresh_for_use(&Css(CssHeight::from(px(50))));
            assert_eq!(
                e2.node
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
                Translation3::<f64>::new(-4.0, -10.0, 0.0)
            );
            e2.set_size(px(100), px(100));
            assert_eq!(
                e2.node
                    .get()
                    .get(&EPath(vector![
                        edge_index_no_source("root"),
                        edge_index("root", "1"),
                        edge_index("1", "2")
                    ]))
                    .and_then(EdgeItemNode::as_edge_data)
                    .unwrap()
                    .calculated
                    .size
                    .get(),
                    Vector2::<f64>::new(100.0, 100.0)
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
            e2.refresh_for_use(&Css(CssHeight::from(px(150))));

            trace!("refresh_use after {:#?}", &e2);
            info!("..=========================================================");
            trace!(
                "{}",
                e2.node
                    .get()
                    .get(&EPath(vector![
                        edge_index_no_source( "root"),
                        edge_index("root", "1"),
                        edge_index("1", "2")
                    ]))
                    .and_then(EdgeItemNode::as_edge_data)
                    .unwrap()
                    .styles_string
                    .get(),
            );
            insta::assert_debug_snapshot!("it_works-root_e", &root_e);
            insta::assert_debug_snapshot!("it_works-e1", &e1);
            insta::assert_debug_snapshot!("it_works-e2", &e2);
            info!("..=========================================================");
        }
    }

    #[test]
    fn change_parent() {
        let _xx = setup_global_subscriber();
        {
            let _g = span!(Level::TRACE, "change_parent").entered();

            let e_dict_sv:StateVar<GraphEdgesDict<&str>> = use_state(Dict::new());

            let root_e_source =use_state( None);
            let root_e_target = use_state(Some(node_index("root")));
            let root_e = EmgEdgeItem::default_with_wh_in_topo(root_e_source.watch(), root_e_target.watch(),e_dict_sv.watch(),100, 100);
            e_dict_sv.set_with(|d|{
                let mut nd = d .clone();
                nd.insert(EdgeIndex::new(None,Some(node_index("root"))), Edge::new(root_e_source, root_e_target, root_e.clone()));
                nd
            });


            let s_root_e2_source =use_state( None);
            let root_e2_target = use_state(Some(node_index("root2")));
            let root_e2 = EmgEdgeItem::default_with_wh_in_topo(s_root_e2_source.watch(), root_e2_target.watch(),e_dict_sv.watch(),200, 200);
            e_dict_sv.set_with(|d|{
                let mut nd = d .clone();
                nd.insert(EdgeIndex::new(None,Some(node_index("root2"))), Edge::new(s_root_e2_source, root_e2_target, root_e2.clone()));
                nd
            });
            // ---------------------------------------

            let e1_source =use_state( Some(node_index("root")));
            let e1_target = use_state(Some(node_index("1")));
            let e1 = EmgEdgeItem::new_in_topo(
                    e1_source.watch(),
                    e1_target.watch(),
                e_dict_sv.watch(),
                (px(10).into(), px(10).into()),
                 (pc(0).into(), pc(0).into(), pc(0).into()), 
                 (pc(50).into(), pc(50).into(), pc(50).into()),
            );
            e_dict_sv.set_with(|d|{
                let mut nd = d .clone();
                nd.insert(edge_index("root","1"), Edge::new(e1_source, e1_target, e1.clone()));
                nd
            });

            //-------------------------------------
            
            let e2_source =use_state( Some(node_index("1")));
            let e2_target = use_state(Some(node_index("2")));
            let e2 = EmgEdgeItem::new_in_topo(
                e2_source.watch(),
                    e2_target.watch(),
                  e_dict_sv.watch(),
                     (px(10).into(), px(10).into()), 
                     (pc(0).into(), pc(0).into(), pc(0).into()), 
                     (pc(100).into(), pc(000).into(), pc(000).into()),
            );
            e_dict_sv.set_with(|d|{
                let mut nd = d .clone();
                nd.insert(edge_index("1","2"), Edge::new(e2_source, e2_target, e2.clone()));
                nd
            });
          

            trace!("---e1 {}", &e1); 

            assert_eq!(
                e1.edge_data(&EPath(vector![edge_index_no_source("root"),edge_index("root", "1")]))
                    .unwrap()
                    .calculated
                    .coordinates_trans
                    .get(),
                Translation3::<f64>::new(50.0, 50.0, 0.)
            );

            e1_source
                .set(Some(node_index("root2")));
            trace!("---new root2:e1 {}", &e1); 

            assert_eq!(
                e1.edge_data(&EPath(vector![edge_index_no_source("root2"),edge_index("root2", "1")]))
                    .unwrap()
                    .calculated
                    .coordinates_trans
                    .get(),
                Translation3::<f64>::new(100., 100., 0.)
            );
            info!("..=========================================================");
            trace!("new root_e2:e1 {}", &e1);
            info!("--------------------------------------------------");
            trace!("new root_e2:e2 {}", &e2);
            info!("..=========================================================");
            //local
            assert_eq!(e2.id.get(), edge_index("1", "2"));
            assert_eq!(
                e2.edge_data(&EPath(vector![
                    edge_index_no_source( "root2"),
                    edge_index("root2", "1"),
                    edge_index("1", "2"),
                ]))
                .unwrap()
                .calculated
                .coordinates_trans
                .get(),
                Translation3::<f64>::new(10.0, 00.0, 0.)
            );

            // e2.id
            //     .set_with(|id| id.clone().use_incoming(node_index("root")));
            // parent_for_e2_sa.set(root_e.clone());
            // local use root
            e2_source.set(Some(node_index("root")));
            assert_eq!(e2.id.get(), edge_index("root", "2"));
            assert_eq!(
                e2.edge_data(&EPath(vector![
                    edge_index_no_source( "root"),
                    edge_index("root", "2"),
                ]))
                    .unwrap()
                    .calculated
                    .coordinates_trans
                    .get(),
                Translation3::<f64>::new(100.0, 0.0, 0.)
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
