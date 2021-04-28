/*
 * @Author: Rais
 * @Date: 2021-04-25 19:56:42
 * @LastEditTime: 2021-04-26 23:17:48
 * @LastEditors: Rais
 * @Description:
 */

use seed_style_macros::AddStyleMacro;

use crate::{
    styles::{ExactLength, Percent},
    GenericSize,
};
use derive_more::Display;
use derive_more::From;

#[derive(Display, Clone, Debug, From, AddStyleMacro)]
#[display(fmt = "{}")]
pub enum OriginX {
    #[display(fmt = "auto")]
    Auto,
    Length(ExactLength),
    Percentage(Percent),
    #[display(fmt = "initial")]
    Initial,
    #[display(fmt = "inherit")]
    Inherit,
    StringValue(String),
}
impl From<OriginX> for GenericSize {
    fn from(v: OriginX) -> Self {
        match v {
            OriginX::Auto => Self::Auto,
            OriginX::Length(x) => x.into(),
            OriginX::Percentage(x) => x.into(),
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
    Length(ExactLength),
    Percentage(Percent),
    #[display(fmt = "initial")]
    Initial,
    #[display(fmt = "inherit")]
    Inherit,
    StringValue(String),
}
impl From<OriginY> for GenericSize {
    fn from(v: OriginY) -> Self {
        match v {
            OriginY::Auto => Self::Auto,
            OriginY::Length(x) => x.into(),
            OriginY::Percentage(x) => x.into(),
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
    Length(ExactLength),
    Percentage(Percent),
    #[display(fmt = "initial")]
    Initial,
    #[display(fmt = "inherit")]
    Inherit,
    StringValue(String),
}
impl From<AlignX> for GenericSize {
    fn from(v: AlignX) -> Self {
        match v {
            AlignX::Auto => Self::Auto,
            AlignX::Length(x) => x.into(),
            AlignX::Percentage(x) => x.into(),
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
    Length(ExactLength),
    Percentage(Percent),
    #[display(fmt = "initial")]
    Initial,
    #[display(fmt = "inherit")]
    Inherit,
    StringValue(String),
}
impl From<AlignY> for GenericSize {
    fn from(v: AlignY) -> Self {
        match v {
            AlignY::Auto => Self::Auto,
            AlignY::Length(x) => x.into(),
            AlignY::Percentage(x) => x.into(),
            AlignY::Initial => Self::Initial,
            AlignY::Inherit => Self::Inherit,
            AlignY::StringValue(x) => x.into(),
        }
    }
}
