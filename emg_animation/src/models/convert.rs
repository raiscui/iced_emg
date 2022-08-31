use emg_common::{
    measures::ExactLengthSimplex, GenericSize, TypeCheck, TypeCheckObjectSafe, TypeName,
};
use seed_styles::{CssHeight, CssWidth, LogicLength, Unit};

use crate::init_motion;

use super::PropertyOG;
use super::{Motion, Property};

/*
 * @Author: Rais
 * @Date: 2021-08-20 12:06:12
 * @LastEditTime: 2022-08-30 12:14:56
 * @LastEditors: Rais
 * @Description:
 */

#[allow(clippy::fallible_impl_from)]
impl From<LogicLength> for Motion {
    fn from(l: LogicLength) -> Self {
        match l {
            LogicLength::Simplex(v) => init_motion(v.value, v.unit),
            //TODO calculated_motion
            LogicLength::Calculation(_multiple_unit_l) => todo!("use multiple unit am for each"),
        }
    }
}

#[allow(clippy::fallible_impl_from)]
#[allow(clippy::match_same_arms)]
impl From<Motion> for LogicLength {
    fn from(v: Motion) -> Self {
        match v.unit {
            Unit::Px | Unit::Rem | Unit::Em | Unit::Cm | Unit::Empty => ExactLengthSimplex {
                unit: v.unit,
                value: v.position,
            }
            .into(),
            Unit::Vw | Unit::Vh | Unit::Pc => ExactLengthSimplex {
                unit: v.unit,
                value: v.position,
            }
            .into(),
        }
    }
}

// impl From<Percent> for Motion {
//     fn from(v: Percent) -> Self {
//         init_motion(v.0, Unit::Pc)
//     }
// }

#[allow(clippy::match_same_arms)]
#[allow(clippy::fallible_impl_from)]
impl From<(TypeName, GenericSize)> for PropertyOG {
    fn from((type_name, gs): (TypeName, GenericSize)) -> Self {
        match gs {
            GenericSize::Auto | GenericSize::Initial | GenericSize::Inherit => todo!(),
            GenericSize::Length(l) => Self::Prop(type_name, l.into()),
            GenericSize::StringValue(_) => todo!(),
            GenericSize::Calculation(_) => todo!(),
            GenericSize::Parent(_) => todo!(),
            GenericSize::None => todo!(),
        }
    }
}
#[allow(clippy::match_same_arms)]
#[allow(clippy::fallible_impl_from)]
impl From<(TypeName, GenericSize)> for Property {
    fn from((type_name, gs): (TypeName, GenericSize)) -> Self {
        match gs {
            GenericSize::Auto | GenericSize::Initial | GenericSize::Inherit => todo!(),
            GenericSize::Length(l) => Self::Prop(type_name, l.into()),
            GenericSize::StringValue(_) => todo!(),
            GenericSize::Calculation(_) => todo!(),
            GenericSize::Parent(_) => todo!(),
            GenericSize::None => todo!(),
        }
    }
}

//TODO need implement
#[allow(clippy::fallible_impl_from)]
impl From<CssWidth> for PropertyOG {
    fn from(v: CssWidth) -> Self {
        let type_name = CssWidth::TYPE_NAME;
        match v {
            CssWidth::Gs(gs) => (type_name, gs).into(),
            CssWidth::Length(l) => Self::Prop(type_name, l.into()),
            CssWidth::Auto | CssWidth::Initial | CssWidth::Inherit | CssWidth::StringValue(_) => {
                todo!()
            }
        }
    }
}

#[allow(clippy::fallible_impl_from)]
impl From<CssHeight> for PropertyOG {
    fn from(v: CssHeight) -> Self {
        let type_name = CssHeight::TYPE_NAME;
        match v {
            CssHeight::Gs(gs) => (type_name, gs).into(),
            CssHeight::Length(l) => Self::Prop(type_name, l.into()),
            CssHeight::Auto
            | CssHeight::Initial
            | CssHeight::Inherit
            | CssHeight::StringValue(_) => {
                todo!()
            }
        }
    }
}
#[allow(clippy::fallible_impl_from)]
impl From<CssWidth> for Property {
    fn from(v: CssWidth) -> Self {
        let type_name = CssWidth::TYPE_NAME;
        match v {
            CssWidth::Gs(gs) => (type_name, gs).into(),
            CssWidth::Length(l) => Self::Prop(type_name, l.into()),
            CssWidth::Auto | CssWidth::Initial | CssWidth::Inherit | CssWidth::StringValue(_) => {
                todo!()
            }
        }
    }
}
#[allow(clippy::fallible_impl_from)]
impl From<CssHeight> for Property {
    fn from(v: CssHeight) -> Self {
        let type_name = CssHeight::TYPE_NAME;
        match v {
            CssHeight::Gs(gs) => (type_name, gs).into(),
            CssHeight::Length(l) => Self::Prop(type_name, l.into()),
            CssHeight::Auto
            | CssHeight::Initial
            | CssHeight::Inherit
            | CssHeight::StringValue(_) => {
                todo!()
            }
        }
    }
}

// //TODO full this

#[allow(clippy::fallible_impl_from)]
impl From<Property> for CssWidth {
    fn from(v: Property) -> Self {
        // panic!("check here");
        match v {
            //TODO need implement
            Property::Prop(name, m) => {
                if name.as_str() == Self::TYPE_NAME.as_str()
                    && matches!(
                        m.unit,
                        Unit::Px | Unit::Rem | Unit::Em | Unit::Cm | Unit::Empty
                    )
                {
                    LogicLength::from(m).into()
                } else {
                    panic!(
                        "propertyName is not width:{}, or unit not match:{:?}",
                        name.as_str(),
                        m.unit
                    );
                }
            }
            _ => panic!("Property can't convert to CssWidth "),
        }
    }
}

#[allow(clippy::fallible_impl_from)]
impl From<Property> for GenericSize {
    fn from(v: Property) -> Self {
        match v {
            //TODO need implement
            Property::Prop(p_name, m) => {
                if (p_name == CssWidth::TYPE_NAME || p_name == CssHeight::TYPE_NAME)
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
                    LogicLength::from(m).into()
                } else {
                    panic!(
                        "propertyName is {}, it can't convert to GenericSize",
                        p_name.as_str()
                    );
                }
            }
            _ => panic!("Property can't convert to GenericSize "),
        }
    }
}
// ────────────────────────────────────────────────────────────────────────────────

#[allow(clippy::fallible_impl_from)]
impl From<PropertyOG> for CssWidth {
    fn from(v: PropertyOG) -> Self {
        // panic!("check here");
        match v {
            //TODO need implement
            PropertyOG::Prop(name, m) => {
                if name.as_str() == Self::TYPE_NAME.as_str()
                    && matches!(
                        m.unit,
                        Unit::Px | Unit::Rem | Unit::Em | Unit::Cm | Unit::Empty
                    )
                {
                    LogicLength::from(m).into()
                } else {
                    panic!(
                        "propertyName is not width:{}, or unit not match:{:?}",
                        name.as_str(),
                        m.unit
                    );
                }
            }
            _ => panic!("Property can't convert to CssWidth "),
        }
    }
}

#[allow(clippy::fallible_impl_from)]
impl From<PropertyOG> for GenericSize {
    fn from(v: PropertyOG) -> Self {
        match v {
            //TODO need implement
            PropertyOG::Prop(p_name, m) => {
                if (p_name == CssWidth::TYPE_NAME || p_name == CssHeight::TYPE_NAME)
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
                    LogicLength::from(m).into()
                } else {
                    panic!(
                        "propertyName is {}, it can't convert to GenericSize",
                        p_name.as_str()
                    );
                }
            }
            _ => panic!("Property can't convert to GenericSize "),
        }
    }
}
