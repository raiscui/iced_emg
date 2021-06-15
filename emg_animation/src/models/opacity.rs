use seed_styles::Unit;

use crate::custom;

/*
 * @Author: Rais
 * @Date: 2021-05-10 15:58:45
 * @LastEditTime: 2021-06-14 22:21:44
 * @LastEditors: Rais
 * @Description:
 */
// opacity : Float -> Animation.Model.Property
#[must_use]
pub fn opacity(val: f64) -> super::Property {
    custom("opacity".to_string(), val, Unit::None)
}
