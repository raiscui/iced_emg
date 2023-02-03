/*
 * @Author: Rais
 * @Date: 2022-06-23 22:52:57
 * @LastEditTime: 2023-02-03 18:09:01
 * @LastEditors: Rais
 * @Description:
 */

use std::{hash::BuildHasherDefault, rc::Rc};

use cassowary::{
    strength::{REQUIRED, WEAK},
    Constraint, Variable, WeightedRelation,
};

use emg_common::{im::HashMap, IdStr, NotNan};
use emg_hasher::CustomHasher;
use emg_state::Dict;

use indexmap::IndexMap;
use parse_display::Display;

mod impl_refresh;
mod ops;
pub mod svv_process;

#[derive(Debug, Clone, Display, PartialEq, Eq)]
pub enum NameCharsOrNumber<Ix = IdStr> {
    #[display("#{0}")]
    Id(Ix), // #xxx
    #[display(".{0}")]
    Class(Ix), // .xxx
    #[display("{0}")]
    Element(Ix), // xxxx
    #[display("\"{0}\"")]
    Virtual(Ix), //"xxx"
    #[display("{0}")]
    Number(NotNan<f64>), // 12 | 12.1
    #[display("{0}:next")]
    Next(Box<Self>), // TODO do parse
    #[display("{0}:last")]
    Last(Box<Self>),
    #[display("{0}:first")]
    First(Box<Self>),
}
#[derive(Debug, Copy, Clone, Display, PartialEq, Eq)]
pub enum PredEq {
    #[display(" == ")]
    Eq,
    #[display(" < ")]
    Lt,
    #[display(" <= ")]
    Le,
    #[display(" >= ")]
    Ge,
    #[display(" > ")]
    Gt,
}

// ────────────────────────────────────────────────────────────────────────────────
// ────────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Copy, Clone, Display, PartialEq, Eq)]
pub enum Scope {
    #[display("&")]
    Local,
    //
    #[display("^({0})")]
    Parent(u8),
    //
    #[display("$")]
    Global,
}
#[derive(Debug, Clone, Display, PartialEq, Eq)]
#[display("[{0}]")]
pub struct PredVariable(pub IdStr);

impl std::ops::Deref for PredVariable {
    type Target = IdStr;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ScopeViewVariable {
    pub scope: Option<Scope>, // eg. & or # or .

    pub view: Option<NameCharsOrNumber>, // eg. idxx  or  1 / 2/ 3/ 4...
    pub variable: Option<PredVariable>,  //  eg. [width]
}

impl std::fmt::Display for ScopeViewVariable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let scope = self
            .scope
            .as_ref()
            .map_or_else(String::new, |x| format!("{x}"));
        let view = self.view.as_ref().map_or(String::new(), |x| format!("{x}"));
        let variable = self
            .variable
            .as_ref()
            .map_or_else(String::new, |x| format!("{x}"));

        write!(f, "{scope}{view}{variable}")
    }
}

impl ScopeViewVariable {
    #[must_use]
    pub const fn new(
        scope: Option<Scope>,
        view: Option<NameCharsOrNumber>,
        variable: Option<PredVariable>,
    ) -> Self {
        Self {
            scope,
            view,
            variable,
        }
    }
    // #[must_use]
    // pub fn turn_with_var(&self, var: &str) -> Self {
    //     let Self {
    //         scope,
    //         view,
    //         variable,
    //     } = self;
    //     Self::new(*scope, view.clone(), Some(PredVariable(var.into())))
    // }
    /// # Panics
    ///
    /// Will panic if number is not nan
    #[must_use]
    pub fn new_number(number: f64) -> Self {
        Self::new(
            None,
            Some(NameCharsOrNumber::Number(NotNan::new(number).unwrap())),
            None,
        )
    }
    #[must_use]
    pub fn new_id_var(id: &str, var: &str) -> Self {
        Self::new(
            None,
            Some(NameCharsOrNumber::Id(id.into())),
            Some(PredVariable(var.into())),
        )
    }
}

#[derive(Debug, Clone, Display, PartialEq, Eq)]
pub enum PredOp {
    #[display("+")]
    Add,
    #[display("-")]
    Sub,
    #[display("*")]
    Mul,
}

#[derive(Debug, Clone, Display, PartialEq, Eq)]
#[display(" {op} {svv}")]
pub struct CCSSOpSvv {
    pub op: PredOp,
    pub svv: ScopeViewVariable,
}
impl CCSSOpSvv {
    #[must_use]
    pub const fn new(op: PredOp, svv: ScopeViewVariable) -> Self {
        Self { op, svv }
    }

    #[must_use]
    pub const fn new_add(svv: ScopeViewVariable) -> Self {
        Self {
            op: PredOp::Add,
            svv,
        }
    }
    #[must_use]
    pub const fn new_sub(svv: ScopeViewVariable) -> Self {
        Self {
            op: PredOp::Sub,
            svv,
        }
    }
    #[must_use]
    pub const fn new_mul(svv: ScopeViewVariable) -> Self {
        Self {
            op: PredOp::Mul,
            svv,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CCSSSvvOpSvvExpr {
    pub svv: ScopeViewVariable,
    pub op_exprs: Vec<CCSSOpSvv>,
}

impl CCSSSvvOpSvvExpr {
    #[must_use]
    pub fn new(svv: ScopeViewVariable, op_exprs: Vec<CCSSOpSvv>) -> Self {
        Self { svv, op_exprs }
    }
}

impl std::fmt::Display for CCSSSvvOpSvvExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.svv)?;
        for op_expr in &self.op_exprs {
            write!(f, "{op_expr}")?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Display, PartialEq, Eq)]
#[display("{eq} {expr}")]
pub struct CCSSEqExpression {
    pub eq: PredEq,
    pub expr: CCSSSvvOpSvvExpr,
}

impl CCSSEqExpression {
    #[must_use]
    pub const fn new(eq: PredEq, expr: CCSSSvvOpSvvExpr) -> Self {
        Self { eq, expr }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StrengthAndWeight {
    Weak(Option<NotNan<f64>>),
    Medium(Option<NotNan<f64>>),
    Strong(Option<NotNan<f64>>),
    Require,
}

impl StrengthAndWeight {
    #[must_use]
    pub fn to_number(&self) -> f64 {
        match self {
            Self::Weak(opt_n) => opt_n.map_or(cassowary::strength::WEAK, |nn| {
                (nn * cassowary::strength::WEAK).into_inner()
            }),
            Self::Medium(opt_n) => opt_n.map_or(cassowary::strength::MEDIUM, |nn| {
                (nn * cassowary::strength::MEDIUM).into_inner()
            }),
            Self::Strong(opt_n) => opt_n.map_or(cassowary::strength::STRONG, |nn| {
                (nn * cassowary::strength::STRONG).into_inner()
            }),
            Self::Require => cassowary::strength::REQUIRED,
        }
    }
}

impl std::fmt::Display for StrengthAndWeight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Weak(x) => {
                if let Some(i) = x {
                    write!(f, " !weak({i})")
                } else {
                    write!(f, " !weak")
                }
            }
            Self::Medium(x) => {
                if let Some(i) = x {
                    write!(f, " !medium({i})")
                } else {
                    write!(f, " !medium")
                }
            }
            Self::Strong(x) => {
                if let Some(i) = x {
                    write!(f, " !strong({i})")
                } else {
                    write!(f, " !strong")
                }
            }
            Self::Require => {
                write!(f, " !require")
            }
        }
    }
}

fn disp_opt<T: std::fmt::Display>(o: Option<T>) -> String {
    o.map_or(String::new(), |x| format!("{x}"))
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CCSS {
    pub svv_op_svvs: CCSSSvvOpSvvExpr,
    pub eq_exprs: Vec<CCSSEqExpression>,
    pub opt_sw: Option<StrengthAndWeight>,
}
impl std::fmt::Display for CCSS {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            svv_op_svvs: var_op_vars,
            eq_exprs,
            opt_sw: sw,
        } = self;
        let sw_str = disp_opt(sw.as_ref());
        write!(f, "{var_op_vars} ")?;
        for eqe in eq_exprs {
            write!(f, "{eqe}")?;
        }
        write!(f, "{sw_str}")
    }
}

impl CCSS {
    #[must_use]
    pub fn new(
        svv_op_svvs: CCSSSvvOpSvvExpr,
        eq_exprs: Vec<CCSSEqExpression>,
        opt_sw: Option<StrengthAndWeight>,
    ) -> Self {
        Self {
            svv_op_svvs,
            eq_exprs,
            opt_sw,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneralVar(pub IdStr, pub ScopeViewVariable);

impl GeneralVar {
    #[must_use]
    pub fn with_virtual_name(mut self, name_space: &IdStr) -> Self {
        self.0 = name_space.clone() + "." + &self.0;
        self
    }
    #[must_use]
    pub fn new(prop: IdStr) -> Self {
        Self(prop, ScopeViewVariable::default())
    }
}

static VIRTUAL_PROPS: [&str; 6] = ["width", "height", "top", "left", "bottom", "right"];
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Virtual(pub IdStr, pub Vec<GeneralVar>);

impl Virtual {
    #[must_use]
    pub const fn name(&self) -> &IdStr {
        &self.0
    }
}

type VirtualProcessed<'a> = (
    IndexMap<
        &'a str,
        (
            Variable, /*top */
            Variable, /*current */
            Option<&'a GeneralVar>,
        ),
        BuildHasherDefault<CustomHasher>,
    >,
    (ConstraintList /*top */, ConstraintList),
    HashMap<&'a str, Option<&'a GeneralVar>>,
);

impl Virtual {
    #[allow(clippy::many_single_char_names)]
    #[must_use]
    /// # Panics
    ///
    /// Will panic if y is 0
    pub fn process(&self) -> VirtualProcessed {
        // let mut added_prop_var = vec![];
        let mut gvs_map = self
            .1
            .iter()
            .map(|gv| (gv.0.as_str(), Some(gv)))
            .collect::<HashMap<_, _>>();

        //NOTE need strict order for [w,h,t,l,b,r] iter
        let gvs_match_props = VIRTUAL_PROPS
            .iter()
            .map(|&p| {
                if let Some(&Some(gv)) = gvs_map.get(p) {
                    (p, (Variable::new(), Variable::new(), Some(gv)))
                } else {
                    (p, (Variable::new(), Variable::new(), None))
                }
            })
            .collect::<IndexMap<_, _, BuildHasherDefault<CustomHasher>>>();

        //NOTE need strict order for [(tw, w), (th, h), (tt, t), (tl, l), (tb, b), (tr, r)] iter
        let vars_constraints = if let &[(tw, w), (th, h), (tt, t), (tl, l), (tb, b), (tr, r)] =
            gvs_match_props
                .iter()
                .map(|(_, (top_var, var, _))| (*top_var, *var))
                .collect::<Vec<_>>()
                .as_slice()
        {
            (
                [
                    tt | WeightedRelation::EQ(REQUIRED) | (tb - th),
                    tl | WeightedRelation::EQ(REQUIRED) | (tr - tw),
                    // tb | WeightedRelation::EQ(REQUIRED) | (tt + th),
                    // tr | WeightedRelation::EQ(REQUIRED) | (tl + tw),
                    // tw | WeightedRelation::EQ(REQUIRED) | (tr - tl),
                    // th | WeightedRelation::EQ(REQUIRED) | (tb - tt),
                    // .....
                    tb | WeightedRelation::GE(REQUIRED) | tt,
                    tr | WeightedRelation::GE(REQUIRED) | tl,
                    tw | WeightedRelation::GE(REQUIRED) | 0.0,
                    th | WeightedRelation::GE(REQUIRED) | 0.0,
                    tt | WeightedRelation::GE(WEAK) | 0.0,
                    tl | WeightedRelation::GE(WEAK) | 0.0,
                ],
                [
                    t | WeightedRelation::EQ(REQUIRED) | (b - h),
                    l | WeightedRelation::EQ(REQUIRED) | (r - w),
                    // b | WeightedRelation::EQ(REQUIRED) | (t + h),
                    // r | WeightedRelation::EQ(REQUIRED) | (l + w),
                    // w | WeightedRelation::EQ(REQUIRED) | (r - l),
                    // h | WeightedRelation::EQ(REQUIRED) | (b - t),
                    // .....
                    b | WeightedRelation::GE(REQUIRED) | t,
                    r | WeightedRelation::GE(REQUIRED) | l,
                    w | WeightedRelation::GE(REQUIRED) | 0.0,
                    h | WeightedRelation::GE(REQUIRED) | 0.0,
                    t | WeightedRelation::GE(WEAK) | 0.0,
                    l | WeightedRelation::GE(WEAK) | 0.0,
                    // ────────────────────────────────────────────────────────────────────────────────
                ],
            )
        } else {
            unreachable!()
        };

        for p in VIRTUAL_PROPS {
            let _ = gvs_map.remove(p);
        }

        (gvs_match_props, vars_constraints, gvs_map)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CassowaryVar {
    General(GeneralVar),
    Virtual(Virtual),
}

#[derive(Debug, Clone, PartialEq)]
pub struct CassowaryGeneralMap {
    pub(crate) map: HashMap<IdStr, Variable, BuildHasherDefault<CustomHasher>>,
    pub(crate) v_v_suggest: Dict<Variable, f64>,
    pub(crate) virtual_constraints:
        HashMap<IdStr, ConstraintList, BuildHasherDefault<CustomHasher>>,
    pub(crate) top_virtual_constraints:
        HashMap<IdStr, ConstraintList, BuildHasherDefault<CustomHasher>>,
    pub(crate) top_map: HashMap<IdStr, Variable, BuildHasherDefault<CustomHasher>>,
    pub(crate) top_v_v_suggest: Dict<Variable, f64>,
    pub(crate) parent: Option<Rc<CassowaryGeneralMap>>,
    // pub(crate) cassowary_map: Option<Rc<CassowaryMap>>,
}

pub type ConstraintList = [Constraint; 8];

impl CassowaryGeneralMap {
    // #[must_use]
    // pub fn top_v(&self, var: &Variable) -> Option<f64> {
    //     self.top_v_v.get(var).copied()
    // }
    #[must_use]
    pub fn constraint(&self, v_name: &str) -> Option<&ConstraintList> {
        self.virtual_constraints.get(v_name)
    }

    pub fn top_var<BK>(&self, key: &BK) -> Option<Variable>
    where
        BK: core::hash::Hash + Eq + ?Sized,
        IdStr: std::borrow::Borrow<BK>,
    {
        self.top_map.get(key).copied()
    }
    // #[must_use]
    // pub fn v(&self, var: &Variable) -> Option<f64> {
    //     self.v_v.get(var).copied()
    // }

    pub fn var<BK>(&self, key: &BK) -> Option<Variable>
    where
        BK: core::hash::Hash + Eq + ?Sized,
        IdStr: std::borrow::Borrow<BK>,
    {
        self.map.get(key).copied()
    }

    pub fn insert_with_suggest(&mut self, id: IdStr, v: f64) {
        self.insert_with_var_and_suggest_opt(id, None, None, Some(v));
    }
    pub fn insert_with_var(&mut self, id: IdStr, top_var: Variable, var: Variable) {
        self.insert_with_var_and_suggest_opt(id, Some(top_var), Some(var), None);
    }

    pub fn insert_with_var_and_suggest(
        &mut self,
        id: IdStr,
        top_var: Variable,
        var: Variable,
        v: f64,
    ) {
        self.insert_with_var_and_suggest_opt(id, Some(top_var), Some(var), Some(v));
    }

    fn insert_with_var_and_suggest_opt(
        &mut self,
        id: IdStr,
        opt_top_var: Option<Variable>,
        opt_var: Option<Variable>,
        opt_v: Option<f64>,
    ) {
        if !self.top_map.contains_key(&id) {
            let top_var = opt_top_var.unwrap_or_default();
            self.top_map.insert(id.clone(), top_var);
            if let Some(v) = opt_v {
                self.top_v_v_suggest.insert(top_var, v);
            }
        }
        let var = opt_var.unwrap_or_default();

        self.map.insert(id, var);
        if let Some(v) = opt_v {
            self.v_v_suggest.insert(var, v);
        }
    }

    fn insert_with_suggest_not_overwrite(&mut self, id: IdStr, v: f64) {
        if !self.top_map.contains_key(&id) {
            let var = Variable::new();
            self.top_map.insert(id.clone(), var);
            self.top_v_v_suggest.insert(var, v);
        }
        if !self.map.contains_key(&id) {
            let var2 = Variable::new();
            self.map.insert(id, var2);
            self.v_v_suggest.insert(var2, v);
        }
    }

    pub fn insert_constants(
        &mut self,
        v_name: IdStr,
        top_constants: ConstraintList,
        constants: ConstraintList,
    ) {
        if self.top_virtual_constraints.contains_key(&v_name) {
            self.top_virtual_constraints
                .insert(v_name.clone(), top_constants);
        }

        self.virtual_constraints.insert(v_name, constants);
    }
    #[must_use]
    pub fn new() -> Self {
        let top_map = HashMap::<IdStr, Variable, BuildHasherDefault<CustomHasher>>::default();
        let top_v_v: Dict<Variable, f64> = Dict::new();

        let map = top_map.clone();
        let v_v = top_v_v.clone();

        let virtual_constraints =
            HashMap::<IdStr, ConstraintList, BuildHasherDefault<CustomHasher>>::default();
        let top_virtual_constraints = virtual_constraints.clone();

        // let hgap = Variable::new();
        // map.insert("hgap".into(), hgap);
        // v_v.insert(hgap, 10.0);

        Self {
            map,
            v_v_suggest: v_v,
            top_map,
            top_v_v_suggest: top_v_v,
            parent: None,
            virtual_constraints,
            top_virtual_constraints,
            // cassowary_map: None,
        }
    }
    // pub fn with_default(mut self) -> Self {
    //     self.insert("hgap".into(), 10.0);
    //     self.insert("vgap".into(), 10.0);

    //     self
    // }
    //TODO global prop config
    #[must_use]
    pub fn with_default_not_overwrite(mut self) -> Self {
        self.insert_with_suggest_not_overwrite("hgap".into(), 10.0);
        self.insert_with_suggest_not_overwrite("vgap".into(), 10.0);
        self.insert_with_suggest_not_overwrite("baseline".into(), 16.0);

        self
    }
}

impl Default for CassowaryGeneralMap {
    fn default() -> Self {
        Self::new()
    }
}

impl std::ops::Add<CassowaryGeneralMap> for Rc<CassowaryGeneralMap> {
    type Output = CassowaryGeneralMap;
    fn add(self, current_new: CassowaryGeneralMap) -> Self::Output {
        Self::Output {
            map: current_new.map.union_with(self.map.clone(), |l, _| l),
            v_v_suggest: current_new
                .v_v_suggest
                .union_with(self.v_v_suggest.clone(), |l, _| l),
            top_map: self
                .top_map
                .clone()
                .union_with(current_new.top_map.clone(), |l, _| l),
            top_v_v_suggest: self
                .top_v_v_suggest
                .clone()
                .union_with(current_new.top_v_v_suggest.clone(), |l, _| l),
            virtual_constraints: current_new
                .virtual_constraints
                .union(self.virtual_constraints.clone()),
            top_virtual_constraints: current_new
                .top_virtual_constraints
                .union(self.top_virtual_constraints.clone()),
            parent: Some(self),
            // cassowary_map: current_new.cassowary_map,
        }
    }
}

impl std::ops::Add<Rc<CassowaryMap>> for CassowaryGeneralMap {
    type Output = Self;

    fn add(mut self, self_cassowary_map: Rc<CassowaryMap>) -> Self::Output {
        self.map = self_cassowary_map
            .map
            .clone()
            .union_with(self.map, |l, _| l);

        // self.cassowary_map = Some(self_cassowary_map);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CassowaryMap {
    pub(crate) map: HashMap<IdStr, Variable, BuildHasherDefault<CustomHasher>>,
    v_k: HashMap<Variable, IdStr, BuildHasherDefault<CustomHasher>>,
}

impl Default for CassowaryMap {
    fn default() -> Self {
        Self::new()
    }
}

impl CassowaryMap {
    // fn var(&self, key: &IdStr) -> Option<Variable> {
    //     self.map.get(key).copied()
    // }

    #[must_use]
    pub fn prop(&self, var: &Variable) -> Option<&IdStr> {
        self.v_k.get(var)
    }

    pub fn var<BK>(&self, key: &BK) -> Option<Variable>
    where
        BK: core::hash::Hash + Eq + ?Sized,
        IdStr: std::borrow::Borrow<BK>,
    {
        self.map.get(key).copied()
    }

    #[must_use]
    #[allow(clippy::similar_names)]
    pub fn new() -> Self {
        let mut map = HashMap::<IdStr, Variable, BuildHasherDefault<CustomHasher>>::default();
        let mut v_k = HashMap::<Variable, IdStr, BuildHasherDefault<CustomHasher>>::default();
        // @add self layout ─────────────────────────────────────────────────────────────────

        let width = Variable::new();
        let width_idstr: IdStr = "width".into();
        map.insert(width_idstr.clone(), width);
        v_k.insert(width, width_idstr);

        let height = Variable::new();
        let height_idstr: IdStr = "height".into();
        map.insert(height_idstr.clone(), height);
        v_k.insert(height, height_idstr);

        let top = Variable::new();
        let top_idstr: IdStr = "top".into();
        map.insert(top_idstr.clone(), top);
        v_k.insert(top, top_idstr);

        let left = Variable::new();
        let left_idstr: IdStr = "left".into();
        map.insert(left_idstr.clone(), left);
        v_k.insert(left, left_idstr);

        let bottom = Variable::new();
        let bottom_idstr: IdStr = "bottom".into();
        map.insert(bottom_idstr.clone(), bottom);
        v_k.insert(bottom, bottom_idstr);

        let right = Variable::new();
        let right_idstr: IdStr = "right".into();
        map.insert(right_idstr.clone(), right);
        v_k.insert(right, right_idstr);

        let z = Variable::new();
        let z_idstr: IdStr = "z".into();
        map.insert(z_idstr.clone(), z);
        v_k.insert(z, z_idstr);

        let origin_x = Variable::new();
        let origin_x_idstr: IdStr = "origin_x".into();
        map.insert(origin_x_idstr.clone(), origin_x);
        v_k.insert(origin_x, origin_x_idstr);

        let origin_y = Variable::new();
        let origin_y_idstr: IdStr = "origin_y".into();
        map.insert(origin_y_idstr.clone(), origin_y);
        v_k.insert(origin_y, origin_y_idstr);

        let origin_z = Variable::new();
        let origin_z_idstr: IdStr = "origin_z".into();
        map.insert(origin_z_idstr.clone(), origin_z);
        v_k.insert(origin_z, origin_z_idstr);

        let align_x = Variable::new();
        let align_x_idstr: IdStr = "align_x".into();
        map.insert(align_x_idstr.clone(), align_x);
        v_k.insert(align_x, align_x_idstr);

        let align_y = Variable::new();
        let align_y_idstr: IdStr = "align_y".into();
        map.insert(align_y_idstr.clone(), align_y);
        v_k.insert(align_y, align_y_idstr);

        let align_z = Variable::new();
        let align_z_idstr: IdStr = "align_z".into();
        map.insert(align_z_idstr.clone(), align_z);
        v_k.insert(align_z, align_z_idstr);

        Self { map, v_k }
    }
}
