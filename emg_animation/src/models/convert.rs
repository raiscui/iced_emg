use std::rc::Rc;

use emg_core::GenericSize;
use seed_styles::{CssWidth, ExactLength, Percent, Unit};

use crate::init_motion;

use super::{Motion, Property};

/*
 * @Author: Rais
 * @Date: 2021-08-20 12:06:12
 * @LastEditTime: 2021-08-21 18:05:29
 * @LastEditors: Rais
 * @Description:
 */
impl From<ExactLength> for Motion {
    fn from(v: ExactLength) -> Self {
        init_motion(v.value, v.unit)
    }
}

//TODO 确定 定长 不定长 在一起?
#[allow(clippy::fallible_impl_from)]
#[allow(clippy::match_same_arms)]
impl From<Motion> for ExactLength {
    fn from(v: Motion) -> Self {
        match v.unit {
            Unit::Px | Unit::Rem | Unit::Em | Unit::Cm | Unit::Empty => Self {
                unit: v.unit,
                value: v.position,
            },
            Unit::Vw | Unit::Vh | Unit::Pc => Self {
                unit: v.unit,
                value: v.position,
            },
        }
    }
}

impl From<Percent> for Motion {
    fn from(v: Percent) -> Self {
        init_motion(v.0, Unit::Pc)
    }
}

//TODO need implement
#[allow(clippy::fallible_impl_from)]
impl From<CssWidth> for Property {
    fn from(v: CssWidth) -> Self {
        match v {
            CssWidth::Length(l) => Self::Prop(Rc::new("width".to_string()), l.into()),
            CssWidth::Percentage(p) => Self::Prop(Rc::new("width".to_string()), p.into()),
            CssWidth::Auto | CssWidth::Initial | CssWidth::Inherit | CssWidth::StringValue(_) => {
                todo!()
            }
        }
    }
}

// //TODO full this
// #[allow(clippy::fallible_impl_from)]
// impl From<Property> for CssWidth {
//     fn from(v: Property) -> Self {
//         match v {
//             //TODO need implement
//             Property::Prop(name, m) => {
//                 if name.as_str() == "width"
//                     && matches!(
//                         m.unit,
//                         Unit::Px | Unit::Rem | Unit::Em | Unit::Cm | Unit::Empty
//                     )
//                 {
//                     ExactLength::from(m).into()
//                 } else {
//                     panic!(
//                         "propertyName is not width:{}, or unit not match:{:?}",
//                         name.as_str(),
//                         m.unit
//                     );
//                 }
//             }
//             _ => panic!("Property can't convert to CssWidth "),
//         }
//     }
// }
#[allow(clippy::fallible_impl_from)]
impl From<Property> for GenericSize {
    fn from(v: Property) -> Self {
        match v {
            //TODO need implement
            Property::Prop(name, m) => {
                let p_name = name.as_str();
                if (p_name == "width" || p_name == "height")
                    && matches!(
                        m.unit,
                        Unit::Px
                            | Unit::Rem
                            | Unit::Em
                            | Unit::Cm
                            | Unit::Empty
                            //TODO 确定不定长 要不要单独处理
                            | Unit::Vw
                            | Unit::Vh
                            | Unit::Pc
                    )
                {
                    ExactLength::from(m).into()
                } else {
                    panic!(
                        "propertyName is {}, it can't convert to GenericSize",
                        name.as_str()
                    );
                }
            }
            _ => panic!("Property can't convert to GenericSize "),
        }
    }
}
