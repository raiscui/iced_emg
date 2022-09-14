mod func;
mod layout;
mod macros;
mod tools;
// ────────────────────────────────────────────────────────────────────────────────
pub mod any;
pub mod keyboard;
pub mod measures;
pub mod mouse;
pub mod time;
// ────────────────────────────────────────────────────────────────────────────────
pub type Pos<T = f32> = na::Point2<T>;
pub use crate::SVec::{smallvec, SmallVec};
pub use ::smallvec as SVec;
pub use better_any;
pub use compact_str as id_str;
pub use compact_str::CompactString as IdStr;
pub use dyn_partial_eq;
pub use im::{vector, Vector};
pub use im_rc as im;
pub use layout::*;
pub use measures::*;
pub use nalgebra as na;
pub use ordered_float::NotNan;
pub use tools::*;
// ────────────────────────────────────────────────────────────────────────────────

use derive_more::{Display, From};
// pub use smol_str::SmolStr as IdStr;
// pub use smartstring::alias::String as SmString;
// use compact_str::CompactString as SmString;

// pub use tinyvec::{tiny_vec, TinyVec};
// ────────────────────────────────────────────────────────────────────────────────
pub trait TypeCheck {
    const TYPE_NAME: TypeName;
    // fn static_type_name() -> TypeName;
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

#[derive(Display, Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct TypeName(IdStr);

impl TypeName {
    pub const fn new(name: IdStr) -> Self
// where
    //     T: AsRef<str>,
    {
        Self(name)
    }
}

impl std::ops::Deref for TypeName {
    type Target = IdStr;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T> From<T> for TypeName
where
    T: Into<IdStr>,
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
#[derive(Display, Default, Clone, Debug, From, PartialEq, PartialOrd, Eq)]
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
    #[default]
    None,
}

impl GenericSize {
    pub fn zero() -> Self {
        Self::Length(px(0))
    }
}

pub fn parent_ty<T>() -> GenericSize
where
    T: TypeCheck,
{
    GenericSize::Parent(T::TYPE_NAME)
}
pub fn parent_str(type_name: &str) -> GenericSize {
    GenericSize::Parent(TypeName::from(type_name))
}

// impl Default for GenericSize {
//     fn default() -> Self {
//         Self::Length(px(0))
//         //TODO init val
//         // Self::Initial
//     }
// }

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

    /// Returns `true` if the generic size is [`None`].
    ///
    /// [`None`]: GenericSize::None
    #[must_use]
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use crate::Vector;

    use crate::into_vector;

    struct XX(Rc<dyn Fn() -> u32>);

    impl std::fmt::Debug for XX {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_tuple("XX").finish()
        }
    }

    impl PartialEq for XX {
        fn eq(&self, other: &Self) -> bool {
            std::ptr::eq(
                (std::ptr::addr_of!(*self.0)).cast::<u8>(),
                (std::ptr::addr_of!(*other.0)).cast::<u8>(),
            )
        }
    }

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
        let _f: Vector<i32> = into_vector![1, 2, 3];

        let a = XX(Rc::new(|| 2u32));

        let b = XX(a.0.clone());
        let c = XX(Rc::new(|| 2u32));

        assert_eq!(a, b);
        assert_ne!(a, c);
        assert!(Rc::ptr_eq(&a.0, &b.0));
        assert!(!Rc::ptr_eq(&a.0, &c.0));
    }
}
