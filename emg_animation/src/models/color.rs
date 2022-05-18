use emg_core::vector;
use seed_styles::Unit;

/*
 * @Author: Rais
 * @Date: 2021-05-10 15:31:40
 * @LastEditTime: 2022-01-25 17:58:53
 * @LastEditors: Rais
 * @Description:
 */
use crate::init_motion;

use crate::models::PropertyOG;

use super::Property;

pub struct Color {
    red: u8,
    green: u8,
    blue: u8,
    alpha: f64,
}

impl Color {
    #[must_use]
    pub const fn new(red: u8, green: u8, blue: u8, alpha: f64) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }
}

fn custom_color_og(
    name: &str,
    Color {
        red,
        green,
        blue,
        alpha,
    }: Color,
) -> PropertyOG {
    PropertyOG::Color(
        name.into(),
        vector![
            init_motion(f64::from(red), Unit::Empty),
            init_motion(f64::from(green), Unit::Empty),
            init_motion(f64::from(blue), Unit::Empty),
            init_motion(alpha, Unit::Empty),
        ],
    )
}
fn custom_color(
    name: &str,
    Color {
        red,
        green,
        blue,
        alpha,
    }: Color,
) -> Property {
    Property::Color(
        name.into(),
        Box::new([
            init_motion(f64::from(red), Unit::Empty),
            init_motion(f64::from(green), Unit::Empty),
            init_motion(f64::from(blue), Unit::Empty),
            init_motion(alpha, Unit::Empty),
        ]),
    )
}
#[must_use]
pub fn fill_sm(color: Color) -> Property {
    custom_color("fill", color)
}

#[must_use]
pub fn fill(color: Color) -> PropertyOG {
    custom_color_og("fill", color)
}
