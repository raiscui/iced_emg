use cassowary::Variable;
use emg_core::{im::HashMap, IdStr, NotNan, Vector};
use parse_display::{Display, FromStr};
mod impl_refresh;
mod ops;
/*
 * @Author: Rais
 * @Date: 2022-06-23 22:52:57
 * @LastEditTime: 2022-07-13 17:55:50
 * @LastEditors: Rais
 * @Description:
 */

#[derive(Debug, Clone, Display, PartialEq, Eq)]
pub enum NameChars<Ix = IdStr> {
    #[display("#{0}")]
    Id(Ix), // #xxx
    #[display(".{0}")]
    Class(Ix), // .xxx
    #[display("{0}")]
    Element(Ix), // xxxx
    #[display("\"{0:?}\"")]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopeViewVariable {
    pub scope: Option<Scope>,
    pub view: Option<NameChars>,
    pub variable: Option<PredVariable>,
}

impl std::fmt::Display for ScopeViewVariable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let scope = self
            .scope
            .as_ref()
            .map_or("".to_string(), |x| format!("{}", x));
        let view = self
            .view
            .as_ref()
            .map_or("".to_string(), |x| format!("{}", x));
        let variable = self
            .variable
            .as_ref()
            .map_or("".to_string(), |x| format!("{}", x));

        write!(f, "{}{}{}", scope, view, variable)
    }
}

impl ScopeViewVariable {
    pub fn new(
        scope: Option<Scope>,
        view: Option<NameChars>,
        variable: Option<PredVariable>,
    ) -> Self {
        Self {
            scope,
            view,
            variable,
        }
    }
    pub fn turn_with_var(&self, var: &str) -> Self {
        let Self {
            scope,
            view,
            variable,
        } = self;
        Self::new(scope.clone(), view.clone(), Some(PredVariable(var.into())))
    }
    pub fn new_number(number: f64) -> Self {
        Self::new(
            None,
            Some(NameChars::Number(NotNan::new(number).unwrap())),
            None,
        )
    }
    pub fn new_id_var(id: &str, var: &str) -> Self {
        Self::new(
            None,
            Some(NameChars::Id(id.into())),
            Some(PredVariable(var.into())),
        )
    }

    pub(crate) fn scope(&self) -> Option<Scope> {
        self.scope
    }

    pub(crate) fn view(&self) -> Option<&NameChars> {
        self.view.as_ref()
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
#[display("{op} {svv}")]
pub struct CCSSOpSvv {
    pub op: PredOp,
    pub svv: ScopeViewVariable,
}
impl CCSSOpSvv {
    pub fn new(op: PredOp, svv: ScopeViewVariable) -> Self {
        Self { op, svv }
    }

    pub fn new_add(svv: ScopeViewVariable) -> Self {
        Self {
            op: PredOp::Add,
            svv,
        }
    }
    pub fn new_sub(svv: ScopeViewVariable) -> Self {
        Self {
            op: PredOp::Sub,
            svv,
        }
    }
    pub fn new_mul(svv: ScopeViewVariable) -> Self {
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
    pub fn new(svv: ScopeViewVariable, op_exprs: Vec<CCSSOpSvv>) -> Self {
        Self { svv, op_exprs }
    }
}

impl std::fmt::Display for CCSSSvvOpSvvExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.svv)?;
        for op_expr in self.op_exprs.iter() {
            write!(f, "{}", op_expr)?;
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
    pub fn new(eq: PredEq, expr: CCSSSvvOpSvvExpr) -> Self {
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
                    write!(f, " !weak({})", i)
                } else {
                    write!(f, " !weak")
                }
            }
            Self::Medium(x) => {
                if let Some(i) = x {
                    write!(f, " !medium({})", i)
                } else {
                    write!(f, " !medium")
                }
            }
            Self::Strong(x) => {
                if let Some(i) = x {
                    write!(f, " !strong({})", i)
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
    o.map_or("".to_string(), |x| format!("{}", x))
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
        write!(f, "{} ", var_op_vars)?;
        for eqe in eq_exprs {
            write!(f, "{}", eqe)?;
        }
        write!(f, "{}", sw_str)
    }
}

impl CCSS {
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
pub struct CCSSVecDisp(pub Vector<CCSS>);
impl std::fmt::Display for CCSSVecDisp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "CCSSS [")?;
        for ccss in &self.0 {
            writeln!(f, "{},", ccss)?;
        }
        writeln!(f, "]")
    }
}

// #[derive(Debug, Clone, PartialEq, Eq)]
// pub(crate) struct CCSSList(Vector<CCSS>);

// impl std::ops::DerefMut for CCSSList {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.0
//     }
// }

// impl std::ops::Deref for CCSSList {
//     type Target = Vector<CCSS>;

//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

// impl CCSSList {
//     pub fn new() -> Self {
//         Self(Vector::new())
//     }
// }

#[derive(Debug, Clone, PartialEq)]
pub struct CassowaryMap {
    pub(crate) map: HashMap<IdStr, Variable>,
    v_k: HashMap<Variable, IdStr>,
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

    pub fn new() -> Self {
        let mut map: HashMap<IdStr, Variable> = HashMap::new();
        let mut v_k = HashMap::new();
        // @add self layout ─────────────────────────────────────────────────────────────────

        let width = Variable::new();
        map.insert("width".into(), width);
        v_k.insert(width, "width".into());

        let height = Variable::new();
        map.insert("height".into(), height);
        v_k.insert(height, "height".into());

        let top = Variable::new();
        map.insert("top".into(), top);
        v_k.insert(top, "top".into());

        let left = Variable::new();
        map.insert("left".into(), left);
        v_k.insert(left, "left".into());

        let bottom = Variable::new();
        map.insert("bottom".into(), bottom);
        v_k.insert(bottom, "bottom".into());

        let right = Variable::new();
        map.insert("right".into(), right);
        v_k.insert(right, "right".into());

        let z = Variable::new();
        map.insert("z".into(), z);
        v_k.insert(z, "z".into());

        let origin_x = Variable::new();
        map.insert("origin_x".into(), origin_x);
        v_k.insert(origin_x, "origin_x".into());

        let origin_y = Variable::new();
        map.insert("origin_y".into(), origin_y);
        v_k.insert(origin_y, "origin_y".into());

        let origin_z = Variable::new();
        map.insert("origin_z".into(), origin_z);
        v_k.insert(origin_z, "origin_z".into());

        let align_x = Variable::new();
        map.insert("align_x".into(), align_x);
        v_k.insert(align_x, "align_x".into());

        let align_y = Variable::new();
        map.insert("align_y".into(), align_y);
        v_k.insert(align_y, "align_y".into());

        let align_z = Variable::new();
        map.insert("align_z".into(), align_z);
        v_k.insert(align_z, "align_z".into());

        CassowaryMap { map, v_k }
    }
}
