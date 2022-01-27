use seed_styles::Unit;

use crate::{custom, custom_og};

/*
 * @Author: Rais
 * @Date: 2021-05-10 15:58:45
 * @LastEditTime: 2022-01-26 10:48:52
 * @LastEditors: Rais
 * @Description:
 */
// opacity : Float -> Animation.Model.Property

#[must_use]
pub fn opacity(val: f64) -> super::Property {
    custom("opacity", val, Unit::Empty)
}

#[allow(clippy::module_name_repetitions)]
#[must_use]
pub fn opacity_og(val: f64) -> super::PropertyOG {
    custom_og("opacity", val, Unit::Empty)
}
