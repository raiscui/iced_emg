mod func;
mod macros;
pub mod measures;
use measures::{px, LogicLength};
use measures::{ExactLengthSimplex, Unit};

// ────────────────────────────────────────────────────────────────────────────────

use derive_more::Display;
use derive_more::From;
pub use im_rc;
pub use im_rc::vector;
pub use im_rc::Vector;
use ordered_float::NotNan;

// ────────────────────────────────────────────────────────────────────────────────
pub trait TypeCheck {
    fn static_type_name() -> TypeName;
}
pub trait TypeCheckObjectSafe {
    fn type_name(&self) -> TypeName;
}

// // use derive_more::Into;
//TODO full this  "-,/"
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

#[derive(Display, Clone, Debug, PartialEq, PartialOrd, Eq)]
pub struct TypeName(String);

impl TypeName {
    pub fn new<T>(name: T) -> Self
    where
        T: Into<String>,
    {
        Self(name.into())
    }
}

impl std::ops::Deref for TypeName {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T> From<T> for TypeName
where
    T: Into<String>,
{
    fn from(v: T) -> Self {
        Self(v.into())
    }
}

// impl From<TypeName> for String {
//     fn from(v: TypeName) -> Self {
//         v.0
//     }
// }

//TODO 创建 (css name, GenericSize) 类型
#[derive(Display, Clone, Debug, From, PartialEq, PartialOrd, Eq)]
#[display(fmt = "{}")]
pub enum GenericSize {
    #[display(fmt = "auto")]
    #[from(ignore)]
    Auto,
    // #[display(fmt = "{}", _0)]
    Length(LogicLength),
    // Percentage(Percent),
    #[display(fmt = "initial")]
    #[from(ignore)]
    Initial,
    #[display(fmt = "inherit")]
    #[from(ignore)]
    Inherit,
    // #[display(fmt = "{}", _0)]
    StringValue(String),
    // #[display(fmt = "{}", _0)]
    Calculation(Box<CalcOp<GenericSize>>),
    Parent(TypeName),
}

pub fn parent_ty<T>() -> GenericSize
where
    T: TypeCheck,
{
    GenericSize::Parent(T::static_type_name())
}
pub fn parent_str(type_name: &str) -> GenericSize {
    GenericSize::Parent(TypeName::from(type_name))
}

impl Default for GenericSize {
    fn default() -> Self {
        Self::Length(px(0))
    }
}

impl From<f64> for GenericSize {
    fn from(v: f64) -> Self {
        Self::Length(
            ExactLengthSimplex {
                unit: Unit::Empty,
                value: NotNan::new(v).unwrap(),
            }
            .into(),
        )
    }
}

impl ::core::ops::Mul<f64> for GenericSize {
    type Output = GenericSize;
    fn mul(self, rhs: f64) -> GenericSize {
        Self::Calculation(Box::new(CalcOp::mul(self, rhs)))
    }
}
// impl ::core::ops::Add for GenericSize {
//     type Output = GenericSize;
//     fn add(self, rhs: GenericSize) -> GenericSize {
//         Self::Calculation(Box::new(CalcOp::add(self, rhs)))
//     }
// }
impl<T> ::core::ops::Add<T> for GenericSize
where
    T: Into<Self>,
{
    type Output = GenericSize;
    fn add(self, rhs: T) -> GenericSize {
        Self::Calculation(Box::new(CalcOp::add(self, rhs.into())))
    }
}

impl GenericSize {
    #[must_use]
    pub fn get_length_value(&self) -> f64 {
        self.try_get_length_value()
            .expect("directly get length value failed, expected Length Px or None struct")
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
    pub const fn as_length(&self) -> Option<&LogicLength> {
        if let Self::Length(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use im_rc::Vector;

    use crate::into_vector;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
        let f: Vector<i32> = into_vector![1, 2, 3];
    }
}
