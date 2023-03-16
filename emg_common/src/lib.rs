#![feature(specialization)]
#![feature(box_patterns)]
#![feature(let_chains)]
// ─────────────────────────────────────────────────────────────────────────────

mod func;
mod layout;
mod macros;
mod tools;
// ────────────────────────────────────────────────────────────────────────────────
pub mod animation;
pub mod any;
pub mod display;
pub mod drag;
pub mod keyboard;
pub mod measures;
pub mod mouse;
pub mod time;
pub mod touch;
pub mod window;
pub use nalgebra as na;

// ─────────────────────────────────────────────────────────────────────────────

pub use crate::SVec::{smallvec, SmallVec};
pub use ::smallvec as SVec;
pub use better_any;
use better_any::Tid;
pub use compact_str as id_str;
pub use compact_str::CompactString as IdStr;
pub use dyn_partial_eq;

pub extern crate im_rc as im;
// pub extern crate imbl as im;
pub use im::Vector;
pub use layout::*;
pub use measures::*;
use num_traits::AsPrimitive;

pub use num_traits;
pub use ordered_float::NotNan;
pub use tools::*;
// ────────────────────────────────────────────────────────────────────────────────

use derive_more::{Display, From};
// pub use smol_str::SmolStr as IdStr;
// pub use smartstring::alias::String as SmString;
// use compact_str::CompactString as SmString;

// pub use tinyvec::{tiny_vec, TinyVec};
// ────────────────────────────────────────────────────────────────────────────────
// ────────────────────────────────────────────────────────────────────────────────
pub type Precision = f32;
pub type Pos<T = Precision> = na::Point2<T>;
pub type Affine<T = Precision> = na::Affine2<T>;

pub trait TypeCheck {
    const TYPE_NAME: TypeName;
    // fn static_type_name() -> TypeName;
}
pub trait TypeCheckObjectSafe {
    fn type_name(&self) -> TypeName;
}
pub trait TypeCheckObjectSafeTid: for<'a> Tid<'a> + TypeCheckObjectSafe {}
impl<T> TypeCheckObjectSafeTid for T where T: for<'a> Tid<'a> + TypeCheckObjectSafe {}

// // use derive_more::Into;
//TODO full this  "-,/" op
#[derive(Display, Copy, Clone, Debug, PartialEq, PartialOrd, Eq)]
pub enum CalcOp<T>
where
    T: Clone + std::fmt::Debug + PartialEq + PartialOrd + Eq,
{
    #[display(fmt = "{} + {}", a, b)]
    Add { a: T, b: T },
    #[display(fmt = "{} * {}", a, b)]
    Mul { a: T, b: NotNan<Precision> },
}

impl<T> CalcOp<T>
where
    T: Clone + std::fmt::Debug + PartialEq + PartialOrd + Eq,
{
    pub fn add(a: T, b: T) -> Self {
        Self::Add { a, b }
    }

    pub fn mul(a: T, b: Precision) -> Self {
        Self::Mul {
            a,
            b: NotNan::new(b.as_()).unwrap(),
        }
    }
}
impl CalcOp<LogicLength> {
    pub fn has_add_unit(&self, unit: crate::measures::Unit) -> bool {
        match self {
            Self::Add { a, b } => a.has_add_unit(unit) || b.has_add_unit(unit),
            Self::Mul { a, b: _ } => false,
        }
    }
}
impl CalcOp<GenericSize> {
    pub fn has_add_unit(&self, unit: crate::measures::Unit) -> bool {
        match self {
            Self::Add { a, b } => a.has_add_unit(unit) || b.has_add_unit(unit),
            Self::Mul { a, b: _ } => false,
        }
    }
}

#[derive(Display, Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
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
    pub fn add_directly(l: Self, r: Self) -> Self {
        Self::Calculation(Box::new(CalcOp::add(l, r)))
    }
    pub fn zero() -> Self {
        Self::Length(px(0))
    }
    pub fn has_add_unit(&self, unit: crate::measures::Unit) -> bool {
        match self {
            Self::Length(v) => v.has_add_unit(unit),
            Self::Calculation(v) => v.has_add_unit(unit),
            _ => false,
        }
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

//         // Self::Initial
//     }
// }

impl From<f64> for GenericSize {
    fn from(v: f64) -> Self {
        Self::Length(
            ExactLengthSimplex {
                unit: Unit::Empty,
                value: NotNan::new(v.as_()).unwrap(),
            }
            .into(),
        )
    }
}

impl<T> ::core::ops::Mul<T> for GenericSize
where
    T: AsPrimitive<Precision>,
{
    type Output = GenericSize;
    fn mul(self, rhs: T) -> GenericSize {
        Self::Calculation(Box::new(CalcOp::mul(self, rhs.as_())))
    }
}

impl<T> ::core::ops::Add<T> for GenericSize
where
    T: Into<GenericSize>,
{
    type Output = GenericSize;
    default fn add(self, rhs: T) -> GenericSize {
        Self::Calculation(Box::new(CalcOp::add(self, rhs.into())))
    }
}
impl ::core::ops::Add for GenericSize {
    fn add(self, rhs: GenericSize) -> GenericSize {
        // Self::Calculation(Box::new(CalcOp::add(self, rhs)))
        match (self, rhs) {
            (Self::Length(l), Self::Length(r)) => Self::Length(l + r),
            (Self::Length(l), calculation) => calculation + l,
            (calculation, Self::Length(r)) => calculation + r,
            (Self::Calculation(l), Self::Calculation(r)) => {
                Self::add_directly(Self::Calculation(l), Self::Calculation(r))
            }
            (a, b) => Self::add_directly(a, b),
        }
    }
}

impl ::core::ops::Add<LogicLength> for GenericSize {
    fn add(self, rhs: LogicLength) -> GenericSize {
        match self {
            Self::Length(l) => Self::Length(l + rhs),
            Self::Calculation(x) => {
                if let Some(unit) = rhs.get_unit() && x.has_add_unit(unit) {
                    match *x {
                        CalcOp::Add { a, b } => {
                            if a.has_add_unit(unit) {
                                return a + rhs + b
                            }
                            if b.has_add_unit(unit) {
                                return a + (b + rhs);
                            }
                            unreachable!("...");
                        }
                        CalcOp::Mul { .. } => {
                            unreachable!("...");
                        }
                    }
                } else {
                    Self::add_directly(Self::Calculation(x), Self::Length(rhs))
                }
            }
            other => {
                Self::add_directly(other, Self::Length(rhs))
            }
        }
    }
}

#[cfg(test)]
mod generic_size_test {
    use ordered_float::NotNan;

    use crate::{pc, px, vh, Affine, CalcOp, ExactLengthSimplex, GenericSize, LogicLength, Unit};

    #[test]
    fn add_test2() {
        let a: GenericSize = GenericSize::Auto;
        let b: GenericSize = pc(100).into();
        let end = a + b;
        println!("{end:?}");

        let end = end + GenericSize::StringValue("xx".to_string());
        println!("{end:?}");

        let end = end + pc(50);
        let x: GenericSize = pc(100).into();
        println!("{end:?}");

        let end = end + x;

        println!("{end:?}");
        let res = GenericSize::add_directly(
            GenericSize::add_directly(GenericSize::Auto, pc(250).into()),
            GenericSize::StringValue("xx".to_string()),
        );
        assert_eq!(end, res);
    }

    #[test]
    fn add_test() {
        let a: GenericSize = px(11).into();
        let b: GenericSize = pc(100).into();
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
        }))
        .into();
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
        }))
        .into();
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
        }))
        .into();
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
        }))
        .into();
        assert_eq!(end, res);

        let end = end.clone() + end;
        // println!("{end:?}");

        let a: GenericSize = px(11).into();
        let b: LogicLength = pc(100);
        let end = a + b;
        let end = end.clone() + end;
        // println!("{end:?}");
        let a: LogicLength = px(11);
        let end = end + a;
        println!("{end:?}");
    }
}

impl GenericSize {
    #[must_use]
    pub fn get_length_value(&self) -> Precision {
        let msg = format!(
            "directly get length value failed, expected Length Px or None struct, v:{:?}",
            self
        );
        self.try_get_length_value().expect(&msg)
    }

    /// # Errors
    ///
    /// Will return `Err` if `self` does not `Length` and `Length`  unit is not px
    pub fn try_get_length_value(&self) -> Result<Precision, &Self> {
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

    use crate::{Affine, Vector};

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
    #[allow(clippy::vtable_address_comparisons)]

    fn it_works() {
        let f = Affine::<f32>::default();
        println!("{f:?}");
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
