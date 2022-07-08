use std::{cell::RefCell, rc::Rc};

use cassowary::{Solver, Variable, WeightedRelation};
use emg_core::{im::HashMap, IdStr, NotNan, Vector};
use parse_display::{Display, FromStr};
use slotmap::SlotMap;

use crate::LayoutCalculated;

/*
 * @Author: Rais
 * @Date: 2022-06-23 22:52:57
 * @LastEditTime: 2022-07-08 22:24:54
 * @LastEditors: Rais
 * @Description:
 */

#[derive(Debug, Clone, Display, PartialEq, Eq)]
pub(crate) enum NameChars<Ix = IdStr> {
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
pub(crate) enum PredEq {
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
enum Scope {
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
struct PredVariable(IdStr);

impl std::ops::Deref for PredVariable {
    type Target = IdStr;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ScopeViewVariable {
    scope: Option<Scope>,
    view: Option<NameChars>,
    variable: Option<PredVariable>,
}

impl ScopeViewVariable {
    pub(crate) fn scope(&self) -> Option<Scope> {
        self.scope
    }

    pub(crate) fn view(&self) -> Option<&NameChars> {
        self.view.as_ref()
    }
}

#[derive(Debug, Clone, Display, PartialEq, Eq)]
pub(crate) enum PredOp {
    #[display("+")]
    Add,
    #[display("-")]
    Sub,
    #[display("*")]
    Mul,
}

#[derive(Debug, Clone, Display, PartialEq, Eq)]
#[display("{op} {var}")]
pub(crate) struct CCSSOpSvv {
    op: PredOp,
    svv: ScopeViewVariable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CCSSSvvOpSvvExpr {
    svv: ScopeViewVariable,
    op_exprs: Vec<CCSSOpSvv>,
}

#[derive(Debug, Clone, Display, PartialEq, Eq)]
#[display("{eq} {expr}")]
pub(crate) struct CCSSEqExpression {
    eq: PredEq,
    expr: CCSSSvvOpSvvExpr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum StrengthAndWeight {
    Weak(Option<NotNan<f64>>),
    Medium(Option<NotNan<f64>>),
    Strong(Option<NotNan<f64>>),
    Require,
}

impl StrengthAndWeight {
    pub fn to_number(&self) -> f64 {
        match self {
            StrengthAndWeight::Weak(opt_n) => opt_n.map_or(cassowary::strength::WEAK, |nn| {
                (nn * cassowary::strength::WEAK).into_inner()
            }),
            StrengthAndWeight::Medium(opt_n) => opt_n.map_or(cassowary::strength::MEDIUM, |nn| {
                (nn * cassowary::strength::MEDIUM).into_inner()
            }),
            StrengthAndWeight::Strong(opt_n) => opt_n.map_or(cassowary::strength::STRONG, |nn| {
                (nn * cassowary::strength::STRONG).into_inner()
            }),
            StrengthAndWeight::Require => cassowary::strength::REQUIRED,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CCSS {
    svv_op_svvs: CCSSSvvOpSvvExpr,
    eq_exprs: Vec<CCSSEqExpression>,
    opt_sw: Option<StrengthAndWeight>,
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
    map: HashMap<IdStr, Variable>,
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

    pub fn var<BK>(&self, key: &BK) -> Option<&Variable>
    where
        BK: core::hash::Hash + Eq + ?Sized,
        IdStr: std::borrow::Borrow<BK>,
    {
        self.map.get(key)
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

        // let z = Variable::new();
        // map.insert("z".into(), z);
        // v_k.insert(z, "z".into());

        // let origin_x = Variable::new();
        // map.insert("origin_x".into(), origin_x);
        // v_k.insert(origin_x, "origin_x".into());

        // let origin_y = Variable::new();
        // map.insert("origin_y".into(), origin_y);
        // v_k.insert(origin_y, "origin_y".into());

        // let origin_z = Variable::new();
        // map.insert("origin_z".into(), origin_z);
        // v_k.insert(origin_z, "origin_z".into());

        // let align_x = Variable::new();
        // map.insert("align_x".into(), align_x);
        // v_k.insert(align_x, "align_x".into());

        // let align_y = Variable::new();
        // map.insert("align_y".into(), align_y);
        // v_k.insert(align_y, "align_y".into());

        // let align_z = Variable::new();
        // map.insert("align_z".into(), align_z);
        // v_k.insert(align_z, "align_z".into());

        CassowaryMap { map, v_k }
    }
}
