use emg_common::{SmallVec, Vector};
use tracing::error;

use crate::models::PropertyOG;
use crate::{func::list_find_dup, models::Property, PROP_SIZE};

/*
 * @Author: Rais
 * @Date: 2021-06-02 12:40:24
 * @LastEditTime: 2022-01-25 21:38:39
 * @LastEditors: Rais
 * @Description:
 */

#[must_use]
pub fn is_transformation_og(prop: &PropertyOG) -> bool {
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

pub fn warn_for_double_listed_properties(props: &SmallVec<[Property; PROP_SIZE]>) {
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

pub fn warn_for_double_listed_properties_og(props: &Vector<PropertyOG>) {
    let mut names = props
        .iter()
        .filter(|&prop| !is_transformation_og(prop))
        .map(PropertyOG::name)
        .collect::<Vec<_>>();
    names.sort_unstable();
    let dup = list_find_dup(|a, b| a == b, names.as_slice());
    if !dup.is_empty() {
        error!("{:?}", dup);
    }
    // ────────────────────────────────────────────────────────────────────────────────
}
