/*
 * @Author: Rais
 * @Date: 2022-09-09 16:53:34
 * @LastEditTime: 2023-02-23 15:06:20
 * @LastEditors: Rais
 * @Description:
 */

use std::cmp::Ordering;

use crate::im::OrdSet;
use ordered_float::NotNan;
use tracing::{debug, debug_span};

use crate::{Pos, Precision};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct RectLTRB {
    /// The minimum x coordinate (left edge).
    pub x0: NotNan<Precision>,
    /// The minimum y coordinate (top edge in y-down spaces).
    pub y0: NotNan<Precision>,
    /// The maximum x coordinate (right edge).
    pub x1: NotNan<Precision>,
    /// The maximum y coordinate (bottom edge in y-down spaces).
    pub y1: NotNan<Precision>,
}
// const EPSILON: f64 = 1e-10;

// impl PartialEq for RectLTRB {
//     fn eq(&self, other: &Self) -> bool {
//         (self.x0 - other.x0).abs() < EPSILON
//             && (self.y0 - other.y0).abs() < EPSILON
//             && (self.x1 - other.x1).abs() < EPSILON
//             && (self.y1 - other.y1).abs() < EPSILON
//     }
// }

impl Ord for RectLTRB {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.x0 != other.x0 {
            self.x0.cmp(&other.x0)
        } else if self.y0 != other.y0 {
            self.y0.cmp(&other.y0)
        } else if self.x1 != other.x1 {
            self.x1.cmp(&other.x1)
        } else {
            self.y1.cmp(&other.y1)
        }
    }
}

impl PartialOrd for RectLTRB {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl RectLTRB {
    #[inline]
    pub fn from_origin_size(origin: Pos, w: Precision, h: Precision) -> Self {
        Self {
            x0: NotNan::new(origin.x).unwrap(),
            y0: NotNan::new(origin.y).unwrap(),
            x1: NotNan::new(origin.x + w).unwrap(),
            y1: NotNan::new(origin.y + h).unwrap(),
        }
    }
    #[inline]
    pub fn union(&self, other: RectLTRB) -> Self {
        Self {
            x0: self.x0.min(other.x0),
            y0: self.y0.min(other.y0),
            x1: self.x1.max(other.x1),
            y1: self.y1.max(other.y1),
        }
    }
    #[inline]
    pub fn contains(&self, point: &Pos<f64>) -> bool {
        point.x >= *self.x0 as f64
            && point.x < *self.x1 as f64
            && point.y >= *self.y0 as f64
            && point.y < *self.y1 as f64
    }

    #[inline]
    pub fn is_completely_wrapped(&self, check: &RectLTRB) -> bool {
        self.x0 <= check.x0 && self.y0 <= check.y0 && self.x1 >= check.x1 && self.y1 >= check.y1
    }
    //检测 两个 RectLTRB 完全不相交
    #[inline]
    pub fn is_completely_disjoint(&self, check: &RectLTRB) -> bool {
        self.x0 >= check.x1 || self.x1 <= check.x0 || self.y0 >= check.y1 || self.y1 <= check.y0
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct LayoutOverride {
    rect_tree: OrdSet<RectLTRB>,
    bbox: RectLTRB,
}

impl std::ops::Add for LayoutOverride {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self::Output {
        let _span =
            debug_span!("LayoutOverride", func = "LayoutOverride add", ?self, ?rhs).entered();

        if self.bbox.is_completely_disjoint(&rhs.bbox) {
            self.rect_tree = self.rect_tree.union(rhs.rect_tree);
            self.bbox = self.bbox.union(rhs.bbox);
            self
        } else {
            rhs.rect_tree.into_iter().fold(self, |mut old, rect| {
                #[cfg(feature = "debug")]
                old.underlay(None, rect);
                #[cfg(not(feature = "debug"))]
                old.underlay(rect);

                debug!("{old:?}");
                old
            })
        }
    }
}

// impl std::ops::Add<&Self> for LayoutOverride {
//     type Output = Self;

//     fn add(self, rhs: &Self) -> Self::Output {
//         rhs.rect_tree
//             .clone()
//             .into_iter()
//             .fold(self, |mut old, rect| {
//                 old.underlay(rect);
//                 old
//             })
//     }
// }
#[cfg(feature = "debug")]
use crate::IdStr;
impl LayoutOverride {
    pub fn new(rect: RectLTRB) -> Self {
        Self {
            rect_tree: OrdSet::unit(rect),
            bbox: rect,
        }
    }
    #[cfg(feature = "debug")]
    pub fn contains(&self, point: &Pos<f64>) -> bool {
        let _span = debug_span!("LayoutOverride").entered();

        if !self.bbox.contains(point) {
            debug!("bbox contains: false");

            return false;
        }

        let any = self.rect_tree.iter().any(|rect| {
            debug!("rect_tree rect: {:?}", rect);
            rect.contains(point)
        });
        debug!("any rect contains: {}", any);

        if !any {
            return false;
        }

        true
    }
    #[cfg(not(feature = "debug"))]
    pub fn contains(&self, point: &Pos<f64>) -> bool {
        self.bbox.contains(point) && self.rect_tree.iter().any(|rect| rect.contains(point))
    }

    #[cfg(feature = "debug")]
    pub fn underlay(&mut self, ix: Option<IdStr>, rect: RectLTRB) {
        let _span = debug_span!("LayoutOverride", ?ix, func = "underlay").entered();
        debug!(target = "underlay", "self:{:#?}", self);
        debug!(target = "underlay", "rect:{:#?}", rect);

        if rect.is_completely_wrapped(&self.bbox) {
            //NOTE rect 完全包裹  bb外框

            debug!(target = "underlay", "rect 完全包裹  bb外框");

            self.bbox = rect;
            self.rect_tree.clear();
            self.rect_tree.insert(rect);
        } else if rect.is_completely_disjoint(&self.bbox) {
            //NOTE rect 完全不相交  bb外框

            debug!(target = "underlay", "rect 完全不相交  bb外框");

            self.rect_tree.insert(rect);

            self.bbox = self.bbox.union(rect);
        } else {
            //NOTE rect 与 bb外框 有交集
            debug!(target = "underlay", "rect 与 bb外框 有交集");
            for big in self.rect_tree.range(..=rect) {
                if big.is_completely_wrapped(&rect) {
                    return;
                }
            }

            let mut remove_list = vec![];
            for sm in self.rect_tree.range(rect..) {
                if rect.is_completely_wrapped(sm) {
                    remove_list.push(*sm);
                }
            }

            for rect_to_remove in remove_list {
                self.rect_tree.remove(&rect_to_remove);
            }
            self.rect_tree.insert(rect);
            self.bbox = self.bbox.union(rect);
        }

        debug!(target = "underlay end", ?self);
    }

    #[cfg(not(feature = "debug"))]
    pub fn underlay(&mut self, rect: RectLTRB) {
        let _span = debug_span!("LayoutOverride", func = "underlay").entered();
        debug!(target = "underlay", "self:{:#?}", self);
        debug!(target = "underlay", "rect:{:#?}", rect);

        if rect.is_completely_wrapped(&self.bbox) {
            //NOTE rect 完全包裹  bb外框

            debug!(target = "underlay", "rect 完全包裹  bb外框");

            self.bbox = rect;
            self.rect_tree.clear();
            self.rect_tree.insert(rect);
        } else if rect.is_completely_disjoint(&self.bbox) {
            //NOTE rect 完全不相交  bb外框

            debug!(target = "underlay", "rect 完全不相交  bb外框");

            self.rect_tree.insert(rect);

            self.bbox = self.bbox.union(rect);
        } else {
            //NOTE rect 与 bb外框 有交集
            debug!(target = "underlay", "rect 与 bb外框 有交集");
            for big in self.rect_tree.range(..=rect) {
                if big.is_completely_wrapped(&rect) {
                    return;
                }
            }

            let mut remove_list = vec![];
            for sm in self.rect_tree.range(rect..) {
                if rect.is_completely_wrapped(sm) {
                    remove_list.push(*sm);
                }
            }

            for rect_to_remove in remove_list {
                self.rect_tree.remove(&rect_to_remove);
            }
            self.rect_tree.insert(rect);
            self.bbox = self.bbox.union(rect);
        }

        debug!(target = "underlay end", ?self);
    }

    // pub fn underlay(mut self, rect: RectLTRB) -> Self {
    //     if rect.is_completely_wrapped(&self.bbox) {
    //         //NOTE rect 完全包裹  bb外框

    //         Self {
    //             rect_list: vector![rect],
    //             bbox: rect,
    //         }
    //     } else if rect.is_completely_disjoint(&self.bbox) {
    //         //NOTE rect 完全不相交  bb外框
    //         self.rect_list.push_back(rect);
    //         let bbox = self.bbox.union(rect);
    //         Self {
    //             rect_list: self.rect_list,
    //             bbox,
    //         }
    //     } else {
    //         //NOTE rect 与 bb外框 有交集
    //         if !self
    //             .rect_list
    //             .iter()
    //             .any(|any_rect| any_rect.is_completely_wrapped(&rect))
    //         {
    //             //自身没有任何一个 rect 能完全包裹 rect(自身都没有全完大过rect)
    //             self.rect_list.retain(|sr| !rect.is_completely_wrapped(sr)); // 先移除所有自身完全被包裹的rect (移除比目标小的)
    //             self.rect_list.push_back(rect);
    //         }

    //         let bbox = self.bbox.union(rect);

    //         Self {
    //             rect_list: self.rect_list,
    //             bbox,
    //         }
    //     }
    // }
}
