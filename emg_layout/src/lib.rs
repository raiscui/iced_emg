#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::non_ascii_literal)]
#![allow(clippy::used_underscore_binding)]//for display attr

// ────────────────────────────────────────────────────────────────────────────────
#![feature(specialization)]
// #![feature(min_specialization)]
#![feature(auto_traits)]
#![feature(negative_impls)]
#![feature(iter_intersperse)]
// ────────────────────────────────────────────────────────────────────────────────
#![feature(test)]
// ────────────────────────────────────────────────────────────────────────────────



// // 固定 type  T:xxx
// #![feature(trivial_bounds)]
// #![feature(negative_impls)]
// #![feature(auto_traits)]
use std::{cell::RefCell, clone::Clone, cmp::{Eq, Ord}, collections::HashMap, hash::{BuildHasherDefault, Hash}, rc::Rc, time::Duration};
use cassowary::{Variable, Constraint, Expression, WeightedRelation, Solver};
use ccsa::{CassowaryMap, CCSS, CCSSEqExpression, CCSSSvvOpSvvExpr, NameChars, ScopeViewVariable, CCSSOpSvv, PredOp, PredEq, StrengthAndWeight};
use emg_hasher::CustomHasher;

use calc::layout_calculating;
use derive_more::Display;
use derive_more::From;
use derive_more::Into;
// use derive_more::TryInto;
use emg_core::{GenericSize, im::{OrdSet, ordmap::{NodeDiffItem, self}, self}, IdStr, NotNan, vector};
use emg::{Edge, EdgeIndex, NodeIndex, };
use emg_refresh::RefreshFor;
use emg_state::{Anchor, CloneStateAnchor, CloneStateVar, Dict, GStateStore, StateAnchor, StateMultiAnchor, StateVar, state_store, topo, use_state, use_state_impl::Engine};
use emg_core::Vector;
use na::{Affine3, Isometry3, Matrix4, Point3, Rotation3, Similarity3, Translation3,Vector2, Vector3};
use nalgebra as na;
pub use seed_styles as styles;
use styles::{ CssTransform, CssValueTrait, Style, UpdateStyle, px, s};
// use styles::Percent;
// use styles::ExactLength;
// use styles::CssWidth;
// use styles::CssHeight;
use styles::{CssHeightTrait, CssTransformTrait, CssWidthTrait};
//
// ────────────────────────────────────────────────────────────────────────────────

use indented::indented;
use tracing::{span, trace_span,error,instrument, trace, Level, warn, debug, info, debug_span, warn_span};
// ────────────────────────────────────────────────────────────────────────────────

mod calc;
mod impl_refresh;
pub mod animation;
pub mod add_values;
pub use animation::AnimationE;

use crate::ccsa::{CCSSVecDisp, PredVariable};

pub mod old;
pub mod ccsa;


// ────────────────────────────────────────────────────────────────────────────────



thread_local! {
    static G_CLOCK: StateVar<Duration> = use_state(Duration::ZERO);
}

thread_local! {
    static G_ANIMA_RUNNING_STORE: StateVar<Vector<Anchor<bool>>> = use_state(Vector::new());
}
thread_local! {
    static G_AM_RUNING: StateAnchor<bool> = global_anima_running_build();
}
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


#[derive(Debug, Clone, PartialEq, Eq)]
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
    cassowary_constants:StateVar<StateAnchor<Vector<CCSS>>>,
    cassowary_selectors:StateVar<Vector<ScopeViewVariable>>,
}

impl Layout
{
    fn get(&self,prop:&str)->StateVar<GenericSizeAnchor>{
        match prop {
            "width" => self.w.clone(),
            "height" => self.h.clone(),
            "origin_x" => self.origin_x.clone(),
            "origin_y" => self.origin_y.clone(),
            "origin_z" => self.origin_z.clone(),
            "align_x" => self.align_x.clone(),
            "align_y" => self.align_y.clone(),
            "align_z" => self.align_z.clone(),
            _ => panic!("unknown prop {}",prop),
        }

    }
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
    fmt = "{{\nsuggest size:\n{},\nreal size:\n{},\norigin:\n{},\nalign:\n{},\ncoordinates_trans:\n{},\ncass_trans:\n{},\nmatrix:\n{},\nloc_styles:\n{},\n}}",
    "indented(suggest_size)",
    "indented(real_size)",
    "indented(origin)",
    "indented(align)",
    "indented(coordinates_trans)",
    "indented(cass_trans)",
    "indented(matrix)",
    "indented(loc_styles)"
)]
pub struct LayoutCalculated {
    
    suggest_size: StateAnchor<Vector2<f64>>,
    size_constraints: StateAnchor<Vec<Constraint>>,
    real_size: StateAnchor<Vector2<f64>>,
    origin: StateAnchor<Translation3<f64>>,
    align: StateAnchor<Translation3<f64>>,
    coordinates_trans: StateAnchor<Translation3<f64>>,
    cass_trans: StateAnchor<Translation3<f64>>,
    matrix: StateAnchor<Mat4>,
    loc_styles: StateAnchor<Style>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EdgeData {
    path_layout: StateAnchor<Layout>,
    calculated: LayoutCalculated,
    cassowary_map:Rc<CassowaryMap>,
    cassowary_calculated_vars:StateAnchor<Dict<Variable, (NotNan<f64>,IdStr)>>,
    cassowary_calculated_layout:StateAnchor<(f64,f64)>,
    pub styles_string: StateAnchor<String>, 
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

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Hash, Debug, Default)]
// pub struct EPath<Ix: Clone + Hash + Eq + PartialEq + Default>(TinyVec<[EdgeIndex<Ix>;2]>);
//TODO  loop check
pub struct EPath<Ix: Clone + Hash + Eq + PartialEq + Default>(Vector<EdgeIndex<Ix>>);

impl<Ix: Clone + Hash + Eq + PartialEq + Default> std::ops::Deref for EPath<Ix> {
    type Target = Vector<EdgeIndex<Ix>>;
    // type Target = TinyVec<[EdgeIndex<Ix>;2]>;

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
    #[must_use] 
    pub const fn new(vec:Vector<EdgeIndex<Ix>>)->Self{
        Self(vec)
    }

    pub fn last_target(&self) -> Option<&NodeIndex<Ix>> {
         self.0.last().and_then(|e| e.target_nix().as_ref())
    }
    pub fn except_tail_match(&self,other_no_tail:&EPath<Ix>) -> bool {
        if self.0.len() -1 != other_no_tail.0.len() {
            return false;
        }
        for i in 0..self.0.len()-1 {
            if self.0[i] != other_no_tail.0[i] {
                return false;
            }
        }
        true

    }

    #[must_use]
        pub fn link_ref(&self, target_nix:NodeIndex<Ix>)-> Self {
        let last = self.last().and_then(|e|e.target_nix().as_ref()).cloned();
        let mut new_e = self.clone();
        new_e.push_back(EdgeIndex::new(last,target_nix));
        new_e
        
    }
    #[must_use]
    pub fn link(mut self, target_nix:NodeIndex<Ix>)-> Self {
        let last = self.last().and_then(|e|e.target_nix().as_ref()).cloned();
        self.push_back(EdgeIndex::new(last,target_nix));
        self
        
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
// use rustc_hash::FxHasher as CustomHasher;

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
    pub paths:DictPathEiNodeSA<Ix>, // with parent self  // current not has current node
    pub layout: Layout,
    path_styles: StateVar<PathVarMap<Ix, Style>>, //TODO check use
    path_layouts:StateVar<PathVarMap<Ix, Layout>>,// layout only for one path 

    pub other_styles: StateVar<Style>,
    // no self  first try
    pub edge_nodes:DictPathEiNodeSA<Ix>, //TODO with self?  not with self?  (current with self)
    store:Rc<RefCell<GStateStore>>
}
impl<Ix> Eq for EmgEdgeItem<Ix>
where 
Ix: Clone + Hash + Eq + Default + PartialOrd + std::cmp::Ord + 'static,
{}


impl<Ix> PartialEq for EmgEdgeItem<Ix> 
where 
Ix: Clone + Hash + Eq + Default + PartialOrd + std::cmp::Ord + 'static,

{
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.paths == other.paths && self.layout == other.layout && self.path_styles == other.path_styles
        && self.other_styles == other.other_styles && self.edge_nodes == other.edge_nodes
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
            indented(DictDisplay(self.edge_nodes.get())),
        );
        write!(f, "EdgeDataWithParent {{\n{}\n}}", indented(&x))
    }
}
pub type DictPathEiNodeSA<Ix> = StateAnchor<Dict<EPath<Ix>, EdgeItemNode>>; //NOTE: EdgeData or something


impl<Ix> EmgEdgeItem<Ix>
where
    Ix: Clone + Hash + Ord + Default + std::fmt::Debug 
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
      
        //TODO not get(), use ref
        self.edge_nodes
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
        #[cfg(debug_assertions)]
        {
            let oo = self.edge_nodes.store_get_with(store,|o|{
                o.clone()
            });
            trace!("edge_nodes: {:#?}",&oo);
        }
   

        self.edge_nodes.store_get_with(store,|o|{
           func(o
            .get(key)
            .and_then(EdgeItemNode::as_edge_data) )
        })
       
            
    }
    pub fn engine_edge_data_with<F: FnOnce(Option<&EdgeData>)->R,R>(&self,engine:&mut Engine, key: &EPath<Ix>,func:F) -> R {
        self.edge_nodes.engine_get_with(engine,|o|{
           func(o
            .get(key)
            .and_then(EdgeItemNode::as_edge_data) )
        })
       
            
    }
   
}

impl<Ix> EmgEdgeItem<Ix>
where
    Ix: Clone + Hash + Eq + PartialEq + PartialOrd + Ord + Default + std::fmt::Display + std::borrow::Borrow<str>,
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
    where
     Ix:std::fmt::Debug,
    (IdStr, NotNan<f64>):PartialEq

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
            cassowary_constants: use_state(StateAnchor::constant(vector![])),
            cassowary_selectors: use_state(vector![]),
            

        };
        // let path_styles= use_state(Dict::unit(EPath::<Ix>::default(), s()));
        let path_styles:StateVar<PathVarMap<Ix, Style>> = use_state(PathVarMap::default());
        let path_layouts:StateVar<PathVarMap<Ix, Layout>> = use_state(PathVarMap::default());
      

        let other_styles_sv = use_state(s());

        let opt_self_source_node_nix_sa_re_get:StateAnchor<Option<NodeIndex<Ix>>> = id_sv.watch().then(|eid_sa_inner|{
            let _g = trace_span!( "[ source_node_nix_sa_re_get recalculation ]:id_sv change ").entered();

            eid_sa_inner.map(|i:&EdgeIndex<Ix>|{
                
                let _g = span!(Level::TRACE, "[ source_node_nix_sa_re_get recalculation ]:eid_sa_inner change ",edge_index=%i).entered();
                
                i.source_nix().clone()
            }).into()
        });

        let opt_self_target_node_nix_sa_re_get:StateAnchor<Option<NodeIndex<Ix>>> = id_sv.watch().then(|eid_sa_inner|{

            eid_sa_inner.map(|i:&EdgeIndex<Ix>|{
                
                i.target_nix().clone()
            }).into()
        });

        let edges2 = edges.clone();
      


        let parent_paths: DictPathEiNodeSA<Ix> = 
            opt_self_source_node_nix_sa_re_get.then(move|opt_self_source_nix:&Option<NodeIndex<Ix>>| {

                let _g = span!(Level::TRACE, "[ source_node_incoming_edge_dict_sa recalculation ]:source_node_nix_sa_re_get change ").entered();

                if opt_self_source_nix.is_none(){
                    //NOTE 如果 source nix  是没有 node index 那么他就是无上一级的
                    Anchor::constant(Dict::<EPath<Ix>, EdgeItemNode>::unit(EPath::<Ix>::default(), EdgeItemNode::Empty))
                    //TODO check why use unit? answer:need for `EdgeItemNode::Empty => path_ein_empty_node_builder`
                    // Anchor::constant(Dict::<EPath<Ix>, EdgeItemNode>::new())
                }else{
                    let opt_some_source_nix_clone = opt_self_source_nix.clone();
                    edges.filter_map(move|someone_eix, e| {
                        
                        println!("********************** \n one_eix.target_node_ix: {:?} ?? opt_source_nix_clone:{:?}",someone_eix.target_nix(),&opt_some_source_nix_clone);
                        if   someone_eix.target_nix() == &opt_some_source_nix_clone {

                            Some(e.item.edge_nodes.clone())

                        }else{
                            None
                        }
                        
                    })
                    .anchor()
                    .then(|x:&Dict<EdgeIndex<Ix>, DictPathEiNodeSA<Ix>>|{
                        x.values().map(emg_state::StateAnchor::anchor)
                        .collect::<Anchor<Vector<_>>>()
                        .map(|v:&Vector<_>|{
                            let _g = trace_span!( "[  paths dict recalculation ]:vector paths change ").entered();
                            Dict::unions(v.clone())})
                    })
                }
                       

                
            });



        // NOTE children cassowary_map
        let children_nodes = opt_self_target_node_nix_sa_re_get.then(move|opt_self_target_nix|{
            if opt_self_target_nix.is_none() {
                //NOTE 尾
                    Anchor::constant(Dict::<EPath<Ix>, EdgeItemNode>::default())
            }else{

                // TODO  try  use node outgoing  find which is good speed? maybe make loop, because node in map/then will calculating
                // TODO ? let e = edges2.map(|e|e.get(Edge::default()));
                let opt_self_target_nix2 =opt_self_target_nix.clone();
                edges2.filter_map(move |child_eix,v|{
                    //NOTE  edge source is self_target, this is children
                    if child_eix.source_nix() == &opt_self_target_nix2 {
                        Some(v.edge_nodes.clone())
                    }else{
                        None
                    }
                }).anchor()
                .then(|x:&Dict<EdgeIndex<Ix>, DictPathEiNodeSA<Ix>>|{
                    x.values().map(emg_state::StateAnchor::anchor)
                    .collect::<Anchor<Vector<_>>>()
                    .map(|v:&Vector<_>|{
                        Dict::unions(v.clone())})
                })
            }
        });
        // ─────────────────────────────────────────────────────────────────

       

        //TODO not paths: StateVar<Dict<EPath<Ix>,EdgeItemNode>>  use edgeIndex instead to Reduce memory
        let paths_clone = parent_paths.clone();
        let nodes:DictPathEiNodeSA<Ix> = id_sv.watch().then(move|id_sa|{
            
            let paths_clone2 = paths_clone.clone();
            let path_layouts2 = path_layouts.clone(); 
            let children_nodes2 = children_nodes.clone();
                
            id_sa.then(move |eid:&EdgeIndex<Ix>|{

            let children_nodes3 = children_nodes2.clone();

                let eid_clone = eid.clone();
                
                paths_clone2.map(move |p_node_as_paths:&Dict<EPath<Ix>, EdgeItemNode>|{
                    
                    p_node_as_paths.iter()
                        .map(|(parent_e_path, p_ei_node_v)| {
                            let mut p_ep_add_self = parent_e_path.clone();
                            
                            //TODO node 可以自带 self nix ,下游不必每个子节点都重算

                            p_ep_add_self.0.push_back(eid_clone.clone());
                            (p_ep_add_self, p_ei_node_v.clone())
                        })
                        .collect::<Dict<EPath<Ix>, EdgeItemNode>>()

                }) .map_( move |self_path:&EPath<Ix>, p_path_edge_item_node:&EdgeItemNode| {

                    //@ current var=>ix ix=>var cassowary_map
                    let current_cassowary_map = Rc::new(CassowaryMap::new());

                    
                    let self_path2 =self_path.clone();
                    let self_path3 =self_path.clone();
                    let self_path4 =self_path.clone();

                    let _child_span =
                        span!(Level::TRACE, "[ node recalculation ]:paths change ").entered();

                        //TODO use Dict anchor Collection
                    let path_layout:StateAnchor<Layout> = path_layouts2.watch().map(move|path_layouts_map:&PathVarMap<Ix, Layout>|{
                        // println!("--> id: {:?}", &id_sv);
                        trace!("--> finding path_layout in path_with_ed_node_builder------------------- len:{}",path_layouts_map.len());
                        // println!("--> layout:{:?}",pls_map.get(&path_clone));
                        *path_layouts_map.get(&self_path2).unwrap_or(&layout)
                    });
                        
                    let (opt_p_calculated,layout_calculated,layout_styles_string) =  match p_path_edge_item_node {
                        //NOTE 上一级节点: empty => 此节点是root
                        EdgeItemNode::Empty => path_ein_empty_node_builder(&path_layout, self_path,&current_cassowary_map,path_styles, other_styles_sv),
                        EdgeItemNode::EdgeData(ped)=> path_with_ed_node_builder(id_sv, ped,&current_cassowary_map, &path_layout, self_path, path_styles, other_styles_sv),
                        EdgeItemNode::String(_)  => {
                            todo!("parent is EdgeItemNode::String(_) not implemented yet");
                        }
                                
                    };

                    //NOTE children cassowary_map
                    let children_cass_maps_sa = children_nodes3.filter_map(move |child_path,child_node|{
                        if child_path.except_tail_match(&self_path3) {
                            match (child_path.last_target(),child_node.as_edge_data()){
                               
                                // (Some(nix), Some(ed)) =>Some( (nix.index().clone(),(ed.cassowary_map.clone(),ed.path_layout.clone()))),
                                (Some(nix), Some(ed)) =>Some( (nix.index().clone(),(ed.cassowary_map.clone(),ed.calculated.suggest_size.clone(),ed.calculated.size_constraints.clone()))),
                                _=>None
                            }
                        }else{
                            None
                        }
                    })
                    .map(|x|{
                        x.values().cloned()
                        .collect::<Dict<Ix, (Rc<CassowaryMap>,StateAnchor<Vector2<f64>>,StateAnchor<Vec<Constraint>>)>>()
                    });
                    



                 

                    //NOTE 约束
                    let ccss_list_sa = layout.cassowary_constants.watch().then(|x|x.clone().into_anchor());
                    //TODO 不要每一次变更 ccss_list ,都全部重新计算 
                    let constant_sets_sa = (&ccss_list_sa,&children_cass_maps_sa).then(move | ccss_list,children_cass_maps|{

                            let _debug_span_ = warn_span!( "->[ constant_sets_sa calc then ] ").entered();


                            let (constant_sets,prop_suggestions,constraints_sa) = ccss_list.iter()
                            .fold((OrdSet::<Constraint>::new(),Dict::<Variable, StateAnchor<Option<f64>>>::new(),Vector::<Anchor<Vec<Constraint>>>::new()), 
                            |( mut constraint_sets,mut prop_suggestions0,mut constraints_sa0),CCSS{ svv_op_svvs,  eq_exprs, opt_sw }|{

                                //TODO use left_prop_gss if loop when use child:layout_calculated
                                if let Some((left_expr,left_prop_directly_layout_val,left_all_consensus_constraints,left_all_consensus_constraints_sa)) = svv_op_svvs_to_expr(svv_op_svvs,children_cass_maps){
                                    
                                    prop_suggestions0 = prop_suggestions0.union(left_prop_directly_layout_val);
                                    constraint_sets.extend(left_all_consensus_constraints);
                                    constraints_sa0.append(left_all_consensus_constraints_sa);

                                    let (constants,_,prop_suggestions2,constraints_sa2) = eq_exprs.into_iter().fold((constraint_sets,left_expr,prop_suggestions0,constraints_sa0), |(mut constraints,left_expr, prop_suggestions1,mut constraints_sa1),CCSSEqExpression{ eq, expr }|{


                                        if let Some((right_expr,right_prop_directly_layout_val,right_all_consensus_constraints,right_all_consensus_constraints_sa)) = svv_op_svvs_to_expr(expr,children_cass_maps){

                                            let constraint = left_expr | eq_opt_sw_to_weighted_relation(eq,opt_sw)| right_expr.clone();

                                            constraints.insert(constraint);
                                            constraints.extend(right_all_consensus_constraints);
                                            constraints_sa1.extend(right_all_consensus_constraints_sa);

                                            (constraints,right_expr,prop_suggestions1.union(right_prop_directly_layout_val),constraints_sa1)

                                        }else{

                                            (constraints,left_expr, prop_suggestions1,constraints_sa1)

                                        }

                                    });
                                    (constants,prop_suggestions2,constraints_sa2)
                                }else{
                                    (constraint_sets,prop_suggestions0,constraints_sa0)
                                }

                            });
                            warn!("[constant_sets_sa] ccss_list:\n{}", CCSSVecDisp(ccss_list.clone()));

                            constraints_sa.into_iter().collect::<Anchor<Vector<Vec<Constraint>>>>()
                            .map(move|c_vector|{
                                let mut x = constant_sets.clone();
                                x.extend(c_vector.clone().into_iter().flatten());
                                x
                            })


                            // // //todo use add_edit_variable suggest_value
                            // let prop_suggestions_anchor = prop_suggestions.into_iter().map(|(var,sa_v)|{
                                
                            //     sa_v.map(move |v|{
                            //         v.as_ref().map(|vv|{
                            //             var | WeightedRelation::EQ(cassowary::strength::WEAK*100.0)| *vv
                            //         })
                                    
                            //     }
                        
                            // ).into_anchor()}).collect::<Anchor<OrdSet<Option<Constraint>>>>().map(|o|o.clone().into_iter().filter_map(|o|o).collect::<OrdSet<Constraint>>());

                            // prop_suggestions_anchor.map(move |prop_suggestions|{
                            //     constant_sets.clone().union(prop_suggestions.clone())
                            // })
                            
                        
                    });

                    let LayoutCalculated{real_size, origin, align, coordinates_trans ,..} = &layout_calculated;
                    //NOTE 层建议值 (层当前计算所得)
                    let current_calculated_prop_val_sa = ( real_size, origin, align ).map(|size, origin, align|{
                        let width = size.x;
                        let height = size.y;
                        let origin_x = origin.x;
                        let origin_y = origin.y;
                        let align_x = align.x;
                        let align_y = align.y;

                        //TODO real val because cassowary calc need suggestions???
                        //TODO bottom right use from bottom or from top??
                        let top  =  0f64;
                        let bottom = height;
                        let left = 0f64;
                        let right = width;

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
let children_cass_maps_no_val_sa = children_cass_maps_sa.map_(|_ix,(map,..)|{
    map.clone()
});
let current_cassowary_map3 = current_cassowary_map.clone();

let children_for_current_constants_sa =  children_cass_maps_no_val_sa.map(move |cass_maps|{
    
    let (ws,hs) = cass_maps.values().fold( (vec![],vec![]),|(mut ws,mut hs),map|{
        let w = map.var("right");
        let h = map.var("bottom");
        ws.push(w);
        hs.push(h);
        (ws,hs)
    });
    let mut  res_exprs = OrdSet::new();
    // for (opt_r,opt_b) in ws.into_iter().zip(hs.into_iter()) {
    //     if let Some(r) = opt_r{
    //         res_exprs.insert(
    //             current_cassowary_map3.var("width").unwrap() | WeightedRelation::GE(cassowary::strength::WEAK) | r
    //         );
    //     }
    //     if let Some(b) = opt_b{
  
    //         res_exprs.insert(
    //             current_cassowary_map3.var("height").unwrap() | WeightedRelation::GE(cassowary::strength::WEAK) | b

    //         );
    //     }
      
    // }

    res_exprs
     

});
// ────────────────────────────────────────────────────────────────────────────────

                    let mut last_observation_constants:OrdSet<Constraint>  =  OrdSet::new();
                    let mut last_observation_current_props:Dict<IdStr, NotNan<f64>> =  Dict::new();
                    let mut last_observation_children_for_current_constants :OrdSet<Constraint>  =  OrdSet::new();
                    let current_cassowary_map2 = current_cassowary_map.clone();
                    let mut cass_solver = Solver::new();
                 
                    cass_solver.add_constraints(&[
                        current_cassowary_map.var("bottom").unwrap() | WeightedRelation::EQ(cassowary::strength::REQUIRED) | current_cassowary_map.var("top").unwrap() + current_cassowary_map.var("height").unwrap(),
                        current_cassowary_map.var("right").unwrap() | WeightedRelation::EQ(cassowary::strength::REQUIRED) | current_cassowary_map.var("left").unwrap()+ current_cassowary_map.var("width").unwrap(),
                        current_cassowary_map.var("bottom").unwrap() | WeightedRelation::GE(cassowary::strength::REQUIRED) | current_cassowary_map.var("top").unwrap(),
                        current_cassowary_map.var("right").unwrap() | WeightedRelation::GE(cassowary::strength::REQUIRED) | current_cassowary_map.var("left").unwrap(),
                        current_cassowary_map.var("width").unwrap() | WeightedRelation::GE(cassowary::strength::REQUIRED) | 0.0,
                        current_cassowary_map.var("height").unwrap() | WeightedRelation::GE(cassowary::strength::REQUIRED) | 0.0,
                    ]).unwrap();


                    let calculated_changed_vars_sa  = (&children_for_current_constants_sa,&constant_sets_sa,&current_calculated_prop_val_sa).map_mut( Dict::<Variable,NotNan<f64>>::new(),move |out,children_for_current_constants,newest_constants,newest_current_prop_vals| {
                        let _debug_span_ = warn_span!( "->[ calculated_changed_vars_sa calc map_mut ] ").entered();
                        warn!("[calculated_changed_vars_sa] newest_current_prop_vals :{:?}",&newest_current_prop_vals);

                        let mut children_for_current_constants_did_update = false;


                        if children_for_current_constants.len() == 0 && last_observation_children_for_current_constants.len() != 0{
                            for constant in last_observation_children_for_current_constants.iter(){
                                cass_solver.remove_constraint(constant).unwrap();

                            }
                            last_observation_children_for_current_constants.clear();
                            children_for_current_constants_did_update=true;
                        }else{
                            for diff_item in last_observation_children_for_current_constants.diff(children_for_current_constants){
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
                            last_observation_children_for_current_constants = children_for_current_constants.clone();

                        }

                        let mut constants_did_update = false;
                        let mut prop_vals_did_update = false;

                        if newest_constants.len() == 0 && last_observation_constants.len() != 0 {
                            for constant in last_observation_constants.iter() {
                                cass_solver.remove_constraint(constant).unwrap();
                            }
                            last_observation_constants.clear();
                            // cass_solver.reset();
                            constants_did_update = true;
                        }else{
                            for diff_item in last_observation_constants.diff(newest_constants){
                                match diff_item {
                                    NodeDiffItem::Add(x) => {
                                        cass_solver.add_constraint(x.clone()).unwrap();
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
                            last_observation_constants = newest_constants.clone();

                        }
                        

                        // ────────────────────────────────────────────────────────────────────────────────
                        let current_calculated_prop_sw_mul = 1.0f64;
                            
                        info!("current_cassowary_map2===== \n all= \n{:?}",&current_cassowary_map2.map);

                        for diff_item in last_observation_current_props.diff(newest_current_prop_vals){
                            info!("current_cassowary_map2 \n all= \n{:?}",&current_cassowary_map2.map);

                            match diff_item {
                                ordmap::DiffItem::Add(prop, v) => {
                                    info!("current props  add (maybe first time)");
                                    // panic!("current_cassowary_map2:want:{:?} \n all= \n{:?}",&prop,&current_cassowary_map2.map);
                                    let var = current_cassowary_map2.var(&**prop).unwrap();
                                    cass_solver.add_edit_variable(var, cassowary::strength::MEDIUM * current_calculated_prop_sw_mul).ok();
                                    cass_solver.suggest_value(var, *v).unwrap();
                                    prop_vals_did_update = true;

                                },
                                ordmap::DiffItem::Update { old:(old_prop,_old_v), new:(prop,v) } => {
                                    //TODO check, remove .
                                    assert_eq!(old_prop,prop);
                                    let var = current_cassowary_map2.var(&**prop).unwrap();
                                    cass_solver.add_edit_variable(var, cassowary::strength::MEDIUM * current_calculated_prop_sw_mul).ok();
                                    cass_solver.suggest_value(var, *v).unwrap();
                                    prop_vals_did_update = true;

                                },
                                ordmap::DiffItem::Remove(_, _) => {
                                    panic!("current props never remove (current now)")
                                },
                            };

                        };
                        last_observation_current_props = newest_current_prop_vals.clone();

                        // ────────────────────────────────────────────────────────────────────────────────
                        if constants_did_update || prop_vals_did_update || children_for_current_constants_did_update{
                            let changes = cass_solver.fetch_changes();
                            warn!("cass solver change:{:#?}",&changes);
                            if changes.len() > 0 {
                                *out =  changes.into();
                                return true
                            }
                        }

                        false

                    });
                    // ────────────────────────────────────────────────────────────────────────────────
                    let current_cassowary_map3 = current_cassowary_map.clone();
                    let cassowary_calculated_vars =  (&children_cass_maps_sa,&calculated_changed_vars_sa).map_mut(Dict::<Variable, (NotNan<f64>,IdStr)>::new(),move|out,children_cass_maps,changed_vars|{
                        let _debug_span_ = warn_span!( "->[ calculated_vars calc map_mut ] ").entered();

                        if !changed_vars.is_empty() {
                            // warn!("[calculated_vars] changed_vars======== \n{:?}",&changed_vars);

                            //TODO remove if release
                            for (var,v) in changed_vars.iter() {
                                let id_prop_str =   children_cass_maps.iter().find_map(|(id,(cassowary_map ,_directly_layout,_constraints_sa))|{
                                     cassowary_map.prop(&var).map(|prop|{
                                        let vv:IdStr = format!("{} |=> #{}[{}]",&self_path4, &id,&prop).into(); 
                                        vv
                                     })
                                }).or_else(||{
                                    current_cassowary_map3.prop(&var).map(|prop|{
                                        let vv:IdStr = format!("{}[{}] ",&self_path4,&prop).into();
                                        vv
                                    })
                                }).unwrap_or_default();

                                warn!("[calculated_vars] changed  prop:{:?}  v:{}",&id_prop_str,&v);

                                
                                out.insert(*var,(*v,id_prop_str));


                            }
                            warn!("[calculated_vars] total  prop:\n{:#?} ",&out);

                            return true
                        }
                        false
                        
                    });



                    //TODO check diff with [calculated], because calculated_vars may re suggestion some value.
                    //TODO replace current_cassowary_map4.var("width") use   width
                    let current_cassowary_map4 = current_cassowary_map.clone();
                    let cassowary_calculated_layout = cassowary_calculated_vars.map(move |cassowary_vars|{
                        warn!("[calculated_vars] [calculated_cassowary_layout] total  prop:\n{:#?} ",&cassowary_vars);


                        let w  =cassowary_vars.get(&current_cassowary_map4.var("width").unwrap()).unwrap().0.into_inner();
                        let h  =cassowary_vars.get(&current_cassowary_map4.var("height").unwrap()).unwrap().0.into_inner();
                        (w,h)
                    });
                    let styles_string:StateAnchor<String> = (&layout_styles_string, &cassowary_calculated_layout).map(move |layout_styles,(w,h)|{
                        
                        format!(
                            "{} {}",
                            layout_styles,
                            s().w(px(*w)).h(px(*h)).render()
                        )

                    });

// ────────────────────────────────────────────────────────────────────────────────



                    EdgeItemNode::EdgeData(Box::new(EdgeData {
                        path_layout,
                        calculated: layout_calculated,
                        cassowary_map: current_cassowary_map,
                        cassowary_calculated_vars,
                        cassowary_calculated_layout,
                        styles_string,
                        opt_p_calculated,
                    }))
                }).into()

            }).into()
        });
            

        Self {
            id: id_sv,
            paths:parent_paths,
            layout,
            
            path_styles,
            path_layouts,
            other_styles: other_styles_sv,
            edge_nodes: nodes,
            store:state_store()
        }
    }
}

fn eq_opt_sw_to_weighted_relation(
    eq: &PredEq,
    opt_sw:& Option<StrengthAndWeight>,
) -> WeightedRelation {
    let weight = opt_sw.as_ref().map_or(cassowary::strength::MEDIUM, |sw| sw.to_number());
    match eq {
        PredEq::Eq => WeightedRelation::EQ(weight),
        PredEq::Lt => todo!(),
        PredEq::Le => WeightedRelation::LE(weight),
        PredEq::Ge => WeightedRelation::GE(weight),
        PredEq::Gt => todo!(),
    }
}


#[instrument(skip(children_cass_maps))]
fn svv_op_svvs_to_expr<Ix>(svv_op_svvs:&CCSSSvvOpSvvExpr,children_cass_maps:&Dict<Ix, (Rc<CassowaryMap>,StateAnchor<Vector2<f64>>,StateAnchor<Vec<Constraint>>)>) ->Option<(Expression,Dict<Variable, StateAnchor<Option<f64>>>,OrdSet<Constraint>, Vector<Anchor<Vec<Constraint>>>) >
where
Ix:std::fmt::Debug+ Clone + Hash + Eq + PartialEq + PartialOrd + Ord + Default + std::fmt::Display + std::borrow::Borrow<str>,
{
    let CCSSSvvOpSvvExpr{  svv:main_svv, op_exprs }=svv_op_svvs;
    svv_to_var(main_svv,children_cass_maps).map(|(first_var,first_prop_layout_directly_val_sa,consensus_constraints,consensus_constraints_sa)  |{
        let mut prop_directly_layout_vals = Dict::unit(first_var, first_prop_layout_directly_val_sa);
        let mut all_consensus_constraints : OrdSet<Constraint> = consensus_constraints.into();
        let mut all_consensus_constraints_sa : Vector<Anchor<Vec<Constraint>>> = vector![consensus_constraints_sa];

        add_suggestions_props(main_svv,children_cass_maps,&mut prop_directly_layout_vals,&mut all_consensus_constraints);

        let expr =op_exprs.into_iter().fold(first_var.into(), | exp:Expression,op_expr| {
            let CCSSOpSvv{ op, svv } = op_expr;
            match op{
                PredOp::Add => {
                    if let Some((var,prop_layout_directly_val_sa,consensus_constraints,consensus_constraints_sa,))     = svv_to_var(svv,children_cass_maps){
                        prop_directly_layout_vals.insert(var, prop_layout_directly_val_sa);
                        all_consensus_constraints.extend(consensus_constraints);
                        all_consensus_constraints_sa.push_back(consensus_constraints_sa);

                        add_suggestions_props(svv,children_cass_maps,&mut prop_directly_layout_vals,&mut all_consensus_constraints);

                        exp + var
                    }else{
                        exp
                    }
                 
                
                },
                PredOp::Sub => todo!(),
                PredOp::Mul => todo!(),
            }
       
        });
        (expr,prop_directly_layout_vals,all_consensus_constraints,all_consensus_constraints_sa)
    })
    
}

//NOTE add width height 常识规则
fn add_suggestions_props<Ix>(svv: &ScopeViewVariable,children_cass_maps:&Dict<Ix, (Rc<CassowaryMap>,StateAnchor<Vector2<f64>>,StateAnchor<Vec<Constraint>>)>,prop_directly_layout_vals:&mut Dict<Variable, StateAnchor<Option<f64>>>,all_consensus_constraints :&mut OrdSet<Constraint>) 
where
Ix:std::fmt::Debug+ Clone + Hash + Eq + PartialEq + PartialOrd + Ord + Default + std::fmt::Display + std::borrow::Borrow<str>,
{
    if let Some((var,prop_layout_directly_val_sa,consensus_constraints,..)) = svv_to_var(&svv.turn_with_var("width"),children_cass_maps){
        prop_directly_layout_vals.insert(var,prop_layout_directly_val_sa);
        all_consensus_constraints.extend(consensus_constraints);
    }else{
        panic!("svv_op_svvs:turn_with_var(width) not found");
    }
    if let Some((var,prop_layout_directly_val_sa,consensus_constraints,..)) = svv_to_var(&svv.turn_with_var("height"),children_cass_maps){
        prop_directly_layout_vals.insert(var,prop_layout_directly_val_sa);
        all_consensus_constraints.extend(consensus_constraints);
    }else{
        panic!("svv_op_svvs:turn_with_var(height) not found");
    }
}

#[instrument(skip(children_cass_maps))]
fn svv_to_var<Ix>(scope_view_variable:&ScopeViewVariable,children_cass_maps: &Dict<Ix, (Rc<CassowaryMap>,StateAnchor<Vector2<f64>>,StateAnchor<Vec<Constraint>>)>) -> Option<(Variable, StateAnchor<Option<f64>>,Vec<Constraint>,Anchor<Vec<Constraint>>) >
where
Ix:std::fmt::Debug+ Clone + Hash + Eq + PartialEq + PartialOrd + Ord + Default + std::fmt::Display + std::borrow::Borrow<str>,
{

    
    let ScopeViewVariable{ scope, view, variable } = scope_view_variable;
    let var = match (scope, view, variable) {
        (None, None, None) => todo!(),
        (None, None, Some(_)) => todo!(),
        (None, Some(_), None) => todo!(),
        (None, Some(name), Some(PredVariable(prop))) => {

            match name {
                NameChars::Id(id) => {
                    let _debug_span_ = debug_span!( "->[ get child variable ] ").entered();

                    warn!("[svv_to_var] parsed scope_view_variable,  find child var : child id:{:?} prop:{:?}",&id,&prop);

                    children_cass_maps.get(id.as_str()).map(|(cass_map,directly_layout_val,size_constraints)|{
                            warn!("[svv_to_var] got child id:{:?} cass_map: {:?}", &id,&cass_map);



                            //TODO smallvec
                            

                            let constraints = vec![
                                        cass_map.var("bottom").unwrap() | WeightedRelation::EQ(cassowary::strength::REQUIRED) | cass_map.var("top").unwrap() + cass_map.var("height").unwrap(),
                                        cass_map.var("right").unwrap() | WeightedRelation::EQ(cassowary::strength::REQUIRED) | cass_map.var("left").unwrap()+ cass_map.var("width").unwrap(),
                                        cass_map.var("bottom").unwrap() | WeightedRelation::GE(cassowary::strength::REQUIRED) | cass_map.var("top").unwrap(),
                                        cass_map.var("right").unwrap() | WeightedRelation::GE(cassowary::strength::REQUIRED) | cass_map.var("left").unwrap(),
                                        cass_map.var("width").unwrap() | WeightedRelation::GE(cassowary::strength::REQUIRED) | 0.0,
                                        cass_map.var("height").unwrap() | WeightedRelation::GE(cassowary::strength::REQUIRED) | 0.0,
                                        cass_map.var("top").unwrap() | WeightedRelation::GE(cassowary::strength::WEAK) | 0.0,
                                        cass_map.var("left").unwrap() | WeightedRelation::GE(cassowary::strength::WEAK) | 0.0,


                            ];
                            // size_constraints.map(|cs|constraints.extend(cs));

                            let prop_id_str2 = prop.clone();
                            let prop_layout_val_sa = directly_layout_val.map(move |l|{
                                match prop_id_str2.as_str(){
                                    "width" => Some(l.x),
                                    "height" => Some(l.y),
                                    _ => {
                                        warn!("other directly_layout val ...  not implemented :{}",&prop_id_str2);
                                        None
                                    },
                                }
                            });

                            let var = cass_map.var(prop).unwrap();

                            (var,prop_layout_val_sa,constraints,size_constraints.get_anchor())
                    })
                

                },
                NameChars::Class(_) => todo!(),
                NameChars::Element(_) => todo!(),
                NameChars::Virtual(_) => todo!(),
                NameChars::Number(_) => todo!(),
                NameChars::Next(_) => todo!(),
                NameChars::Last(_) => todo!(),
                NameChars::First(_) => todo!(),
            }

        },
        (Some(_), None, None) => todo!(),
        (Some(_), None, Some(_)) => todo!(),
        (Some(_), Some(_), None) => todo!(),
        (Some(_), Some(_), Some(_)) => todo!(),
    };
    var
}


fn path_with_ed_node_builder<Ix>(
    id_sv: StateVar<StateAnchor<EdgeIndex<Ix>>>, 
    ped: &EdgeData,
    current_cassowary_map:&Rc<CassowaryMap>,
    path_layout: &StateAnchor<Layout>,
    path: &EPath<Ix>, 
    path_styles: StateVar<PathVarMap<Ix, Style>>,
    other_styles_sv: StateVar<Style>) -> (Option<LayoutCalculated>, LayoutCalculated, StateAnchor<String>) 
where
Ix: std::clone::Clone + std::hash::Hash + std::default::Default + std::cmp::Ord +std::fmt::Display+'static+ std::fmt::Debug
{
    // println!("run path_with_ed_node_builder ******************************************************************");


    let p_calculated = ped.calculated.clone();
    let path_clone2 = path.clone();
    
    
    let layout_calculated = layout_calculating(id_sv, ped,current_cassowary_map, path_layout);
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
    path_layout: &StateAnchor<Layout>,
    path: &EPath<Ix>, 
    current_cassowary_map:&Rc<CassowaryMap>,

    path_styles:StateVar<PathVarMap<Ix, Style>>, other_styles_sv: StateVar<Style>) -> (Option<LayoutCalculated>,LayoutCalculated, StateAnchor<String>)
 where 
    Ix: std::clone::Clone + std::hash::Hash + std::default::Default + std::cmp::Ord 
    {
        // println!("run path_ein_empty_node_builder ******************************************************************");

        // ─────────────────────────────────────────────────────────────────
        let path_clone = path.clone();
        
        let w = path_layout.then(|l:&Layout|l.w.watch().into());
        let h = path_layout.then(|l:&Layout|l.h.watch().into());
        // let origin_x = path_layout.then(|l:&Layout|l.origin_x.watch().into());
        // let origin_y = path_layout.then(|l:&Layout|l.origin_y.watch().into());
        // let align_x = path_layout.then(|l:&Layout|l.align_x.watch().into());
        // let align_y = path_layout.then(|l:&Layout|l.align_y.watch().into());
        // ─────────────────────────────────────────────────────────────────
        let sa_w = w.then(|w|w.get_anchor());
        let sa_h = h.then(|h|h.get_anchor());
        let width_var  =current_cassowary_map.var("width").unwrap();
        let height_var  =current_cassowary_map.var("height").unwrap();
        let current_cassowary_map2 = current_cassowary_map.clone();

        let size_constraints = 
                (&sa_w,&sa_h).map(move |w:&GenericSize,h:&GenericSize|{
                    let size_constraints = vec![
                        width_var | WeightedRelation::EQ(cassowary::strength::REQUIRED) | w.get_length_value(),
                        height_var | WeightedRelation::EQ(cassowary::strength::REQUIRED) |h.get_length_value(),
                        // • • • • •

                                        current_cassowary_map2.var("bottom").unwrap() | WeightedRelation::EQ(cassowary::strength::REQUIRED) | current_cassowary_map2.var("top").unwrap() + height_var,
                                        current_cassowary_map2.var("right").unwrap() | WeightedRelation::EQ(cassowary::strength::REQUIRED) | current_cassowary_map2.var("left").unwrap()+ width_var,
                                        current_cassowary_map2.var("bottom").unwrap() | WeightedRelation::GE(cassowary::strength::REQUIRED) | current_cassowary_map2.var("top").unwrap(),
                                        current_cassowary_map2.var("right").unwrap() | WeightedRelation::GE(cassowary::strength::REQUIRED) | current_cassowary_map2.var("left").unwrap(),
                                        width_var | WeightedRelation::GE(cassowary::strength::REQUIRED) | 0.0,
                                        height_var | WeightedRelation::GE(cassowary::strength::REQUIRED) | 0.0,
                                        current_cassowary_map2.var("top").unwrap() | WeightedRelation::GE(cassowary::strength::WEAK) | 0.0,
                                        current_cassowary_map2.var("left").unwrap() | WeightedRelation::GE(cassowary::strength::WEAK) | 0.0,
                    ];
                    size_constraints
                });

        //TODO 如果没有parent 那么 parent 就是 screen w h
    let calculated_size:StateAnchor<Vector2<f64>> = (&w,&h).then(|sa_w: &GenericSizeAnchor,sa_h: &GenericSizeAnchor| {
            (&**sa_w,&**sa_h).map(|w:&GenericSize,h:&GenericSize|->Vector2<f64>{
                //TODO check editor display error 
                Vector2::<f64>::new(w.get_length_value(), h.get_length_value())
            }).into()    
            
        });

    let real_size:StateAnchor<Vector2<f64>> = (&w,&h).then(|sa_w: &GenericSizeAnchor,sa_h: &GenericSizeAnchor| {
        (&**sa_w,&**sa_h).map(|w:&GenericSize,h:&GenericSize|->Vector2<f64>{
            //TODO check editor display error 
            Vector2::<f64>::new(w.get_length_value(), h.get_length_value())
        }).into()    
        
    });

        //TODO 审视是否要自定义定位
    let calculated_origin = StateAnchor::constant(Translation3::<f64>::identity());
    let calculated_align = StateAnchor::constant(Translation3::<f64>::identity());
    let coordinates_trans = StateAnchor::constant(Translation3::<f64>::identity());
    let cass_trans = StateAnchor::constant(Translation3::<f64>::identity());
    let matrix = cass_trans.map(|x| x.to_homogeneous().into());
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
            suggest_size: calculated_size,
            size_constraints,
            real_size,
            origin: calculated_origin,
            align: calculated_align,
            coordinates_trans,
            cass_trans,
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

                //NOTE fold because edge no in , path_styles only one values.
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
    EdgeData(Box<EdgeData>),
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
    use emg_core::{parent, IdStr};
    use emg_refresh::RefreshForUse;
    use emg_state::StateVar;
    use emg_core::vector;
 
    use styles::{CssWidth, CssHeight,bg_color, h, hsl, pc, width, CssBackgroundColor};
    use tracing::{debug, info, span, warn};

    use tracing_flame::FlameLayer;
    use tracing_subscriber::{fmt, prelude::*, EnvFilter};
    use test::{black_box, Bencher};

    fn setup_global_subscriber() -> impl Drop {
        std::env::set_var("RUST_LOG", "trace");
        // std::env::set_var("RUST_LOG", "warn");
        std::env::set_var("RUST_LOG", "info");

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
            let e_dict_sv:StateVar<GraphEdgesDict<IdStr>> = use_state(Dict::new());


            let root_e_source =use_state( None);
            let root_e_target = use_state(Some(node_index("root")));
            let mut root_e = EmgEdgeItem::<IdStr>::default_with_wh_in_topo(root_e_source.watch(), root_e_target.watch(),e_dict_sv.watch(),1920, 1080);
            e_dict_sv.set_with(|d|{
                let mut nd = d .clone();
                nd.insert(EdgeIndex::new(None,node_index("root")), Edge::new(root_e_source, root_e_target, root_e.clone()));
                nd
            });
                

            let e1_source =use_state( Some(node_index("root")));
            let e1_target = use_state(Some(node_index("1")));
            let e1 = EmgEdgeItem::<IdStr>::new_in_topo(
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
            let mut e2 = EmgEdgeItem::<IdStr>::new_in_topo(
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
            e2.refresh_for_use(&Css(bg_color(hsl(40,70,30))));

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

            let e_dict_sv:StateVar<GraphEdgesDict<IdStr>> = use_state(Dict::new());

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
                e1.edge_nodes
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
                e1.edge_nodes
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
                e1.edge_nodes
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
                e1.edge_nodes
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
                e1.edge_nodes
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
                e1.edge_nodes.get().get(&EPath(vector![edge_index_no_source("root"), edge_index("root", "1")]))
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
                Translation3::<f64>::new(-4.0, -48.0, 0.0)
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
                e2.edge_nodes
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
    fn  test_edge(){
        let f = width(parent!(CssHeight)+ pc(100));
        println!("{}", f);
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

            let e_dict_sv:StateVar<GraphEdgesDict<IdStr>> = use_state(Dict::new());

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
            warn!("{}",e1.edge_nodes
            .get()
            .get(&EPath(vector![edge_index_no_source("root"),edge_index("root","1")]))
            .and_then(EdgeItemNode::as_edge_data)
            .unwrap()
            .calculated);
            assert_eq!(
                e1.edge_nodes
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
            warn!("{}",e1.edge_nodes
            .get()
            .get(&EPath(vector![edge_index_no_source("root"),edge_index("root","1")]))
            .and_then(EdgeItemNode::as_edge_data)
            .unwrap()
            .calculated);

            let xx = vec![css_width];
            // let xx = vec![css(css_width)];

            root_e.refresh_for_use(&xx);

            warn!("calculated 3 =========================================================");
            warn!("{}",e1.edge_nodes
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
            warn!("{}",e1.edge_nodes
            .get()
            .get(&EPath(vector![edge_index_no_source("root"),edge_index("root","1")]))
            .and_then(EdgeItemNode::as_edge_data)
            .unwrap()
            .calculated);

            assert_eq!(
                e1.edge_nodes
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
            let _ff =  e1.edge_nodes
            .get()
            .get(&EPath(vector![edge_index_no_source("root"), edge_index("root", "1")]))
            .and_then(EdgeItemNode::as_edge_data)
                    .unwrap()
                    .styles_string
                    .get();
            assert_ne!(
                e1.edge_nodes
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
                e1.edge_nodes
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
                e1.edge_nodes
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
                e1.edge_nodes.get().get(&EPath(vector![edge_index_no_source("root"), edge_index("root", "1")]))
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
                Translation3::<f64>::new(-4.0, -10.0, 0.0)
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
                e2.edge_nodes
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

            let e_dict_sv:StateVar<GraphEdgesDict<IdStr>> = use_state(Dict::new());

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
            info!("================= e_dict_sv: {:#?}",&e_dict_sv);

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
