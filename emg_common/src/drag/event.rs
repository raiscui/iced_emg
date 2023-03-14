/*
 * @Author: Rais
 * @Date: 2023-03-09 22:57:10
 * @LastEditTime: 2023-03-14 18:50:02
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
pub enum Event {
    DragStart { position: Pos },

    Drag { position: Pos, trans: Affine },

    DragEnd { position: Pos, trans: Affine },
}
