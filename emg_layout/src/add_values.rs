/*
 * @Author: Rais
 * @Date: 2021-04-25 19:56:42
 * @LastEditTime: 2021-04-25 20:02:20
 * @LastEditors: Rais
 * @Description:
 */

use crate::{
    styles::{ExactLength, Percent},
    GenericSize,
};
use derive_more::Display;
use derive_more::From;

#[derive(Display, Clone, Debug, From)]
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
    fn from(w: OriginX) -> Self {
        match w {
            OriginX::Auto => Self::Auto,
            OriginX::Length(x) => x.into(),
            OriginX::Percentage(x) => x.into(),
            OriginX::Initial => Self::Initial,
            OriginX::Inherit => Self::Inherit,
            OriginX::StringValue(x) => x.into(),
        }
    }
}
#[derive(Display, Clone, Debug, From)]
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
    fn from(w: OriginY) -> Self {
        match w {
            OriginY::Auto => Self::Auto,
            OriginY::Length(x) => x.into(),
            OriginY::Percentage(x) => x.into(),
            OriginY::Initial => Self::Initial,
            OriginY::Inherit => Self::Inherit,
            OriginY::StringValue(x) => x.into(),
        }
    }
}
#[derive(Display, Clone, Debug, From)]
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
    fn from(w: AlignX) -> Self {
        match w {
            AlignX::Auto => Self::Auto,
            AlignX::Length(x) => x.into(),
            AlignX::Percentage(x) => x.into(),
            AlignX::Initial => Self::Initial,
            AlignX::Inherit => Self::Inherit,
            AlignX::StringValue(x) => x.into(),
        }
    }
}
#[derive(Display, Clone, Debug, From)]
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
    fn from(w: AlignY) -> Self {
        match w {
            AlignY::Auto => Self::Auto,
            AlignY::Length(x) => x.into(),
            AlignY::Percentage(x) => x.into(),
            AlignY::Initial => Self::Initial,
            AlignY::Inherit => Self::Inherit,
            AlignY::StringValue(x) => x.into(),
        }
    }
}
