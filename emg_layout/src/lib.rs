#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::non_ascii_literal)]
#![allow(clippy::used_underscore_binding)]//for display attr

// ────────────────────────────────────────────────────────────────────────────────
#![feature(iter_intersperse)]
#![feature(min_specialization)]

// // 固定 type  T:xxx
// #![feature(trivial_bounds)]
// #![feature(negative_impls)]
// #![feature(auto_traits)]

use std::{cell::Ref, clone::Clone, cmp::{Eq, Ord}, hash::Hash, rc::Rc};

use calc::layout_calculating;
use derive_more::Display;
use derive_more::From;
use derive_more::Into;
use either::Either;
// use derive_more::TryInto;
use emg::{Edge, EdgeIndex, NodeIndex, };
use emg_refresh::RefreshFor;
use emg_state::{Anchor, CloneStateAnchor, CloneStateVar, Dict, GStateStore, StateAnchor, StateMultiAnchor, StateVar, topo, use_state};

use im::Vector;
use na::{Affine3, Isometry3, Matrix4, Point3, Rotation3, SVector, Similarity3, Translation3, Vector2, Vector3};
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
use tracing::{span, trace_span,error,instrument, trace, Level};
// ────────────────────────────────────────────────────────────────────────────────

mod calc;
mod impl_refresh;
pub mod add_values;


// ────────────────────────────────────────────────────────────────────────────────

type Size2 = SVector<f64,2>;
type Vec3 = SVector<f64,3>;
type Trans3 = Translation3<f64>;
type Rot3 = Rotation3<f64>;
type Transform9 = Affine3<f64>;
type Pos3 = Point3<f64>;

#[derive(Display, Debug, PartialEq, PartialOrd, Copy, Clone, From, Into)]
struct Mat4(Matrix4<f64>);

// type Mat4 = Matrix4<f64>;


#[derive(Display, Clone, Debug, From, PartialEq, PartialOrd, Eq)]
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

impl GenericSize {
    #[must_use]
    pub fn get_length_value(&self) ->  f64 {
        (
            self
                .try_get_length_value()
                .expect("directly get length value failed, expected Length Px struct"),
        )
    }
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

#[derive(Debug,Display,Clone,PartialEq,Eq)]
enum EitherDyn<T>{
    Const(T),
    DynA(StateAnchor<T>)
}

impl<T> From<StateAnchor<T>> for EitherDyn<T> {
    fn from(v: StateAnchor<T>) -> Self {
        Self::DynA(v)
    }
}

impl<T> From<T> for EitherDyn<T> {
    fn from(v: T) -> Self {
        Self::Const(v)
    }
}

impl<T> From <CssWidth> for EitherDyn<T>{
    fn from(v: CssWidth) -> Self {
        T::from(v).into()
    }
}
impl<T> From <CssHeight> for EitherDyn<T>{
    fn from(v: CssHeight) -> Self {
        T::from(v).into()
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
impl Default for GenericWH {
    fn default() -> Self {
        Self {
            w: px(0).into(),
            h: px(0).into(),
        }
    }
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
    w:StateVar<EitherDyn<GenericSize>>,
    h:StateVar<EitherDyn<GenericSize>>,
    z:StateVar<EitherDyn<u64>>,
    origin_x: StateVar<EitherDyn<GenericSize>>,
    origin_y: StateVar<EitherDyn<GenericSize>>,
    origin_z: StateVar<EitherDyn<GenericSize>>,
    align_x: StateVar<EitherDyn<GenericSize>>,
    align_y: StateVar<EitherDyn<GenericSize>>,
    align_z: StateVar<EitherDyn<GenericSize>>,
    // origin: StateVar<GenericLoc>,
    // align: StateVar<GenericLoc>,
    path_styles: StateVar<Dict<EPath<Ix>, Style>>,
}

impl<Ix> Layout<Ix>
where
    Ix: Clone + Hash + Eq + Default + PartialOrd + std::cmp::Ord + 'static,
{
    /// Set the layout's size.
    #[cfg(test)]
    fn set_size(&self,
         w: impl Into< EitherDyn<GenericSize>>,
        h: impl Into< EitherDyn<GenericSize>>,) {
            self.w.set(w.into());
            self.h.set(h.into());
    }
    pub fn store_set_size(&self,store: &GStateStore,
        w: impl Into< EitherDyn<GenericSize>>,
        h: impl Into< EitherDyn<GenericSize>>,) {
        self.store_set_w(store,w);
        self.store_set_h(store,h);
    }

    pub fn store_set_w(&self, store: &GStateStore,w:impl Into< EitherDyn<GenericSize>>) {
        self.w.store_set(store, w.into());

    }
    pub fn store_set_h(&self, store: &GStateStore,h:impl Into< EitherDyn<GenericSize>>) {
        self.h.store_set(store, h.into());

    }
}
impl<Ix> Copy for Layout<Ix> where
    Ix: Clone + Hash + Eq + Default + PartialOrd + std::cmp::Ord + 'static
{
}

impl<Ix> std::fmt::Display for Layout<Ix>
where
    Ix: Clone + Hash + Eq + Default + Ord + 'static + std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let x = format!(
            "size:{{\nw:{},h:{};\n}}\norigin:{{\nx:{},y:{},z:{};\n}}\nalign:{{\nx:{},y:{},z:{};\npath_styles:{{\n{};\n}}",
            indented(&self.w),
            indented(&self.h),
            indented(&self.origin_x),
            indented(&self.origin_y),
            indented(&self.origin_z),
            indented(&self.align_x),
            indented(&self.align_y),
            indented(&self.align_z),
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

#[derive(Debug, Clone, PartialEq)]
pub struct EdgeData {
    calculated: LayoutCalculated,
    styles_string: StateAnchor<String>, // matrix: M4Data,
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

impl<Ix: Clone + Hash + Eq + PartialEq + Default> EPath<Ix> {
    #[must_use] pub fn new(vec:Vector<EdgeIndex<Ix>>)->Self{
        Self(vec)
    }

    #[must_use] pub fn back(&self)->Option< &EdgeIndex<Ix>>{
        self.0.back()
    }

    #[must_use] pub fn get(&self)->&Vector<EdgeIndex<Ix>>{
        &self.0
    }

    pub fn get_mut(&mut self)-> &mut Vector<EdgeIndex<Ix>>{
        &mut self.0
    }

    pub fn set(&mut self,vec:Vector<EdgeIndex<Ix>>){
        self.0 = vec; 
    }

    pub fn set_with<T:FnMut(&mut Vector<EdgeIndex<Ix>>)>(&mut self,mut func:T){
         func( &mut self.0);
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
#[derive(Clone, Debug, PartialEq)]
pub struct EmgEdgeItem<Ix>
where
    Ix: Clone + Hash + Eq + Ord + 'static + Default,
{
    pub id:StateVar< StateAnchor<EdgeIndex<Ix>>>,// dyn by Edge(source_nix , target_nix)
    pub paths:DictPathEiNodeSA<Ix>, // with parent self
    pub layout: Layout<Ix>,
    pub other_styles: StateVar<Style>,
    // no self  first try
    pub node:DictPathEiNodeSA<Ix>, //TODO with self?  not with self?
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
            "id:{{\n{};\n}}\npaths:{{\n{};\n}}\nlayout:{{\n{};\n}}\nother_styles:{{\n{};\n}}\nnode:{{\n{};\n}}",
            indented(&self.id),
            indented(DictDisplay(self.paths.get())),
            indented(&self.layout),
            indented(&self.other_styles),
            indented(DictDisplay(self.node.get()))
        );
        write!(f, "EdgeDataWithParent {{\n{}\n}}", indented(&x))
    }
}
type DictPathEiNodeSA<Ix> = StateAnchor<Dict<EPath<Ix>, EdgeItemNode>>;


impl<Ix> EmgEdgeItem<Ix>
where
    Ix: Clone + Hash + Ord + Default 
{
    #[cfg(test)]
    fn set_size(&self,w: impl Into< EitherDyn<GenericSize>>,
        h: impl Into< EitherDyn<GenericSize>>,){
        self.layout.set_size(w,h)
    }
    pub fn store_set_size(&self,store: &GStateStore,  
        w: impl Into< EitherDyn<GenericSize>>,
        h: impl Into< EitherDyn<GenericSize>>,){
        self.layout.store_set_size(store,w,h)
    }
    
    #[cfg(test)]
    #[must_use]
    fn edge_data(&self, key: &EPath<Ix>) -> Option<EdgeData> {
        self.node
            .get()
            .get(key)
            .and_then(EdgeItemNode::as_edge_data).cloned()
    }

    #[must_use]
    pub fn store_edge_data(&self,store:&GStateStore, key: &EPath<Ix>) -> Option<EdgeData> {
        self.node.store_get(store)
            .get(key)
            .and_then(EdgeItemNode::as_edge_data).cloned()
            
    }
   
}

impl<Ix> EmgEdgeItem<Ix>
where
    Ix: Clone + Hash + Eq + PartialEq + PartialOrd + Ord + Default + std::fmt::Display,
{
  

    #[topo::nested]
    #[instrument(skip(edges))]
    pub fn default_in_topo(
        source_node_nix_sa: StateAnchor<Option<NodeIndex<Ix>>>,
        target_node_nix_sa: StateAnchor<Option<NodeIndex<Ix>>>,
        edges: StateAnchor<GraphEdgesDict<Ix>>,
         ) -> Self  where Ix:std::fmt::Debug{

        Self::new_in_topo(source_node_nix_sa, target_node_nix_sa, edges,    GenericSize::default(), GenericSize::default(), GenericSize::default(),GenericSize::default(),GenericSize::default(),GenericSize::default(),GenericSize::default(),GenericSize::default(),)

         }

    #[topo::nested]
    #[instrument(skip(edges))]
    pub fn default_with_wh_in_topo<T: Into<f64> + std::fmt::Debug>(
        source_node_nix_sa: StateAnchor<Option<NodeIndex<Ix>>>,
        target_node_nix_sa: StateAnchor<Option<NodeIndex<Ix>>>,
        edges: StateAnchor<GraphEdgesDict<Ix>>,
         w: T, h: T) -> Self  where Ix:std::fmt::Debug{

        Self::new_in_topo(source_node_nix_sa, target_node_nix_sa, edges,    size(px(w)), size(px(h)), GenericSize::default(),GenericSize::default(),GenericSize::default(),GenericSize::default(),GenericSize::default(),GenericSize::default())
       
    }
    


#[topo::nested]
    pub fn new_in_topo(
        source_node_nix_sa: StateAnchor<Option<NodeIndex<Ix>>>,
        target_node_nix_sa: StateAnchor<Option<NodeIndex<Ix>>>,
        edges: StateAnchor<GraphEdgesDict<Ix>>,
        w:  impl Into< EitherDyn<GenericSize>>,
        h:  impl Into< EitherDyn<GenericSize>>,
        origin_x: impl Into< EitherDyn<GenericSize>>,
        origin_y: impl Into< EitherDyn<GenericSize>>,
        origin_z: impl Into< EitherDyn<GenericSize>>,
        align_x:  impl Into< EitherDyn<GenericSize>>,
        align_y:  impl Into< EitherDyn<GenericSize>>,
        align_z:  impl Into< EitherDyn<GenericSize>>,
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

        let layout = Layout::<Ix> {
            w: use_state(w.into()),
            h: use_state(h.into()),
            z: use_state(0.into()),
            origin_x: use_state(origin_x.into()),
            origin_y: use_state(origin_y.into()),
            origin_z: use_state(origin_z.into()),
            align_x: use_state(align_x.into()),
            align_y: use_state(align_y.into()),
            align_z: use_state(align_z.into()),
            path_styles: use_state(Dict::unit(EPath::<Ix>::default(), s())),
        };

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
                        
                       
                    let (layout_calculated,styles_string) =  match path_edge_item_node {
                        EdgeItemNode::Empty => path_ein_empty_node_builder(&layout, other_styles_sv),
                        EdgeItemNode::EdgeData(ped)=> path_with_ed_node_builder(id_sv, ped, &layout, path, other_styles_sv),
                        EdgeItemNode::String(_)  => {
                            todo!("parent is EdgeItemNode::String(_) not implemented yet");
                        }
                                
                    };
                    EdgeItemNode::EdgeData(EdgeData {
                        calculated: layout_calculated,
                        styles_string,
                    })
                }).into()

            }).into()
        });
            

        Self {
            id: id_sv,
            paths,
            layout,
            other_styles: other_styles_sv,
            node,
        }
    }
}


fn path_with_ed_node_builder<Ix>(
    id_sv: StateVar<StateAnchor<EdgeIndex<Ix>>>, 
    ped: &EdgeData,
     layout: &Layout<Ix>,
      path: &EPath<Ix>, 
      other_styles_sv: StateVar<Style>) -> (LayoutCalculated, StateAnchor<String>) 
where
Ix: std::clone::Clone + std::hash::Hash + std::default::Default + std::cmp::Ord +std::fmt::Display
{
    let layout_calculated = layout_calculating(id_sv, ped, layout);
    let p = path.clone();
    let this_path_style_string_sa: StateAnchor<Option<String>> = layout
                        .path_styles
                        .watch()
                        .map(move |d: &Dict<EPath<Ix>, Style>| {
                            let _g = trace_span!( "[  this_path_style_string_sa recalculation ]:layout.path_styles change ").entered();
    
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
    (layout_calculated,styles_string)
}

fn path_ein_empty_node_builder<Ix>(layout: &Layout<Ix>, other_styles_sv: StateVar<Style>) -> (LayoutCalculated, StateAnchor<String>)
 where 
    Ix: std::clone::Clone + std::hash::Hash + std::default::Default + std::cmp::Ord 
    {
        
    let calculated_size = (&layout.w.watch(),&layout.h.watch()).then(|w: &EitherDyn<GenericSize>,h: &EitherDyn<GenericSize>| {
            // println!("in layout size watch map");
            match (w,h){
                (EitherDyn::Const(_), EitherDyn::Const(_)) => {
                    (&layout.w.watch(),&layout.h.watch()).map(|w,h|{
                        Size2::new(w.get_length_value(), h.get_length_value())
                    }).into()
                }
                (EitherDyn::Const(w), EitherDyn::DynA(sa_h)) => {
                    sa_h.map(|h: &GenericSize|{
                        Size2::new(w.get_length_value(),h.get_length_value())
                    }).into()
                }
                (EitherDyn::DynA(sa_w), EitherDyn::Const(h)) => {
                    sa_w.map(|w:&GenericSize|{
                        Size2::new(w.get_length_value(), h.get_length_value())
                    }).into()
                }
                (EitherDyn::DynA(sa_w), EitherDyn::DynA(sa_h)) => {
                    (sa_w,sa_h).map(|w,h|{
                        Size2::new(w.get_length_value(), h.get_length_value())
                    }).into()
                }
            }
        });
    let calculated_origin = StateAnchor::constant(Trans3::identity());
    let calculated_align = StateAnchor::constant(Trans3::identity());
    let coordinates_trans = StateAnchor::constant(Trans3::identity());
    let matrix = coordinates_trans.map(|x| x.to_homogeneous().into());
    let loc_styles = (&calculated_size, &matrix).map(move |size: &Size2, mat4: &Mat4| {
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
            &layout.path_styles.watch(),
            &layout_calculated.loc_styles,
            &other_styles_sv.watch(),
        )
        .map(
            move |path_styles: &Dict<EPath<Ix>,Style>, loc_styles: &Style, other_styles: &Style| {
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
    (layout_calculated,styles_string)
}



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

pub fn size(v: impl Into<GenericSize>) -> EitherDyn<GenericSize> {
    v.into().into()
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

#[cfg(test)]
mod tests {
    #![allow(clippy::too_many_lines)]
    use crate::*;

    use emg::{edge_index, edge_index_no_source, node_index};
    use emg_refresh::RefreshUseFor;
    use emg_state::StateVar;
    use im::vector;
    use seed_styles::CssWidth;
    use styles::{CssBackgroundColorTrait, h, hsl, pc, width};
    use tracing::{info, span};

    use tracing_flame::FlameLayer;
    use tracing_subscriber::{fmt, prelude::*, EnvFilter};

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
            let e_dict_sv:StateVar<GraphEdgesDict<&str>> = use_state(Dict::new());


            let root_e_source =use_state( None);
            let root_e_target = use_state(Some(node_index("root")));
            let mut root_e = EmgEdgeItem::default_with_wh_in_topo(root_e_source.watch(), root_e_target.watch(),e_dict_sv.watch(),1920, 1080);
            e_dict_sv.set_with(|d|{
                let mut nd = d .clone();
                nd.insert(EdgeIndex::new(None,node_index("root")), Edge::new(root_e_source, root_e_target, root_e.clone()));
                nd
            });
                

            let e1_source =use_state( Some(node_index("root")));
            let e1_target = use_state(Some(node_index("1")));
            let e1 = EmgEdgeItem::new_in_topo(
                    e1_source.watch(),
                    e1_target.watch(),
                e_dict_sv.watch(),
                size(px(50), px(50)),
                origin2(pc(0), pc(0)),
                align2(pc(50), pc(50)),
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
                size(px(10), px(10)),
                origin2(pc(100), pc(100)),
                align2(pc(100), pc(100)),
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

            root_e.refresh_use(&vec![css(css_width)]);
            // root_e.refresh_use(&css(css_width.clone()));
            root_e.refresh_use(&Css(css_height));
            assert_eq!(
                e1.edge_data(&EPath(vector![edge_index_no_source("root"), edge_index("root", "1")]))
                    .unwrap()
                    .calculated
                    .coordinates_trans
                    .get(),
                Trans3::new(50., 50., 0.)
            );
            info!("=========================================================");

            e2.refresh_use(&Css(CssWidth::from(px(20))));
            e2.refresh_use(&Css(CssHeight::from(px(20))));

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
                Trans3::new(30., 30., 0.)
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
            info!("..=========================================================");
        }
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

            // let cc = Transform9::identity();
            let c = width(pc(11));
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
                size(px(10), px(10)),
                origin2(pc(100), pc(100)),
                align2(pc(50), pc(20)),
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
            size(px(10), px(10)),
                origin2(pc(100), pc(100)),
                align2(pc(50), pc(20)),
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
                Trans3::new(950.0, 206.0, 0.)
            );


            let xx = vec![css_width];
            // let xx = vec![css(css_width)];

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

            root_e.refresh_use(&Css(css_height.clone()));
            let tempcss= use_state(css_height);
            root_e.refresh_use(&tempcss);
            assert_eq!(
                e1.node
                    .get()
                    .get(&EPath(vector![edge_index_no_source("root"), edge_index("root", "1")]))
                    .and_then(EdgeItemNode::as_edge_data)
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
                    .get(&EPath(vector![edge_index_no_source("root"), edge_index("root", "1")]))
                    .and_then(EdgeItemNode::as_edge_data)
                    .unwrap()
                    .calculated
                    .size
                    .get(),
                Size2::new(12., 10.)
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
            e2.refresh_use(&Css(CssHeight::from(px(50))));
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
                Trans3::new(-4.0, -48.0, 0.0)
            );
            e2.set_size(GenericWH::new(px(100), px(100)));
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
                    Size2::new(100.0, 100.0)
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
                size(px(10), px(10)),
                origin2(pc(0), pc(0)),
                align2(pc(50), pc(50)),
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
                     size(px(10), px(10)),
                origin2(pc(0), pc(0)),
                align2(pc(100), pc(000)),
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
                Trans3::new(50.0, 50.0, 0.)
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
                Trans3::new(100.0, 100.0, 0.)
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
                Trans3::new(10.0, 00.0, 0.)
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
                Trans3::new(100.0, 0.0, 0.)
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
