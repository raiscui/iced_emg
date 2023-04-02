/*
 * @Author: Rais
 * @Date: 2023-03-28 16:29:49
 * @LastEditTime: 2023-03-31 23:44:00
 * @LastEditors: Rais
 * @Description:
 */

use anchors::{
    expert::{CastFromValOrAnchor, CastIntoValOrAnchor},
    singlethread::{ValOrAnchor, Var, VarVOA},
};

use crate::{StateAnchor, StateVOA, StateVar};

use super::VTFn;
pub mod sa_impl;
pub mod sv_impl;
pub mod svoa_impl;

impl<T: 'static> VTFn<ValOrAnchor<T>> for VarVOA<T> {
    fn new(val: ValOrAnchor<T>) -> Self {
        Self::new(val)
    }
    fn get(&self) -> std::rc::Rc<ValOrAnchor<T>> {
        self.get()
    }
    fn set(&self, val: ValOrAnchor<T>) {
        self.set(val);
    }
}
impl<T: 'static> VTFn<T> for Var<T> {
    fn new(val: T) -> Self {
        Self::new(val)
    }
    fn get(&self) -> std::rc::Rc<T> {
        self.get()
    }
    fn set(&self, val: T) {
        self.set(val);
    }
}

// ─────────────────────────────────────────────────────────────────────────────

impl<X, T> CastFromValOrAnchor<StateVar<X>> for ValOrAnchor<T>
where
    T: 'static + PartialEq,
    X: Into<T> + Clone + 'static,
{
    fn cast_from(value: StateVar<X>) -> Self {
        Self::Anchor(value.watch().0.map(|x| x.clone().into()))
    }
}
impl<X, T> CastFromValOrAnchor<StateVOA<X>> for ValOrAnchor<T>
where
    T: 'static + PartialEq,
    X: Into<T> + Clone + 'static,
{
    fn cast_from(value: StateVOA<X>) -> Self {
        Self::Anchor(value.watch().0.map(|x| x.clone().into()))
    }
}
impl<X, T> CastFromValOrAnchor<StateAnchor<X>> for ValOrAnchor<T>
where
    T: 'static + PartialEq,
    X: Into<T> + Clone + 'static,
{
    fn cast_from(value: StateAnchor<X>) -> Self {
        Self::Anchor(value.0.map(|x| x.clone().into()))
    }
}
