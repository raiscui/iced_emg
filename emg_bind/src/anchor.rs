/*
 * @Author: Rais
 * @Date: 2021-02-23 19:30:34
 * @LastEditTime: 2021-02-24 12:04:29
 * @LastEditors: Rais
 * @Description:
 */

use anchors::{singlethread::Engine, Anchor, VarSetter};

pub struct AnchorWithUpdater<T>(
    pub  (
        Anchor<T, Engine>,
        VarSetter<T, <Engine as anchors::Engine>::DirtyHandle>,
    ),
)
where
    T: 'static;

impl<T> AnchorWithUpdater<T>
where
    T: std::clone::Clone,
{
    pub fn new(
        v: (
            Anchor<T, Engine>,
            VarSetter<T, <Engine as anchors::Engine>::DirtyHandle>,
        ),
    ) -> Self {
        AnchorWithUpdater(v)
    }
    pub fn get(&self) -> T {
        crate::ENGINE.with(|e| e.borrow_mut().get(self.get_anchor()))
    }

    pub fn set(&self, v: T) {
        self.get_setter().set(v);
    }

    #[inline]
    pub fn get_anchor(&self) -> &Anchor<T, Engine> {
        &self.0 .0
    }
    #[inline]
    pub fn get_setter(&self) -> &VarSetter<T, <Engine as anchors::Engine>::DirtyHandle> {
        &self.0 .1
    }
}
