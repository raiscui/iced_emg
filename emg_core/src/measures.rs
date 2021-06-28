// use crate::style::css_values::*;
use ordered_float::NotNan;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd)]
pub struct ExactLength {
    pub unit: Unit,
    pub value: NotNan<f64>,
}

impl<__RhsT> ::core::ops::Mul<__RhsT> for ExactLength
where
    f64: ::core::ops::Mul<__RhsT, Output = f64>,
{
    type Output = ExactLength;
    fn mul(self, rhs: __RhsT) -> ExactLength {
        Self {
            unit: self.unit,
            value: NotNan::new(self.value.into_inner().mul(rhs)).expect("mut ExactLength"),
        }
    }
}
impl ::core::ops::Add for ExactLength {
    type Output = ExactLength;
    fn add(self, rhs: ExactLength) -> ExactLength {
        if self.unit != rhs.unit {
            panic!("can't add two ExactLength has different units")
        }
        Self {
            unit: self.unit,
            value: self.value.add(rhs.value),
        }
    }
}

impl ExactLength {
    pub fn try_get_number(&self) -> Result<f64, &Self> {
        if matches!(self.unit, Unit::Px | Unit::None) {
            Ok(self.value.into_inner())
        } else {
            Err(self)
        }
    }

    /// Get a reference to the exact length's value.
    pub fn value(&self) -> f64 {
        self.value.into_inner()
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd)]
pub enum Unit {
    Px,
    Rem,
    Em,
    Cm,
    Vw,
    Vh,
    Pc,
    None,
}

pub fn px<T: Into<f64>>(val: T) -> ExactLength {
    ExactLength {
        value: NotNan::new(val.into()).unwrap(),
        unit: Unit::Px,
    }
}

pub fn vw<T: Into<f64>>(val: T) -> ExactLength {
    ExactLength {
        value: NotNan::new(val.into()).unwrap(),
        unit: Unit::Vw,
    }
}

pub fn vh<T: Into<f64>>(val: T) -> ExactLength {
    ExactLength {
        value: NotNan::new(val.into()).unwrap(),
        unit: Unit::Vh,
    }
}

pub fn cm<T: Into<f64>>(val: T) -> ExactLength {
    ExactLength {
        value: NotNan::new(val.into()).unwrap(),
        unit: Unit::Cm,
    }
}

pub fn rem<T: Into<f64>>(val: T) -> ExactLength {
    ExactLength {
        value: NotNan::new(val.into()).unwrap(),
        unit: Unit::Rem,
    }
}

pub fn em<T: Into<f64>>(val: T) -> ExactLength {
    ExactLength {
        value: NotNan::new(val.into()).unwrap(),
        unit: Unit::Em,
    }
}

impl std::fmt::Display for ExactLength {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.unit {
            Unit::Px => write!(f, "{}px", self.value),
            Unit::Rem => write!(f, "{}rem", self.value),
            Unit::Em => write!(f, "{}em", self.value),
            Unit::Cm => write!(f, "{}cm", self.value),
            Unit::Vw => write!(f, "{}vw", self.value),
            Unit::Vh => write!(f, "{}vh", self.value),

            Unit::None => write!(f, "{}", self.value),
            Unit::Pc => unreachable!(),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct Percent(pub NotNan<f64>);

impl Percent {
    pub fn value(&self) -> f64 {
        self.0.into_inner()
    }
}

impl<__RhsT> ::core::ops::Mul<__RhsT> for Percent
where
    f64: ::core::ops::Mul<__RhsT, Output = f64>,
{
    type Output = Percent;
    fn mul(self, rhs: __RhsT) -> Percent {
        Self(NotNan::new(self.0.into_inner().mul(rhs)).expect("mut Percent"))
    }
}

impl ::core::ops::Add for Percent {
    type Output = Percent;
    fn add(self, rhs: Percent) -> Percent {
        Self(self.0.add(rhs.0))
    }
}

pub fn pc<T: Into<f64>>(val: T) -> Percent {
    Percent(NotNan::new(val.into()).unwrap())
}

impl std::fmt::Display for Percent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}%", self.0.into_inner())
    }
}

#[cfg(test)]
mod tests {

    use crate::measures::pc;

    #[test]
    fn mul() {
        let a = pc(11);
        let c = a * 4.;

        println!("c:{:?}", c)
    }
}
