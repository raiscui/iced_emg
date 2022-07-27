use std::rc::Rc;

use cassowary::Expression;
use emg_core::GenericSize;
use seed_styles as styles;
use seed_styles::LogicLength;

use crate::ccsa::CassowaryMap;

/*
 * @Author: Rais
 * @Date: 2022-07-15 14:46:40
 * @LastEditTime: 2022-07-21 17:34:28
 * @LastEditors: Rais
 * @Description:
 */

pub fn cassowary_calculation_logiclength(
    prop: &str,
    p_cass_map: &Rc<CassowaryMap>,
    l: &LogicLength,
) -> Expression {
    match l {
        LogicLength::Simplex(els) => {
            let v = els.value();
            match els.unit {
                styles::Unit::Px | styles::Unit::Empty => v.into(),
                styles::Unit::Rem
                | styles::Unit::Em
                | styles::Unit::Cm
                | styles::Unit::Vw
                | styles::Unit::Vh => {
                    todo!()
                }
                styles::Unit::Pc => (p_cass_map.var(prop).unwrap() * (v * 0.01)).into(),
            }
        }
        LogicLength::Calculation(calc_op) => match calc_op.as_ref() {
            emg_core::CalcOp::Mul { a, b } => {
                cassowary_calculation_logiclength(prop, p_cass_map, a) * b.into_inner()
            }
            emg_core::CalcOp::Add { a, b } => {
                cassowary_calculation_logiclength(prop, p_cass_map, a)
                    + cassowary_calculation_logiclength(prop, p_cass_map, b)
            }
        },
    }
}

pub fn cassowary_calculation(
    prop: &str,
    p_cass_map: &Rc<CassowaryMap>,
    w: &GenericSize,
) -> Expression {
    match w {
        GenericSize::None => unreachable!("should used [is_none()] before call this function"),
        GenericSize::Length(logic_l) => {
            cassowary_calculation_logiclength(prop, p_cass_map, logic_l)
        }
        // GenericSize::Percentage(pc) => p_cass_map.x * pc.value()*0.01,
        GenericSize::Auto
        | GenericSize::Initial
        | GenericSize::Inherit
        | GenericSize::StringValue(_) => {
            todo!()
        }
        GenericSize::Calculation(calc_op) => match calc_op.as_ref() {
            emg_core::CalcOp::Mul { a, b } => {
                cassowary_calculation(prop, p_cass_map, a) * b.into_inner()
            }
            emg_core::CalcOp::Add { a, b } => {
                cassowary_calculation(prop, p_cass_map, a)
                    + cassowary_calculation(prop, p_cass_map, b)
            }
        },
        //TODO 实现 parent 的parent 需要 p_cass_map 保存 parent的 p_cass_map
        GenericSize::Parent(type_name) => match type_name.as_str() {
            "CssWidth" => p_cass_map.var("width").unwrap().into(),
            "CssHeight" => p_cass_map.var("height").unwrap().into(),
            other => {
                panic!("current not implemented for GenericSize::Parent({})", other);
            }
        },
    }
}
