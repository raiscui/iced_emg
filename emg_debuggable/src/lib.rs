use std::fmt;

/// Extends a (possibly unsized) value with a Debug string.
// (This type is unsized when T is unsized)
#[derive(Clone)]
pub struct Debuggable<T: ?Sized> {
    pub text: &'static str,
    pub value: T,
}

impl<T: ?Sized> std::ops::Deref for Debuggable<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

/// Produce a Debuggable<T> from an expression for T
#[macro_export]
macro_rules! dbg4 {
    ($($body:tt)+) => {
        Debuggable {
            text: stringify!($($body)+),
            value: $($body)+,
        }
    };
}

// Note: this type is unsized

impl<T: ?Sized> fmt::Debug for Debuggable<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.text)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
