use std::rc::Rc;

/*
 * @Author: Rais
 * @Date: 2022-06-14 11:38:22
 * @LastEditTime: 2023-01-13 15:55:34
 * @LastEditors: Rais
 * @Description:
 */
pub use anchors::singlethread::Anchor;
pub use anchors::singlethread::Engine;
pub use anchors::singlethread::Var;

use crate::StateAnchor;

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(PartialEq, Eq, Clone)]
pub struct StateVarLit<T>(Var<T>);

impl<T: 'static + std::fmt::Display + Clone> std::fmt::Display for StateVarLit<T> {
    default fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let v = self.get();
        write!(f, "\u{2726}-Lit ({})", &v)
    }
}

impl<T: 'static + std::fmt::Debug + Clone> std::fmt::Debug for StateVarLit<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let v = self.get();
        f.debug_tuple("StateVarLit").field(&v).finish()
    }
}

impl<T: 'static> StateVarLit<T> {
    pub fn new(v: T) -> Self {
        Self(Var::new(v))
    }
    #[must_use]
    pub fn get_rc(&self) -> Rc<T> {
        self.0.get()
    }
    pub fn get(&self) -> T
    where
        T: Clone,
    {
        self.get_with(std::clone::Clone::clone)
    }

    pub fn get_with<F: Fn(&T) -> R, R>(&self, func: F) -> R {
        func(&*self.0.get())
    }

    pub fn set(&self, val: T) {
        self.0.set(val);
    }

    pub fn update<F: FnOnce(&mut T)>(&self, func: F)
    where
        T: Clone,
    {
        let mut v = self.0.get().as_ref().clone();
        func(&mut v);
        self.0.set(v);
    }

    #[must_use]
    pub fn watch(&self) -> StateAnchor<T> {
        StateAnchor(self.0.watch())
    }
}

#[cfg(test)]
mod state_var_lit_test {
    use std::rc::Rc;

    use anchors::singlethread::{Engine, Var};

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct FF(String);

    #[test]
    fn comp() {
        let mut _engine = Engine::new();

        let a = Var::new("a".to_string());
        let a2 = Var::new("a".to_string());
        let b = a.clone();
        assert_eq!(a, b);
        assert_ne!(a, a2);
        a.set("b".to_string());
        assert_eq!(a, b);
        assert_ne!(a, a2);
        assert_eq!(a.get(), b.get());
        println!("{} {}", a.get(), b.get());
        let f = Rc::new(FF("a".to_string()));
        let f2 = f.clone();
        assert_eq!(f, f2);

        assert!(Rc::ptr_eq(&f, &f2));
        let f_like = Rc::new(FF("a".to_string()));
        assert_eq!(f, f_like);
        assert!(!Rc::ptr_eq(&f, &f_like));

        let aa = a.watch();
        a.set("a2".to_string());
        let aac = aa.clone();
        a.set("a23".to_string());

        let ba = b.watch();
        a.set("a24".to_string());
        assert_eq!(aa, aac);
        assert_eq!(aa, ba);
    }
}

// impl<T: 'static + std::fmt::Display + Clone> std::fmt::Display for StateVar<T> {
//     default fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         let v = self.get();
//         write!(f, "\u{2726} ({})", &v)
//     }
// }
// impl<T: 'static + std::fmt::Debug + Clone> std::fmt::Debug for StateVar<T> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         let v = self.get();
//         f.debug_tuple("StateVar").field(&v).finish()
//     }
// }
