/*
 * @Author: Rais
 * @Date: 2021-04-25 19:56:42
 * @LastEditTime: 2022-05-16 15:46:27
 * @LastEditors: Rais
 * @Description:
 */
#![allow(clippy::use_self)]
use seed_style_macros::AddStyleMacro;

use crate::{styles::LogicLength, GenericSize};
use derive_more::Display;
use derive_more::From;

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
impl From<OriginX> for GenericSize {
    fn from(v: OriginX) -> Self {
        match v {
            OriginX::Gs(gs) => gs,
            OriginX::Auto => Self::Auto,
            OriginX::Length(x) => x.into(),
            OriginX::Initial => Self::Initial,
            OriginX::Inherit => Self::Inherit,
            OriginX::StringValue(x) => x.into(),
        }
    }
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
impl From<OriginY> for GenericSize {
    fn from(v: OriginY) -> Self {
        match v {
            OriginY::Gs(gs) => gs,
            OriginY::Auto => Self::Auto,
            OriginY::Length(x) => x.into(),
            OriginY::Initial => Self::Initial,
            OriginY::Inherit => Self::Inherit,
            OriginY::StringValue(x) => x.into(),
        }
    }
}
#[derive(Display, Clone, Debug, From, AddStyleMacro)]
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
impl From<AlignX> for GenericSize {
    fn from(v: AlignX) -> Self {
        match v {
            AlignX::Gs(gs) => gs,
            AlignX::Auto => Self::Auto,
            AlignX::Length(x) => x.into(),
            AlignX::Initial => Self::Initial,
            AlignX::Inherit => Self::Inherit,
            AlignX::StringValue(x) => x.into(),
        }
    }
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
impl From<AlignY> for GenericSize {
    fn from(v: AlignY) -> Self {
        match v {
            AlignY::Gs(gs) => gs,
            AlignY::Auto => Self::Auto,
            AlignY::Length(x) => x.into(),
            AlignY::Initial => Self::Initial,
            AlignY::Inherit => Self::Inherit,
            AlignY::StringValue(x) => x.into(),
        }
    }
}
