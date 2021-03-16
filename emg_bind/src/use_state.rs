use anchors::singlethread::{Anchor, Var};

use crate::ENGINE;

/*
 * @Author: Rais
 * @Date: 2021-03-15 17:10:47
 * @LastEditTime: 2021-03-15 17:46:53
 * @LastEditors: Rais
 * @Description:
 */

#[derive(Clone)]
pub struct StateAccess<T>(Var<T>);

impl<T: 'static> StateAccess<T> {
    fn new(val: T) -> Self {
        ENGINE.with(|_e| Self(Var::new(val)))
    }

    pub fn set(&self, val: T) {
        self.0.set(val);
    }
    pub fn get_with<F: Fn(&T) -> R, R>(&self, func: F) -> R {
        let t = &*self.0.get();
        func(t)
    }
    #[must_use]
    pub fn watch(&self) -> Anchor<T> {
        self.0.watch()
    }
}
pub trait CloneState<T>
where
    T: Clone + 'static,
{
    fn get(&self) -> T;
}
impl<T> CloneState<T> for StateAccess<T>
where
    T: Clone + 'static,
{
    fn get(&self) -> T {
        (&*self.0.get()).clone()
    }
}
pub fn use_state<T: 'static>(data: T) -> StateAccess<T> {
    StateAccess::new(data)
}
