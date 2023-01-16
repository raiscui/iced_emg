/*
 * @Author: Rais
 * @Date: 2023-01-13 12:31:02
 * @LastEditTime: 2023-01-13 12:44:42
 * @LastEditors: Rais
 * @Description:
 */
use bitflags::bitflags;

use crate::Pos;

use super::Finger;
bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct EventFlag: u32 {

        const FINGER_PRESSED =           1<<0;
        const FINGER_MOVED =            1<<1;
        const FINGER_LIFTED =           1<<2;
        const FINGER_LOST =         1<<3 ;

    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
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
