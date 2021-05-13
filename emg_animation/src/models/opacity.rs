use crate::custom;

/*
 * @Author: Rais
 * @Date: 2021-05-10 15:58:45
 * @LastEditTime: 2021-05-10 16:06:54
 * @LastEditors: Rais
 * @Description:
 */
// opacity : Float -> Animation.Model.Property
pub fn opacity(val: f64) -> super::Property {
    custom("opacity".to_string(), val, "".to_string())
}
