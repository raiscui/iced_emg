use im_rc::Vector;
use tracing::error;

use crate::{func::list_find_dup, models::Property};

/*
 * @Author: Rais
 * @Date: 2021-06-02 12:40:24
 * @LastEditTime: 2021-08-29 22:48:19
 * @LastEditors: Rais
 * @Description:
 */

#[must_use]
pub fn is_transformation(prop: &Property) -> bool {
    // List.member (propertyName prop)
    [
        "rotate",
        "rotateX",
        "rotateY",
        "rotateZ",
        "rotate3d",
        "translate",
        "translate3d",
        "scale",
        "scale3d",
    ]
    .contains(&prop.name().as_str())
}

pub fn warn_for_double_listed_properties(props: &Vector<Property>) {
    let mut names = props
        .iter()
        .filter(|&prop| !is_transformation(prop))
        .map(Property::name)
        .collect::<Vec<_>>();
    names.sort_unstable();
    let dup = list_find_dup(|a, b| a == b, names.as_slice());
    if !dup.is_empty() {
        error!("{:?}", dup);
    }
    // ────────────────────────────────────────────────────────────────────────────────
}
