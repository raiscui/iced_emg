/*
 * @Author: Rais
 * @Date: 2023-03-16 11:18:11
 * @LastEditTime: 2023-03-16 11:26:03
 * @LastEditors: Rais
 * @Description:
 */

use emg_common::GenericSize;
use seed_styles::{CssHeight, CssWidth};

use crate::add_values::{AlignX, AlignY, OriginX, OriginY};
macro_rules! impl_css_add {
    ($x:ty) => {
        impl core::ops::Add for &$x {
            type Output = $x;

            fn add(self, rhs: Self) -> Self::Output {
                <$x>::Gs(GenericSize::from(self.clone()) + GenericSize::from(rhs.clone()))
            }
        }
        impl core::ops::Add for $x {
            type Output = $x;

            fn add(self, rhs: Self) -> Self::Output {
                <$x>::Gs(GenericSize::from(self) + GenericSize::from(rhs))
            }
        }
        impl core::ops::Add<&Self> for $x {
            type Output = $x;

            fn add(self, rhs: &Self) -> Self::Output {
                <$x>::Gs(GenericSize::from(self) + GenericSize::from(rhs.clone()))
            }
        }
    };
}

impl_css_add!(AlignX);
impl_css_add!(AlignY);
impl_css_add!(OriginX);
impl_css_add!(OriginY);
