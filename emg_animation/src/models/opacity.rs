use seed_styles::Unit;

use crate::custom;

/*
 * @Author: Rais
 * @Date: 2021-05-10 15:58:45
 * @LastEditTime: 2022-01-07 17:02:57
 * @LastEditors: Rais
 * @Description:
 */
// opacity : Float -> Animation.Model.Property
#[must_use]
pub fn opacity(val: f64) -> super::Property {
    custom("opacity", val, Unit::Empty)
}
