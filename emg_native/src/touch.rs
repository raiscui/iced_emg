/*
 * @Author: Rais
 * @Date: 2022-08-12 13:33:52
 * @LastEditTime: 2022-08-12 13:33:56
 * @LastEditors: Rais
 * @Description:
 */
//! Build touch events.
use crate::Pos;

/// A touch interaction.
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(missing_docs)]
pub enum Event {
    /// A touch interaction was started.
    FingerPressed { id: Finger, position: Pos },

    /// An on-going touch interaction was moved.
    FingerMoved { id: Finger, position: Pos },

    /// A touch interaction was ended.
    FingerLifted { id: Finger, position: Pos },

    /// A touch interaction was canceled.
    FingerLost { id: Finger, position: Pos },
}

/// A unique identifier representing a finger on a touch interaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Finger(pub u64);
