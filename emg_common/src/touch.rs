/*
 * @Author: Rais
 * @Date: 2023-01-13 12:30:28
 * @LastEditTime: 2023-01-13 12:44:24
 * @LastEditors: Rais
 * @Description:
 */
mod event;
pub use event::*;

/// A unique identifier representing a finger on a touch interaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Finger(pub u64);
