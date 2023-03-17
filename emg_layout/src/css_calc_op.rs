/*
 * @Author: Rais
 * @Date: 2023-03-16 11:18:11
 * @LastEditTime: 2023-03-16 23:48:12
 * @LastEditors: Rais
 * @Description:
 */

use emg_common::{px, GenericSize, Pos};

use crate::add_values::{AlignX, AlignY, OriginX, OriginY};
impl core::ops::Add<Pos> for &AlignX {
    type Output = AlignX;

    fn add(self, rhs: Pos) -> Self::Output {
        AlignX::Gs(GenericSize::from(self.clone()) + px(rhs.x))
    }
}
impl core::ops::Add<Pos> for &AlignY {
    type Output = AlignY;

    fn add(self, rhs: Pos) -> Self::Output {
        AlignY::Gs(GenericSize::from(self.clone()) + px(rhs.y))
    }
}
impl core::ops::Add<Pos> for &OriginX {
    type Output = OriginX;

    fn add(self, rhs: Pos) -> Self::Output {
        OriginX::Gs(GenericSize::from(self.clone()) + px(rhs.x))
    }
}
impl core::ops::Add<Pos> for &OriginY {
    type Output = OriginY;

    fn add(self, rhs: Pos) -> Self::Output {
        OriginY::Gs(GenericSize::from(self.clone()) + px(rhs.y))
    }
}
