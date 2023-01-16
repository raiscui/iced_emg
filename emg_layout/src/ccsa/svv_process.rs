/*
 * @Author: Rais
 * @Date: 2022-07-21 10:50:01
 * @LastEditTime: 2023-01-04 21:31:04
 * @LastEditors: Rais
 * @Description:
 */
#![allow(clippy::match_same_arms)]
use std::{
    hash::{BuildHasherDefault, Hash},
    rc::Rc,
};

use cassowary::{Constraint, Expression, Variable, WeightedRelation};
use either::Either;
use emg_common::{
    im::{ordset, HashSet, OrdSet},
    IdStr,
};
use emg_hasher::CustomHasher;
use emg_state::{Dict, StateAnchor};
use tracing::{debug_span, instrument, warn};
use Either::{Left, Right};

use super::{
    CCSSOpSvv, CCSSSvvOpSvvExpr, CassowaryGeneralMap, CassowaryMap, ConstraintList, NameChars,
    PredEq, PredOp, PredVariable, Scope, ScopeViewVariable, StrengthAndWeight,
};

pub(crate) fn eq_opt_sw_to_weighted_relation(
    eq: PredEq,
    opt_sw: &Option<StrengthAndWeight>,
) -> WeightedRelation {
    let weight = opt_sw
        .as_ref()
        .map_or(cassowary::strength::MEDIUM, StrengthAndWeight::to_number);
    match eq {
        PredEq::Eq => WeightedRelation::EQ(weight),
        PredEq::Lt => todo!(),
        PredEq::Le => WeightedRelation::LE(weight),
        PredEq::Ge => WeightedRelation::GE(weight),
        PredEq::Gt => todo!(),
    }
}

type SvvOpSvvsToExpr = (
    OrdSet<Constraint>,
    Option<(
        Expression,
        HashSet<Variable, BuildHasherDefault<CustomHasher>>,
    )>,
);

#[instrument(skip(children_cass_maps))]
pub(crate) fn svv_op_svvs_to_expr<Ix>(
    svv_op_svvs: &CCSSSvvOpSvvExpr,
    children_cass_maps: &Dict<Ix, (Rc<CassowaryMap>, StateAnchor<Vec<Constraint>>)>,
    current_cassowary_inherited_generals: &Rc<CassowaryGeneralMap>,
) -> SvvOpSvvsToExpr
where
    Ix: std::fmt::Debug
        + Clone
        + Hash
        + Eq
        + PartialEq
        + PartialOrd
        + Ord
        + Default
        + std::fmt::Display
        + std::borrow::Borrow<str>,
{
    let CCSSSvvOpSvvExpr {
        svv: main_svv,
        op_exprs,
    } = svv_op_svvs;

    let (opt_first_var_or_expr, opt_first_cs) = svv_to_var(
        main_svv,
        children_cass_maps,
        current_cassowary_inherited_generals,
    );
    let mut all_cs: OrdSet<Constraint> =
        opt_first_cs.map_or_else(|| ordset![], |cs| cs.into_iter().collect());

    let opt_expr_vars = opt_first_var_or_expr.map(|first_var_or_expr| {
        // let mut all_consensus_constraints_sa : Vector<Anchor<Vec<Constraint>>> = vector![first_consensus_constraints_sa];

        let mut child_vars = first_var_or_expr.as_ref().either(
            |var| HashSet::with_hasher(BuildHasherDefault::<CustomHasher>::default()).update(*var),
            |_| HashSet::with_hasher(BuildHasherDefault::<CustomHasher>::default()),
        );
        // let mut child_vars = HashSet::with_hasher(BuildHasherDefault::<CustomHasher>::default()) .update(first_var);

        let expr = op_exprs.iter().fold(
            first_var_or_expr.either_into(),
            |exp: Expression, op_expr| {
                let CCSSOpSvv { op, svv } = op_expr;

                let (opt_var_or_expr, opt_cs) = svv_to_var(
                    svv,
                    children_cass_maps,
                    current_cassowary_inherited_generals,
                );

                if let Some(cs) = opt_cs {
                    all_cs.extend(cs.into_iter());
                }

                if let Some(var_or_expr) = opt_var_or_expr {
                    if let Some(var) = var_or_expr.as_ref().left() {
                        child_vars.insert(*var);
                    }

                    match op {
                        PredOp::Add => {
                            var_or_expr.either_with(exp, |expr, l| expr + l, |expr, r| expr + r)
                        }
                        PredOp::Sub => {
                            var_or_expr.either_with(exp, |expr, l| expr - l, |expr, r| expr - r)
                        }
                        PredOp::Mul => {
                            todo!("mul * ")
                            // var_or_expr.either_with(exp, |expr, l| expr * l, |expr, r| expr * r)
                        }
                    }
                } else {
                    exp
                }
            },
        );
        (expr, child_vars)
    });
    (all_cs, opt_expr_vars)
}

#[instrument(skip(children_cass_maps))]
fn svv_to_var<Ix>(
    scope_view_variable: &ScopeViewVariable,
    children_cass_maps: &Dict<Ix, (Rc<CassowaryMap>, StateAnchor<Vec<Constraint>>)>,
    current_cassowary_inherited_generals: &Rc<CassowaryGeneralMap>,
) -> (Option<Either<Variable, Expression>>, Option<ConstraintList>)
where
    Ix: std::fmt::Debug
        + Clone
        + Hash
        + Eq
        + PartialEq
        + PartialOrd
        + Ord
        + Default
        + std::fmt::Display
        + std::borrow::Borrow<str>,
{
    let ScopeViewVariable {
        scope,
        view,
        variable,
    } = scope_view_variable;
    let var = match (scope, view, variable) {
        (None, None, None) => unreachable!(),
        (None | Some(Scope::Local), None, Some(PredVariable(prop))) => {
            //@ inherited_generals
            warn!(
                "local => current_cassowary_inherited_generals: {:#?}",
                &current_cassowary_inherited_generals
            );
            warn!("local => current prop {}", &prop);

            (
                current_cassowary_inherited_generals.var(prop).map_or_else(
                    || panic!("inherited generals: {prop} -> not find"),
                    |v| Some(Left(v)),
                ),
                None,
            )
        }
        (None, Some(name), None) => {
            //NOTE no prop
            match name {
                NameChars::Id(_) => todo!(),
                NameChars::Class(_) => todo!(),
                NameChars::Element(_) => todo!(),
                NameChars::Virtual(_) => todo!(),
                NameChars::Number(n) => (Some(Right((*n).into())), None),
                NameChars::Next(_) => todo!(),
                NameChars::Last(_) => todo!(),
                NameChars::First(_) => todo!(),
            }
        }
        (None, Some(name), Some(PredVariable(prop))) => match name {
            NameChars::Id(id) => {
                let _debug_span_ = debug_span!("->[ get child variable ] ").entered();

                warn!("[svv_to_var] parsed scope_view_variable,  find child var : child id:{:?} prop:{:?}",&id,&prop);

                (
                    children_cass_maps.get(id.as_str()).map(|(cass_map, ..)| {
                        warn!(
                            "[svv_to_var] got child id:{:?} cass_map: {:?}",
                            &id, &cass_map
                        );

                        let var = cass_map.var(prop).unwrap();

                        Left(var)
                    }),
                    None,
                )
            }
            NameChars::Class(_) => todo!(),
            NameChars::Element(_) => todo!(),
            //TODO return virtual constraints , not need add after this
            NameChars::Virtual(v_name) => {
                let var = current_cassowary_inherited_generals
                    .var(&(v_name.clone() + "." + prop))
                    .map_or_else(
                        || panic!("inherited generals: Virtual:{v_name}.{prop} -> not find"),
                        |v| Some(Left(v)),
                    );

                let expr = current_cassowary_inherited_generals
                    .constraint(v_name)
                    .cloned()
                    .or_else(|| {
                        panic!(
                            "inherited generals: Virtual:{v_name}.{prop} ->constraint ,   not find"
                        )
                    });
                (var, expr)
            }
            NameChars::Number(_) => todo!(),
            NameChars::Next(_) => todo!(),
            NameChars::Last(_) => todo!(),
            NameChars::First(_) => todo!(),
        },
        (Some(_), None, None) => todo!(),
        (Some(s), None, Some(PredVariable(prop))) => match s {
            Scope::Local => unreachable!("should processed in other where (at top)"),
            Scope::Parent(lv) => (
                scope_parent_val(*lv, prop, current_cassowary_inherited_generals),
                None,
            ),
            Scope::Global => (
                current_cassowary_inherited_generals
                    .top_var(prop)
                    .map_or_else(
                        || panic!("top global generals: {prop} -> not find"),
                        |v| Some(Left(v)),
                    ),
                None,
            ),
        },
        (Some(s), Some(name), None) => {
            match s {
                Scope::Local => todo!(),
                Scope::Parent(_lv) => match name {
                    NameChars::Id(_) => todo!(),
                    NameChars::Class(_) => todo!(),
                    NameChars::Element(_) => todo!(),
                    // NameChars::Element(e_as_prop) => {
                    // scope_parent_val(*lv, e_as_prop, current_cassowary_inherited_generals)
                    // }
                    NameChars::Virtual(_) => todo!(),
                    NameChars::Number(_) => todo!(),
                    NameChars::Next(_) => todo!(),
                    NameChars::Last(_) => todo!(),
                    NameChars::First(_) => todo!(),
                },
                Scope::Global => {
                    match name {
                        NameChars::Id(_) => todo!(),
                        NameChars::Class(_) => todo!(),
                        NameChars::Element(_) => todo!(),
                        // NameChars::Element(e_as_prop) => {
                        //     //NOTE use as prop
                        //     if let Some(v) = current_cassowary_inherited_generals.top_var(e_as_prop)
                        //     {
                        //         warn!(
                        //             "[Scope] global scope, find top var(Element as prop) : {:?}",
                        //             &e_as_prop
                        //         );

                        //         Some(Left(v))
                        //     } else {
                        //         // None
                        //         panic!("top global generals: {} -> not find", e_as_prop)
                        //     }
                        // }
                        NameChars::Virtual(_) => todo!(),
                        NameChars::Number(_) => todo!(),
                        NameChars::Next(_) => todo!(),
                        NameChars::Last(_) => todo!(),
                        NameChars::First(_) => todo!(),
                    }
                }
            }
        }
        (Some(_), Some(_), Some(_)) => todo!(),
    };
    var
}

fn scope_parent_val(
    lv: u8,
    prop: &IdStr,
    current_cassowary_inherited_generals: &Rc<CassowaryGeneralMap>,
) -> Option<Either<Variable, Expression>> {
    let mut opt_p = &current_cassowary_inherited_generals.parent;
    let mut n = 1u8;
    while let Some(p) = opt_p && n < lv {
                        warn!("[svv_to_var] [parent] {}: {}",lv,n);

                        opt_p = &p.parent;
                        n+=1;
                    }
    warn!("[svv_to_var] [parent] end, {}: {}", lv, n);

    opt_p.as_ref().and_then(|p| p.var(prop)).map_or_else(
        || panic!("parent {lv}:{n} can't get prop:{prop}"),
        |v| Some(Left(v)),
    )
}
