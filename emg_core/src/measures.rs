// use crate::style::css_values::*;
use derive_more::Display;
use derive_more::From;
use ordered_float::NotNan;

use crate::CalcOp;

#[derive(Display, Clone, Debug, From, PartialEq, PartialOrd, Eq)]
#[display(fmt = "{}")]
pub enum LogicLength {
    #[display(fmt = "{}", _0)]
    Simplex(ExactLengthSimplex),
    #[display(fmt = "{}", _0)]
    Calculation(Box<CalcOp<LogicLength>>),
}

#[derive(Display, Copy, Clone, Debug, PartialEq, Eq, PartialOrd)]
#[display(fmt = "{}{}", value, unit)]
pub struct ExactLengthSimplex {
    pub unit: Unit,
    pub value: NotNan<f64>,
}

impl ExactLengthSimplex {
    /// Get a reference to the exact length's value.
    pub fn value(&self) -> f64 {
        self.value.into_inner()
    }
}

// impl From<ExactLengthSimplex> for NotNan<f64> {
//     fn from(v: ExactLengthSimplex) -> Self {
//         v.value
//     }
// }

//TODO full this
impl ::core::ops::Mul<f64> for LogicLength {
    type Output = LogicLength;
    fn mul(self, rhs: f64) -> LogicLength {
        Self::Calculation(Box::new(CalcOp::mul(self, rhs)))
    }
}
impl ::core::ops::Add for LogicLength {
    type Output = LogicLength;
    fn add(self, rhs: LogicLength) -> LogicLength {
        Self::Calculation(Box::new(CalcOp::add(self, rhs)))
    }
}

// impl<__RhsT> ::core::ops::Mul<__RhsT> for ExactLength
// where
//     f64: ::core::ops::Mul<__RhsT, Output = f64>,
// {
//     type Output = ExactLength;
//     fn mul(self, rhs: __RhsT) -> ExactLength {
//         Self {
//             unit: self.unit,
//             value: NotNan::new(self.value.into_inner().mul(rhs)).expect("mut ExactLength"),
//         }
//     }
// }
// impl ::core::ops::Add for ExactLength {
//     type Output = ExactLength;
//     fn add(self, rhs: ExactLength) -> ExactLength {
//         if self.unit != rhs.unit {
//             panic!("can't add two ExactLength has different units")
//         }
//         Self {
//             unit: self.unit,
//             value: self.value.add(rhs.value),
//         }
//     }
// }

impl LogicLength {
    pub fn try_get_number(&self) -> Result<f64, &Self> {
        match self {
            LogicLength::Simplex(l) => {
                if matches!(l.unit, Unit::Px | Unit::Empty) {
                    Ok(l.value.into_inner())
                } else {
                    Err(self)
                }
            }
            LogicLength::Calculation(_) => Err(self),
        }
    }
}

#[derive(Display, Copy, Clone, Debug, PartialEq, Eq, PartialOrd)]
pub enum Unit {
    #[display(fmt = "px")]
    Px,
    #[display(fmt = "rem")]
    Rem,
    #[display(fmt = "em")]
    Em,
    #[display(fmt = "cm")]
    Cm,
    #[display(fmt = "vw")]
    Vw,
    #[display(fmt = "vh")]
    Vh,
    #[display(fmt = "%")]
    Pc,
    #[display(fmt = "")]
    Empty,
}

pub fn px<T: Into<f64>>(val: T) -> LogicLength {
    ExactLengthSimplex {
        value: NotNan::new(val.into()).unwrap(),
        unit: Unit::Px,
    }
    .into()
}

pub fn vw<T: Into<f64>>(val: T) -> LogicLength {
    ExactLengthSimplex {
        value: NotNan::new(val.into()).unwrap(),
        unit: Unit::Vw,
    }
    .into()
}

pub fn vh<T: Into<f64>>(val: T) -> LogicLength {
    ExactLengthSimplex {
        value: NotNan::new(val.into()).unwrap(),
        unit: Unit::Vh,
    }
    .into()
}

pub fn cm<T: Into<f64>>(val: T) -> LogicLength {
    ExactLengthSimplex {
        value: NotNan::new(val.into()).unwrap(),
        unit: Unit::Cm,
    }
    .into()
}

pub fn rem<T: Into<f64>>(val: T) -> LogicLength {
    ExactLengthSimplex {
        value: NotNan::new(val.into()).unwrap(),
        unit: Unit::Rem,
    }
    .into()
}

pub fn em<T: Into<f64>>(val: T) -> LogicLength {
    ExactLengthSimplex {
        value: NotNan::new(val.into()).unwrap(),
        unit: Unit::Em,
    }
    .into()
}

// impl std::fmt::Display for ExactLength {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self.unit {
//             Unit::Px => write!(f, "{}px", self.value),
//             Unit::Rem => write!(f, "{}rem", self.value),
//             Unit::Em => write!(f, "{}em", self.value),
//             Unit::Cm => write!(f, "{}cm", self.value),
//             Unit::Vw => write!(f, "{}vw", self.value),
//             Unit::Vh => write!(f, "{}vh", self.value),

//             Unit::Empty => write!(f, "{}", self.value),
//             Unit::Pc => write!(f, "{}%", self.value),
//         }
//     }
// }

// #[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
// pub struct Percent(pub NotNan<f64>);

// impl Percent {
//     pub fn value(&self) -> f64 {
//         self.0.into_inner()
//     }
// }

// impl<__RhsT> ::core::ops::Mul<__RhsT> for Percent
// where
//     f64: ::core::ops::Mul<__RhsT, Output = f64>,
// {
//     type Output = Percent;
//     fn mul(self, rhs: __RhsT) -> Percent {
//         Self(NotNan::new(self.0.into_inner().mul(rhs)).expect("mut Percent"))
//     }
// }

// impl ::core::ops::Add for Percent {
//     type Output = Percent;
//     fn add(self, rhs: Percent) -> Percent {
//         Self(self.0.add(rhs.0))
//     }
// }
pub fn pc<T: Into<f64>>(val: T) -> LogicLength {
    ExactLengthSimplex {
        value: NotNan::new(val.into()).unwrap(),
        unit: Unit::Pc,
    }
    .into()
}

// pub fn pc<T: Into<f64>>(val: T) -> Percent {
//     Percent(NotNan::new(val.into()).unwrap())
// }

// impl std::fmt::Display for Percent {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{}%", self.0.into_inner())
//     }
// }

#[cfg(test)]
mod tests {

    use crate::measures::pc;

    #[test]
    fn mul() {
        let a = pc(11);
        let c = a * 4.;

        println!("c:{}", c)
    }
}
