/*
 * @Author: Rais
 * @Date: 2021-05-08 15:12:45
 * @LastEditTime: 2021-05-18 15:50:31
 * @LastEditors: Rais
 * @Description:
 */

use im::Vector;
use tracing::error;

use crate::models::{property_name, Property};

// isTransformation : Animation.Model.Property -> Bool
fn is_transformation(prop: &Property) -> bool {
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
    .contains(&property_name(prop))
}

// groupWhile : (a -> a -> Bool) -> List a -> List (List a)
fn list_find_dup<T: Eq>(eq_fn: impl Fn(&T, &T) -> bool, list: &[T]) -> Vec<&T> {
    list.iter()
        .fold((None, Vec::new()), |mut acc, t| match acc.0 {
            Some(t0) if eq_fn(t0, t) => {
                acc.1.push(t);
                acc
            }

            _ => {
                acc.0 = Some(t);
                acc
            }
        })
        .1
}

pub fn warn_for_double_listed_properties(props: &Vector<Property>) -> &Vector<Property> {
    let mut names = props
        .iter()
        .filter(|&prop| !is_transformation(prop))
        .map(property_name)
        .collect::<Vec<_>>();
    names.sort_unstable();
    let dup = list_find_dup(|a, b| a == b, names.as_slice());
    if !dup.is_empty() {
        error!("{:?}", dup);
    }
    // ────────────────────────────────────────────────────────────────────────────────

    props
}
