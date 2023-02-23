use crate::im::Vector;

/*
 * @Author: Rais
 * @Date: 2022-07-19 22:11:46
 * @LastEditTime: 2023-02-22 18:09:23
 * @LastEditors: Rais
 * @Description:
 */
pub struct VectorDisp<T>(pub Vector<T>);
impl<T> std::fmt::Display for VectorDisp<T>
where
    T: std::fmt::Display + Clone,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "[")?;
        for one in self.0.iter() {
            writeln!(f, "{},", one)?;
        }
        writeln!(f, "]")
    }
}
pub struct VecDisp<T>(pub Vec<T>);
impl<T> std::fmt::Display for VecDisp<T>
where
    T: std::fmt::Display + Clone,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "[")?;
        for one in self.0.iter() {
            writeln!(f, "{},", one)?;
        }
        writeln!(f, "]")
    }
}

pub enum ResultWithRef<'a, W, T, E> {
    Ok(&'a W, T),
    Err(&'a W, E),
}

pub trait ResultWithSomething<'a, W, T, E> {
    fn with(self, with: &'a W) -> ResultWithRef<'a, W, T, E>;
}
impl<'a, W, T, E> ResultWithSomething<'a, W, T, E> for Result<T, E> {
    #[inline]
    fn with(self, with: &'a W) -> ResultWithRef<'a, W, T, E> {
        ResultWithRef::<'a, W, T, E>::with(with, self)
    }
}

impl<'a, W: 'a, T, E> ResultWithRef<'a, W, T, E> {
    // #[inline]
    // pub fn get_with(self) -> W {
    //     match self {
    //         ResultWith::Ok(w, _) | ResultWith::Err(w, _) => w,
    //     }
    // }

    #[inline]
    pub fn into_result(self) -> Result<T, E> {
        match self {
            ResultWithRef::Ok(_, v) => Ok(v),
            ResultWithRef::Err(_, e) => Err(e),
        }
    }

    #[inline]
    fn with<T2, E2>(with: &'a W, res: Result<T2, E2>) -> ResultWithRef<'a, W, T2, E2> {
        match res {
            Ok(t) => ResultWithRef::Ok(with, t),
            Err(e) => ResultWithRef::Err(with, e),
        }
    }
    #[inline]
    pub fn or_else<F, O: FnOnce(&'a W, E) -> Result<T, F>>(
        self,
        op: O,
    ) -> ResultWithRef<'a, W, T, F> {
        match self {
            Self::Ok(w, t) => ResultWithRef::<'a, W, T, F>::Ok(w, t),
            Self::Err(w, e) => op(w, e).with(w),
        }
    }

    #[inline]
    #[track_caller]
    pub fn unwrap(self) -> T
    where
        E: std::fmt::Debug,
    {
        match self {
            Self::Ok(_, t) => t,
            Self::Err(_, ref e) => {
                panic!("called `ResultWithRef::unwrap()` on an `Err` value: {e:?}")
            }
        }
    }
}
