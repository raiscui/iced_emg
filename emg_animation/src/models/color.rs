use emg_common::vector;
use seed_styles::Unit;

/*
 * @Author: Rais
 * @Date: 2021-05-10 15:31:40
 * @LastEditTime: 2023-01-13 16:48:11
 * @LastEditors: Rais
 * @Description:
 */
use crate::init_motion;

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
pub fn fill(color: Color) -> Property {
    custom_color("fill", color)
}
