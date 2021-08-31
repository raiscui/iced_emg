use emg_animation::func::list_find_dup;
use emg_animation::props::is_transformation;
use emg_state::CloneStateVar;
use im_rc::Vector;
use tracing::error;

use crate::animation::StateVarProperty;

pub fn warn_for_double_listed_properties(props: &Vector<StateVarProperty>) {
    let mut names = props
        .iter()
        .filter_map(|prop| {
            let p = prop.get();
            if is_transformation(&p) {
                None
            } else {
                Some(p.name())
            }
        })
        .collect::<Vec<_>>();
    names.sort_unstable();
    let dup = list_find_dup(|a, b| a == b, names.as_slice());
    if !dup.is_empty() {
        error!("{:?}", dup);
    }
    // ────────────────────────────────────────────────────────────────────────────────
}
