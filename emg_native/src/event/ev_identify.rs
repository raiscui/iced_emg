/*
 * @Author: Rais
 * @Date: 2023-03-13 14:41:13
 * @LastEditTime: 2023-03-14 00:01:11
 * @LastEditors: Rais
 * @Description:
 */
use integer_hasher::BuildIntHasher;

use crate::mouse;
use crate::touch;

use super::{EventFlag, MOUSE, TOUCH};

#[derive(Clone, Default, PartialEq, Eq)]
pub struct MultiLevelIdentify {
    union: u32,
    map: integer_hasher::IntMap<u32, u32>,
}

impl std::fmt::Debug for MultiLevelIdentify {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let union = format!("{:b}", self.union);
        let mut map = String::new();
        for (k, v) in self.map.iter() {
            map.push_str(&format!(", {:b}:{:b})", k, v));
        }

        f.debug_struct("MultiLevelIdentify")
            .field("union", &union)
            .field("map", &map)
            .finish()
    }
}

impl core::ops::BitOr<EventIdentify> for MultiLevelIdentify {
    type Output = Self;

    fn bitor(mut self, rhs: EventIdentify) -> Self::Output {
        self.insert(rhs);
        self
    }
}

impl MultiLevelIdentify {
    pub fn new(ei: EventIdentify) -> Self {
        let union = ei.0;
        let mut map =
            integer_hasher::IntMap::with_capacity_and_hasher(2, BuildIntHasher::default());
        map.insert(ei.0, ei.1);

        Self { union, map }
    }
    pub fn insert(&mut self, ei: EventIdentify) {
        if self.union & ei.0 == ei.0 {
            //包含
            let v = self.map.get_mut(&ei.0).unwrap();
            *v |= ei.1;
        } else {
            self.union |= ei.0;
            self.map.insert(ei.0, ei.1);
        }
    }
    ///self 宽泛 , ei 具体 ,check self 是否涉及到 ei的flag 且完全在 ev 的 flag 之内 ?
    // #[tracing::instrument]
    pub fn involve(&self, ei: &EventIdentify) -> bool {
        if self.union & ei.0 == ei.0 {
            //包含
            let v = *self.map.get(&ei.0).unwrap();
            ei.1 & v == v
        } else {
            false
        }
    }

    ///self 宽泛 , ei 具体 ,check self 是否涉及到 ei的flag 其中之一(交集)?
    pub fn intersects(&self, ei: &EventIdentify) -> bool {
        if self.union & ei.0 == ei.0 {
            //包含
            let v = *self.map.get(&ei.0).unwrap();
            ei.1 & v != 0
        } else {
            false
        }
    }

    // pub fn contains(&self, ei: &EventIdentify) -> bool {
    //     if self.union & ei.0 == ei.0 {
    //         //包含
    //         let v = self.map.get(&ei.0).unwrap();
    //         *v & ei.1 == ei.1
    //     } else {
    //         false
    //     }
    // }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct EventIdentify(u32, u32);

impl core::ops::BitOr for EventIdentify {
    type Output = MultiLevelIdentify;

    fn bitor(self, rhs: Self) -> Self::Output {
        let mut mli = MultiLevelIdentify::new(self);

        mli.insert(rhs);
        mli
    }
}

impl std::fmt::Debug for EventIdentify {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("EventIdentify")
            .field(&format!("{:08b}", &self.0))
            .field(&format!("{:08b}", &self.1))
            .finish()
    }
}

impl EventIdentify {
    #[inline]
    pub const fn contains(&self, other: &Self) -> bool {
        self.0 == other.0 && (self.1 & other.1) == other.1
    }
}

impl From<mouse::EventFlag> for EventIdentify {
    fn from(x: mouse::EventFlag) -> Self {
        Self(MOUSE.bits(), x.bits())
    }
}
impl From<touch::EventFlag> for EventIdentify {
    fn from(x: touch::EventFlag) -> Self {
        Self(TOUCH.bits(), x.bits())
    }
}

impl From<(EventFlag, u32)> for EventIdentify {
    fn from(x: (EventFlag, u32)) -> Self {
        Self(x.0.bits(), x.1)
    }
}

#[cfg(test)]
mod event_test {
    use crate::{
        event::{EventFlag, EventIdentify},
        mouse, touch, EVENT_HOVER_CHECK,
    };

    #[test]
    fn intersects_test() {
        let mouse_click_left: EventIdentify = mouse::LEFT_RELEASED.into();
        let finger_lost: EventIdentify = (EventFlag::TOUCH, touch::FINGER_LOST.bits()).into();

        assert!(EVENT_HOVER_CHECK.intersects(&mouse_click_left));
        assert!(EVENT_HOVER_CHECK.intersects(&finger_lost));

        let left: EventIdentify = mouse::LEFT.into();
        assert!(!EVENT_HOVER_CHECK.intersects(&left));
    }

    #[test]
    fn bitor() {
        let mouse_click_left: EventIdentify = mouse::LEFT_RELEASED.into();
        let released: EventIdentify = mouse::RELEASED.into();
        let finger_lifted: EventIdentify = (EventFlag::TOUCH, touch::FINGER_LIFTED.bits()).into();
        let finger_lost: EventIdentify = (EventFlag::TOUCH, touch::FINGER_LOST.bits()).into();
        println!("{mouse_click_left:?} {finger_lifted:?}");
        let comb = mouse_click_left | finger_lifted;
        assert!(comb.involve(&mouse_click_left));
        assert!(comb.involve(&finger_lifted));
        // ─────────────────────────────────────────────────────────────

        assert!(!comb.involve(&released));
        assert!(!comb.involve(&finger_lost));
    }
    #[test]
    fn bitor2_sm() {
        let all: EventIdentify = (mouse::EventFlag::all() - mouse::LEFT).into();
        let all: EventIdentify = (mouse::GENERAL_CLICK | mouse::LEFT).into();
        println!("all {all:?}");
        let click: EventIdentify = mouse::CLICK.into();

        // ─────────────────────────────────────────────────────────────────────────────

        let general_click: EventIdentify = mouse::GENERAL_CLICK.into();
        let general_click2: EventIdentify = mouse::GENERAL_CLICK.into();
        let comb = general_click | general_click2;
        println!("{comb:?}");
        assert!(comb.involve(&click));
    }

    #[test]
    fn contains() {
        let mouse_click_left: EventIdentify = mouse::LEFT_RELEASED.into();
        let click: EventIdentify = mouse::GENERAL_CLICK.into();
        let released: EventIdentify = mouse::RELEASED.into();
        assert!(mouse_click_left.contains(&click));
        assert!(mouse_click_left.contains(&released));
    }
    #[test]
    fn group() {
        //一些需要 zoon check 的事件
        let need_zoon_flag = mouse::GENERAL_CLICK | mouse::CURSOR_ENTERED;
        let mouse_click_left = mouse::LEFT_RELEASED;
        let click = mouse::GENERAL_CLICK;
        let released = mouse::RELEASED;
        let left = mouse::LEFT;

        // click 是不是 含有 需要 zoon check的flag呢?
        let intersection = click & need_zoon_flag;
        println!("{intersection:?} , {need_zoon_flag:?}");
        let has = need_zoon_flag.contains(intersection);
        assert!(has);

        let has = click.intersects(need_zoon_flag);
        assert!(has);
        let has2 = need_zoon_flag.intersects(click);
        assert!(has2);

        assert!(mouse_click_left.intersects(need_zoon_flag));
        assert!(released.intersects(need_zoon_flag));
        assert!(!left.intersects(need_zoon_flag));
    }
}
