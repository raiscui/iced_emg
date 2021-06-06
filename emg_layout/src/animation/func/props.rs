use std::rc::Rc;
use std::time::Duration;

use emg_animation::dbg4;
use emg_animation::models::Interpolation;
use tracing::error;

use crate::animation::define::Property;
use crate::animation::define::PropertyType;

use super::list_find_dup;

/*
 * @Author: Rais
 * @Date: 2021-06-02 10:43:25
 * @LastEditTime: 2021-06-02 11:53:23
 * @LastEditors: Rais
 * @Description:
 */

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
    .contains(prop.name())
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

fn set_default_interpolation(prop: Property) -> Property {
    let interp = default_interpolation_by_property(&prop);

    map_to_motion(
        &move |mut m: Motion| -> Motion {
            m.interpolation = interp.clone();
            m
        },
        prop,
    )
}
fn default_interpolation_by_property(prop: &Property) -> Interpolation {
    use Interpolation::*;
    use Property::*;
    // -- progress is set to 1 because it is changed to 0 when the animation actually starts
    // -- This is analagous to the spring starting at rest.
    let linear = |duration: Duration| {
        Easing(emg_animation::models::Easing {
            progress: 1.,
            start: 0.,
            duration,
            ease: Rc::new(dbg4!(std::convert::identity::<f64>)),
        })
    };

    let default_spring = Spring {
        stiffness: 170.,
        damping: 26.,
    };

    match prop.value_type() {
        PropertyType::SA => default_spring,
        // Anchor(..) | Exact(..) | Shadow(..) | Prop(..) | Prop2(..) | Prop4(..) | Points(..)
        // | Path(..) => default_spring,

        // Color(..) => linear(Duration::from_millis(400)),

        // Prop3(name, ..) => {
        //     if name.as_str() == "rotate3d" {
        //         speed(PI)
        //     } else {
        //         default_spring
        //     }
        // }

        // Angle(_, _) => speed(PI),
    }
}
