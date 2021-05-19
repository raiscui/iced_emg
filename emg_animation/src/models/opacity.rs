use crate::custom;

/*
 * @Author: Rais
 * @Date: 2021-05-10 15:58:45
 * @LastEditTime: 2021-05-19 14:46:10
 * @LastEditors: Rais
 * @Description:
 */
// opacity : Float -> Animation.Model.Property
#[must_use]
pub fn opacity(val: f64) -> super::Property {
    custom("opacity".to_string(), val, "".to_string())
}
