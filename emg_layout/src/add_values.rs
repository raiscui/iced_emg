/*
 * @Author: Rais
 * @Date: 2021-04-25 19:56:42
 * @LastEditTime: 2023-03-16 23:24:37
 * @LastEditors: Rais
 * @Description:
 */
#![allow(clippy::use_self)]
use emg_common::impl_to_generic_size_g_a_l_i_i_s;
use seed_style_macros::AddStyleMacro;
use seed_styles::{CssHeight, CssWidth};

use crate::{styles::LogicLength, GenericSize};
use derive_more::Display;
use derive_more::From;

// ─────────────────────────────────────────────────────────────────────────────

#[derive(Display, Clone, Debug, From, AddStyleMacro)]
#[display(fmt = "{}")]
#[generic_size]
#[short_prop = "ox"]
pub enum OriginX {
    #[display(fmt = "auto")]
    Auto,
    Length(LogicLength),
    // Percentage(Percent),
    #[display(fmt = "initial")]
    Initial,
    #[display(fmt = "inherit")]
    Inherit,
    StringValue(String),
    Gs(GenericSize),
}

#[derive(Display, Clone, Debug, From, AddStyleMacro)]
#[display(fmt = "{}")]
pub enum OriginY {
    #[display(fmt = "auto")]
    Auto,
    Length(LogicLength),
    #[display(fmt = "initial")]
    Initial,
    #[display(fmt = "inherit")]
    Inherit,
    StringValue(String),
    Gs(GenericSize),
}

#[derive(Display, Clone, Debug, From, AddStyleMacro, PartialEq, Eq)]
#[display(fmt = "{}")]
pub enum AlignX {
    #[display(fmt = "auto")]
    Auto,
    Length(LogicLength),
    #[display(fmt = "initial")]
    Initial,
    #[display(fmt = "inherit")]
    Inherit,
    StringValue(String),
    Gs(GenericSize),
}

#[derive(Display, Clone, Debug, From, AddStyleMacro)]
#[display(fmt = "{}")]
pub enum AlignY {
    #[display(fmt = "auto")]
    Auto,
    Length(LogicLength),
    #[display(fmt = "initial")]
    Initial,
    #[display(fmt = "inherit")]
    Inherit,
    StringValue(String),
    Gs(GenericSize),
}

// ─────────────────────────────────────────────────────────────────────────────

impl_to_generic_size_g_a_l_i_i_s!(AlignX);
impl_to_generic_size_g_a_l_i_i_s!(AlignY);
impl_to_generic_size_g_a_l_i_i_s!(OriginX);
impl_to_generic_size_g_a_l_i_i_s!(OriginY);

#[cfg(test)]
mod add_test_mod {
    use emg_common::{pc, px, GenericSize};

    use super::*;

    #[test]
    fn align_x_add() {
        let a = AlignX::Gs(GenericSize::Length(px(10)));
        let b = align_x(pc(100));
        let c = a + &b;
        println!("{c:?}");
        let d = b + align_x(px(100));
        println!("{d:?}");
        let d = d + align_x(px(100));
        println!("{d:?}");
        let d = d + align_x(px(100));
        println!("{d:?}");
        let d = d + align_x(px(100));
        println!("{d:?}");
        let res = align_x(pc(100)) + align_x(px(400));
        assert_eq!(d, res);
    }
    #[test]
    fn align_x_add2() {
        let a = align_x(pc(100));
        let b = align_x(px(100));
        // ─────────────────────────────────────────────────────────────

        let c = a.clone() + &b;
        let c = c + &b;
        let c = c + &b;
        let c = c + &b;
        println!("{c:?}");

        let c = &b + &a;
        let c = c + &a;
        let c = c + &a;
        let c = c + &a;
        println!("{c:?}");

        let c = &a + &b;
        let c = &b + &c;
        let c = &b + &c;
        let c = &b + &c;
        println!("{c:?}");
        let c = &a + &b;
        let c = &c + &b;
        let c = &c + &b;
        let c = &c + &b;
        println!("{c:?}");
    }
}
