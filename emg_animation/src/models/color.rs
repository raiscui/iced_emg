use std::rc::Rc;

use im::vector;

/*
 * @Author: Rais
 * @Date: 2021-05-10 15:31:40
 * @LastEditTime: 2021-05-18 22:41:41
 * @LastEditors: Rais
 * @Description:
 */
use crate::{init_motion, models::Property};

pub struct Color {
    red: u8,
    green: u8,
    blue: u8,
    alpha: f64,
}

impl Color {
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
    name: String,
    Color {
        red,
        green,
        blue,
        alpha,
    }: Color,
) -> Property {
    Property::Color(
        Rc::new(name),
        vector![
            init_motion(f64::from(red), "".to_string()),
            init_motion(f64::from(green), "".to_string()),
            init_motion(f64::from(blue), "".to_string()),
            init_motion(alpha, "".to_string()),
        ],
    )
}
#[must_use]
pub fn fill(color: Color) -> Property {
    custom_color("fill".to_string(), color)
}
