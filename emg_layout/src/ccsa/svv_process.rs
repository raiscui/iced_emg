/*
 * @Author: Rais
 * @Date: 2022-07-21 10:50:01
 * @LastEditTime: 2022-07-21 15:34:38
 * @LastEditors: Rais
 * @Description:
 */

use std::{
    hash::{BuildHasherDefault, Hash},
    rc::Rc,
};

use cassowary::{Constraint, Expression, Variable, WeightedRelation};
use either::Either;
use emg_core::{im::HashSet, IdStr};
use emg_hasher::CustomHasher;
use emg_state::{Dict, StateAnchor};
use tracing::{debug_span, instrument, warn};
use Either::{Left, Right};

use super::{
    CCSSOpSvv, CCSSSvvOpSvvExpr, CassowaryGeneralMap, CassowaryMap, NameChars, PredEq, PredOp,
    PredVariable, Scope, ScopeViewVariable, StrengthAndWeight,
};

pub(crate) fn eq_opt_sw_to_weighted_relation(
    eq: &PredEq,
    opt_sw: &Option<StrengthAndWeight>,
) -> WeightedRelation {
    let weight = opt_sw
        .as_ref()
        .map_or(cassowary::strength::MEDIUM, |sw| sw.to_number());
    match eq {
        PredEq::Eq => WeightedRelation::EQ(weight),
        PredEq::Lt => todo!(),
        PredEq::Le => WeightedRelation::LE(weight),
        PredEq::Ge => WeightedRelation::GE(weight),
        PredEq::Gt => todo!(),
    }
}

#[instrument(skip(children_cass_maps))]
pub(crate) fn svv_op_svvs_to_expr<Ix>(
    svv_op_svvs: &CCSSSvvOpSvvExpr,
    children_cass_maps: &Dict<Ix, (Rc<CassowaryMap>, StateAnchor<Vec<Constraint>>)>,
    current_cassowary_inherited_generals: &Rc<CassowaryGeneralMap>,
) -> Option<(
    Expression,
    HashSet<Variable, BuildHasherDefault<CustomHasher>>,
)>
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
    svv_to_var(
        main_svv,
        children_cass_maps,
        current_cassowary_inherited_generals,
    )
    .map(|first_var_or_expr| {
        // let mut all_consensus_constraints_sa : Vector<Anchor<Vec<Constraint>>> = vector![first_consensus_constraints_sa];

        let mut child_vars = first_var_or_expr.as_ref().either(
            |var| HashSet::with_hasher(BuildHasherDefault::<CustomHasher>::default()).update(*var),
            |_| HashSet::with_hasher(BuildHasherDefault::<CustomHasher>::default()),
        );
        // let mut child_vars = HashSet::with_hasher(BuildHasherDefault::<CustomHasher>::default()) .update(first_var);

        let expr = op_exprs.into_iter().fold(
            first_var_or_expr.either_into(),
            |exp: Expression, op_expr| {
                let CCSSOpSvv { op, svv } = op_expr;
                match op {
                    PredOp::Add => {
                        if let Some(var_or_expr) = svv_to_var(
                            svv,
                            children_cass_maps,
                            current_cassowary_inherited_generals,
                        ) {
                            // all_consensus_constraints_sa.push_back(consensus_constraints_sa);
                            if let Some(var) = var_or_expr.as_ref().left() {
                                child_vars.insert(*var);
                            }
                            var_or_expr.either_with(exp, |expr, l| expr + l, |expr, r| expr + r)
                        } else {
                            exp
                        }
                    }
                    PredOp::Sub => todo!(),
                    PredOp::Mul => todo!(),
                }
            },
        );
        (expr, child_vars)
    })
}

#[instrument(skip(children_cass_maps))]
fn svv_to_var<Ix>(
    scope_view_variable: &ScopeViewVariable,
    children_cass_maps: &Dict<Ix, (Rc<CassowaryMap>, StateAnchor<Vec<Constraint>>)>,
    current_cassowary_inherited_generals: &Rc<CassowaryGeneralMap>,
) -> Option<Either<Variable, Expression>>
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
        (None, None, Some(PredVariable(prop)))
        | (Some(Scope::Local), None, Some(PredVariable(prop))) => {
            warn!(
                "local => current_cassowary_inherited_generals: {:#?}",
                &current_cassowary_inherited_generals
            );
            warn!("local => current prop {}", &prop);

            if let Some(v) = current_cassowary_inherited_generals.var(prop) {
                // warn!("local => current prop {} v:{:?}", &prop, &v);
                // let (current_var, prop_str) = current_cassowary_inherited_generals
                //     .cassowary_map
                //     .as_ref()
                //     .map(|x| (x.var(prop).unwrap(), x.prop(&v).unwrap()))
                //     .unwrap();

                // warn!(
                //     "local => current prop => var: {:?}, current_var:{:?} , k:{}",
                //     &v, current_var, prop_str
                // );
                Some(Left(v))
            } else {
                // None
                panic!("inherited generals: {} -> not find", prop)
            }
        }
        (None, Some(name), None) => {
            //NOTE no prop
            match name {
                NameChars::Id(_) => todo!(),
                NameChars::Class(_) => todo!(),
                NameChars::Element(_) => todo!(),
                NameChars::Virtual(_) => todo!(),
                NameChars::Number(n) => Some(Right((*n).into())),
                NameChars::Next(_) => todo!(),
                NameChars::Last(_) => todo!(),
                NameChars::First(_) => todo!(),
            }
        }
        (None, Some(name), Some(PredVariable(prop))) => match name {
            NameChars::Id(id) => {
                let _debug_span_ = debug_span!("->[ get child variable ] ").entered();

                warn!("[svv_to_var] parsed scope_view_variable,  find child var : child id:{:?} prop:{:?}",&id,&prop);

                children_cass_maps.get(id.as_str()).map(|(cass_map, ..)| {
                    warn!(
                        "[svv_to_var] got child id:{:?} cass_map: {:?}",
                        &id, &cass_map
                    );

                    let var = cass_map.var(prop).unwrap();

                    Left(var)
                })
            }
            NameChars::Class(_) => todo!(),
            NameChars::Element(_) => todo!(),
            NameChars::Virtual(_) => todo!(),
            NameChars::Number(_) => todo!(),
            NameChars::Next(_) => todo!(),
            NameChars::Last(_) => todo!(),
            NameChars::First(_) => todo!(),
        },
        (Some(_), None, None) => todo!(),
        (Some(s), None, Some(PredVariable(prop))) => {
            match s {
                Scope::Local => unreachable!("should processed in other where (at top)"),
                Scope::Parent(lv) => {
                    scope_parent_val(*lv, prop, current_cassowary_inherited_generals)
                }
                Scope::Global => {
                    if let Some(v) = current_cassowary_inherited_generals.top_var(prop) {
                        warn!("[Scope] global scope, find top var : {:?}", &prop);
                        Some(Left(v))
                    } else {
                        // None
                        panic!("top global generals: {} -> not find", prop)
                    }
                }
            }
        }
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

    if let Some(v) = opt_p.as_ref().and_then(|p| p.var(prop)) {
        Some(Left(v))
    } else {
        panic!("parent {}:{} can't get prop:{}", lv, n, prop)
    }
}
