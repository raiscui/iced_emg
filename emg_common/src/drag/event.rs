/*
 * @Author: Rais
 * @Date: 2023-03-09 22:57:10
 * @LastEditTime: 2023-03-16 12:00:36
 * @LastEditors: Rais
 * @Description:
 */
use bitflags::bitflags;

use crate::{Affine, Pos};

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct EventFlag: u32 {

        const DRAG_START =      1<<0;
        const DRAG =            1<<1;
        const DRAG_END =        1<<2;

    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Drag {
    pub position: Pos,
    pub trans: Affine,
    pub offset: Affine,
}

impl Drag {
    pub fn offset(&self) -> &Affine {
        &self.offset
    }

    pub fn trans(&self) -> &Affine {
        &self.trans
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Event {
    DragStart { position: Pos },

    Drag(Drag),

    DragEnd { position: Pos, trans: Affine },
}

impl Event {
    #[must_use]
    pub fn as_drag(&self) -> Option<&Drag> {
        if let Self::Drag(v) = self {
            Some(v)
        } else {
            None
        }
    }
}
