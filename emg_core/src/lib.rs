mod func;
pub mod measures;
use measures::{px, ExactLength, Percent};

// ────────────────────────────────────────────────────────────────────────────────

use derive_more::Display;
use derive_more::From;
use ordered_float::NotNan;
// // use derive_more::Into;
#[derive(Display, Clone, Debug, PartialEq, PartialOrd, Eq)]
pub enum CalcOp<T>
where
    T: Clone + std::fmt::Debug + PartialEq + PartialOrd + Eq,
{
    #[display(fmt = "{} + {}", a, b)]
    Add { a: T, b: T },
    #[display(fmt = "{} * {}", a, b)]
    Mul { a: T, b: NotNan<f64> },
}

impl<T> CalcOp<T>
where
    T: Clone + std::fmt::Debug + PartialEq + PartialOrd + Eq,
{
    pub fn add(a: T, b: T) -> Self {
        Self::Add { a, b }
    }
    pub fn mul(a: T, b: f64) -> Self {
        Self::Mul {
            a,
            b: NotNan::new(b).unwrap(),
        }
    }
}

#[derive(Display, Clone, Debug, From, PartialEq, PartialOrd, Eq)]
#[display(fmt = "{}")]
pub enum GenericSize {
    #[display(fmt = "auto")]
    Auto,
    Length(ExactLength),
    Percentage(Percent),
    #[display(fmt = "initial")]
    Initial,
    #[display(fmt = "inherit")]
    Inherit,
    StringValue(String),
    Calculation(Box<CalcOp<GenericSize>>),
}

impl ::core::ops::Mul<f64> for GenericSize {
    type Output = GenericSize;
    fn mul(self, rhs: f64) -> GenericSize {
        Self::Calculation(Box::new(CalcOp::mul(self, rhs)))
    }
}
impl ::core::ops::Add for GenericSize {
    type Output = GenericSize;
    fn add(self, rhs: GenericSize) -> GenericSize {
        Self::Calculation(Box::new(CalcOp::add(self, rhs)))
    }
}

impl GenericSize {
    #[must_use]
    pub fn get_length_value(&self) -> f64 {
        self.try_get_length_value()
            .expect("directly get length value failed, expected Length Px struct")
    }

    /// # Errors
    ///
    /// Will return `Err` if `self` does not `Length` and `Length`  unit is not px
    pub fn try_get_length_value(&self) -> Result<f64, &Self> {
        self.as_length()
            .and_then(|l| l.try_get_number().ok())
            .ok_or(self)
    }

    #[must_use]
    pub const fn as_length(&self) -> Option<&ExactLength> {
        if let Self::Length(v) = self {
            Some(v)
        } else {
            None
        }
    }
}
impl Default for GenericSize {
    fn default() -> Self {
        Self::Length(px(0))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
