/*
 * @Author: Rais
 * @Date: 2022-08-11 17:48:00
 * @LastEditTime: 2022-08-13 15:41:09
 * @LastEditors: Rais
 * @Description:
 */
//! Interact with the window of your application.
use crate::command::{self, Command};
use emg_native::window;

pub use window::*;

/// Resizes the window to the given logical dimensions.
pub fn resize<Message>(width: u32, height: u32) -> Command<Message> {
    Command::single(command::Action::Window(window::Action::Resize {
        width,
        height,
    }))
}

/// Moves a window to the given logical coordinates.
pub fn move_to<Message>(x: i32, y: i32) -> Command<Message> {
    Command::single(command::Action::Window(window::Action::Move { x, y }))
}
