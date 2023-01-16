use emg_animation::{func::list_find_dup, props::is_transformation, PROP_SIZE};
use emg_common::SmallVec;
use tracing::error;

use crate::animation::StateVarProperty;

pub fn warn_for_double_listed_properties(props: &SmallVec<[StateVarProperty; PROP_SIZE]>) {
    let mut names = props
        .iter()
        .filter_map(|prop| {
            prop.get_with(|p| {
                if is_transformation(p) {
                    None
                } else {
                    Some(p.name())
                }
            })
        })
        .collect::<Vec<_>>();
    names.sort_unstable();
    // names.dedup();
    let dup = list_find_dup(|a, b| a == b, names.as_slice());
    if !dup.is_empty() {
        error!("{:?}", dup);
    }
    // ────────────────────────────────────────────────────────────────────────────────
}
