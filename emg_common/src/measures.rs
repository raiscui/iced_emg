// use crate::style::css_values::*;
use derive_more::Display;
use derive_more::From;
use num_traits::AsPrimitive;
use ordered_float::NotNan;

use crate::{CalcOp, Precision};

#[derive(Display, Clone, Debug, From, PartialEq, PartialOrd, Eq)]
#[display(fmt = "{}")]
pub enum LogicLength {
    #[display(fmt = "{}", _0)]
    Simplex(ExactLengthSimplex),
    #[display(fmt = "{}", _0)]
    ///单位绝对不同
    Calculation(Box<CalcOp<LogicLength>>),
}

impl LogicLength {
    pub fn get_unit(&self) -> Option<crate::measures::Unit> {
        match self {
            Self::Simplex(v) => Some(v.unit),
            Self::Calculation(_) => None,
        }
    }
    pub fn has_add_unit(&self, unit: crate::measures::Unit) -> bool {
        match self {
            Self::Simplex(v) => v.unit == unit,
            Self::Calculation(v) => v.has_add_unit(unit),
        }
    }
    #[inline]
    ///this is directly no optimization op
    fn add_directly(l: Self, r: Self) -> Self {
        LogicLength::Calculation(Box::new(CalcOp::add(l, r)))
    }

    pub fn try_into_simplex(self) -> Result<ExactLengthSimplex, Self> {
        if let Self::Simplex(v) = self {
            Ok(v)
        } else {
            Err(self)
        }
    }

    /// Returns `true` if the logic length is [`Simplex`].
    ///
    /// [`Simplex`]: LogicLength::Simplex
    #[must_use]
    pub fn is_simplex(&self) -> bool {
        matches!(self, Self::Simplex(..))
    }
}
impl Default for LogicLength {
    fn default() -> Self {
        LogicLength::Simplex(ExactLengthSimplex::default())
    }
}

#[derive(Display, Copy, Clone, Debug, PartialEq, Eq, PartialOrd)]
#[display(fmt = "{}{}", value, unit)]
pub struct ExactLengthSimplex {
    pub unit: Unit,
    pub value: NotNan<Precision>,
}

impl core::ops::Add for ExactLengthSimplex {
    type Output = LogicLength;

    fn add(self, rhs: Self) -> Self::Output {
        if self.unit == rhs.unit {
            LogicLength::Simplex(ExactLengthSimplex {
                unit: self.unit,
                value: self.value + rhs.value,
            })
        } else {
            LogicLength::add_directly(self.into(), rhs.into())
        }
    }
}
impl core::ops::Add<&Self> for ExactLengthSimplex {
    type Output = LogicLength;

    fn add(self, rhs: &Self) -> Self::Output {
        if self.unit == rhs.unit {
            LogicLength::Simplex(ExactLengthSimplex {
                unit: self.unit,
                value: self.value + rhs.value,
            })
        } else {
            LogicLength::add_directly(self.into(), (*rhs).into())
        }
    }
}
impl Default for ExactLengthSimplex {
    fn default() -> Self {
        ExactLengthSimplex {
            unit: Unit::Px,
            value: NotNan::new(0.0).unwrap(),
        }
    }
}

impl ExactLengthSimplex {
    /// Get a reference to the exact length's value.
    pub fn value(&self) -> Precision {
        self.value.into_inner()
    }
}

// impl From<ExactLengthSimplex> for NotNan<f64> {
//     fn from(v: ExactLengthSimplex) -> Self {
//         v.value
//     }
// }

//TODO full this
impl<T> ::core::ops::Mul<T> for LogicLength
where
    T: AsPrimitive<Precision>,
{
    type Output = LogicLength;
    fn mul(self, rhs: T) -> LogicLength {
        Self::Calculation(Box::new(CalcOp::mul(self, rhs.as_())))
    }
}

impl ::core::ops::Add for LogicLength {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        match (self, rhs) {
            (Self::Simplex(l), Self::Simplex(r)) => l + r,
            (Self::Simplex(l), calculation) => calculation + l,
            (calculation, Self::Simplex(r)) => calculation + r,
            (Self::Calculation(l), Self::Calculation(r)) => {
                Self::add_directly(Self::Calculation(l), Self::Calculation(r))
            }
        }
    }
}
impl ::core::ops::Add<&Self> for LogicLength {
    type Output = Self;
    fn add(self, rhs: &Self) -> Self {
        match (self, rhs) {
            (Self::Simplex(l), Self::Simplex(r)) => l + r,
            (Self::Simplex(l), calculation) => calculation.clone() + l,
            (calculation, Self::Simplex(r)) => calculation + *r,
            (Self::Calculation(l), Self::Calculation(r)) => {
                Self::add_directly(Self::Calculation(l), Self::Calculation(r.clone()))
            }
        }
    }
}
impl ::core::ops::Add<ExactLengthSimplex> for LogicLength {
    type Output = Self;

    //TODO optimization mul like this
    fn add(self, rhs: ExactLengthSimplex) -> Self {
        match self {
            Self::Simplex(l) => l + rhs,
            Self::Calculation(x) => {
                if x.has_add_unit(rhs.unit) {
                    match *x {
                        CalcOp::Add { a, b } => {
                            //
                            if a.has_add_unit(rhs.unit) {
                                return a + rhs + b;
                            }
                            if b.has_add_unit(rhs.unit) {
                                return a + (b + rhs);
                            }
                            unreachable!("...");
                        }

                        CalcOp::Mul { .. } => {
                            unreachable!("...");
                        }
                    }
                } else {
                    Self::add_directly(Self::Calculation(x), Self::Simplex(rhs))
                }
            }
        }
    }
}

#[cfg(test)]
mod logic_length_test {
    use ordered_float::NotNan;

    use crate::{pc, px, vh, CalcOp, ExactLengthSimplex, LogicLength, Unit};

    #[test]
    fn add_test() {
        let a: LogicLength = px(11);
        let b: LogicLength = pc(100);
        let end = a + b;
        // let res assert  match eq the end
        let res = LogicLength::Calculation(Box::new(CalcOp::Add {
            a: LogicLength::Simplex(ExactLengthSimplex {
                unit: Unit::Px,
                value: NotNan::new(11.0).unwrap(),
            }),
            b: LogicLength::Simplex(ExactLengthSimplex {
                unit: Unit::Pc,
                value: NotNan::new(100.0).unwrap(),
            }),
        }));
        assert_eq!(end, res);

        let c = vh(20);
        let end = end + c;
        // let res assert  match eq the end
        let res = LogicLength::Calculation(Box::new(CalcOp::Add {
            a: LogicLength::Calculation(Box::new(CalcOp::Add {
                a: LogicLength::Simplex(ExactLengthSimplex {
                    unit: Unit::Px,
                    value: NotNan::new(11.0).unwrap(),
                }),
                b: LogicLength::Simplex(ExactLengthSimplex {
                    unit: Unit::Pc,
                    value: NotNan::new(100.0).unwrap(),
                }),
            })),
            b: LogicLength::Simplex(ExactLengthSimplex {
                unit: Unit::Vh,
                value: NotNan::new(20.0).unwrap(),
            }),
        }));
        assert_eq!(end, res);

        let c = pc(200);
        let end = end + c;
        // let res assert  match eq the end
        let res = LogicLength::Calculation(Box::new(CalcOp::Add {
            a: LogicLength::Calculation(Box::new(CalcOp::Add {
                a: LogicLength::Simplex(ExactLengthSimplex {
                    unit: Unit::Px,
                    value: NotNan::new(11.0).unwrap(),
                }),
                b: LogicLength::Simplex(ExactLengthSimplex {
                    unit: Unit::Pc,
                    value: NotNan::new(300.0).unwrap(),
                }),
            })),
            b: LogicLength::Simplex(ExactLengthSimplex {
                unit: Unit::Vh,
                value: NotNan::new(20.0).unwrap(),
            }),
        }));
        assert_eq!(end, res);

        let c = px(10);
        let end = end + c;
        // let res assert  match eq the end
        let res = LogicLength::Calculation(Box::new(CalcOp::Add {
            a: LogicLength::Calculation(Box::new(CalcOp::Add {
                a: LogicLength::Simplex(ExactLengthSimplex {
                    unit: Unit::Px,
                    value: NotNan::new(21.0).unwrap(),
                }),
                b: LogicLength::Simplex(ExactLengthSimplex {
                    unit: Unit::Pc,
                    value: NotNan::new(300.0).unwrap(),
                }),
            })),
            b: LogicLength::Simplex(ExactLengthSimplex {
                unit: Unit::Vh,
                value: NotNan::new(20.0).unwrap(),
            }),
        }));
        assert_eq!(end, res);

        let end = end.clone() + end;
        // println!("{end:?}");

        let a: LogicLength = px(11);
        let b: LogicLength = pc(100);
        let end = a + b;
        let end = end.clone() + end;
        // println!("{end:?}");
        let a: LogicLength = px(11);
        let end = end + a;
        println!("{end:?}");
    }
}

impl LogicLength {
    pub fn try_get_number(&self) -> Result<Precision, &Self> {
        match self {
            LogicLength::Simplex(l) => match l.unit {
                Unit::Px | Unit::Empty => Ok(l.value.into_inner()),
                _ => Err(self),
            },
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

pub fn px<T: AsPrimitive<Precision>>(val: T) -> LogicLength {
    ExactLengthSimplex {
        value: NotNan::new(val.as_()).unwrap(),
        unit: Unit::Px,
    }
    .into()
}

pub fn vw<T: AsPrimitive<Precision>>(val: T) -> LogicLength {
    ExactLengthSimplex {
        value: NotNan::new(val.as_()).unwrap(),
        unit: Unit::Vw,
    }
    .into()
}

pub fn vh<T: AsPrimitive<Precision>>(val: T) -> LogicLength {
    ExactLengthSimplex {
        value: NotNan::new(val.as_()).unwrap(),
        unit: Unit::Vh,
    }
    .into()
}

pub fn cm<T: AsPrimitive<Precision>>(val: T) -> LogicLength {
    ExactLengthSimplex {
        value: NotNan::new(val.as_()).unwrap(),
        unit: Unit::Cm,
    }
    .into()
}

pub fn rem<T: AsPrimitive<Precision>>(val: T) -> LogicLength {
    ExactLengthSimplex {
        value: NotNan::new(val.as_()).unwrap(),
        unit: Unit::Rem,
    }
    .into()
}

pub fn em<T: AsPrimitive<Precision>>(val: T) -> LogicLength {
    ExactLengthSimplex {
        value: NotNan::new(val.as_()).unwrap(),
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
pub fn pc<T: AsPrimitive<Precision>>(val: T) -> LogicLength {
    ExactLengthSimplex {
        value: NotNan::new(val.as_()).unwrap(),
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
