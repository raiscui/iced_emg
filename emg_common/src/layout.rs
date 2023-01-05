/*
 * @Author: Rais
 * @Date: 2022-09-09 16:53:34
 * @LastEditTime: 2023-01-03 16:26:35
 * @LastEditors: Rais
 * @Description:
 */

use std::cmp::Ordering;

use im_rc::{vector, OrdSet, Vector};
use nalgebra::{Point2, Translation2, Vector2};
use ordered_float::NotNan;

use crate::Pos;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct RectLTRB {
    /// The minimum x coordinate (left edge).
    pub x0: NotNan<f64>,
    /// The minimum y coordinate (top edge in y-down spaces).
    pub y0: NotNan<f64>,
    /// The maximum x coordinate (right edge).
    pub x1: NotNan<f64>,
    /// The maximum y coordinate (bottom edge in y-down spaces).
    pub y1: NotNan<f64>,
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
    pub fn from_origin_size(origin: Pos<f64>, w: f64, h: f64) -> Self {
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
        point.x >= *self.x0 && point.x < *self.x1 && point.y >= *self.y0 && point.y < *self.y1
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

    fn add(self, rhs: Self) -> Self::Output {
        // for rect in rhs.rect_list {
        //     // if !self
        //     //     .rect_list
        //     //     .iter()
        //     //     .any(|any_rect| any_rect.is_completely_wrapped(&rect))
        //     // {
        //     //     self.rect_list.retain(|sr| !rect.is_completely_wrapped(sr));
        //     //     self.rect_list.push_back(rect);
        //     //     self.bbox = self.bbox.union(rect);
        //     // }
        // }
        // self
        rhs.rect_tree.into_iter().fold(self, |mut old, rect| {
            old.underlay(rect);
            old
        })
    }
}
impl std::ops::Add<&Self> for LayoutOverride {
    type Output = Self;

    fn add(self, rhs: &Self) -> Self::Output {
        rhs.rect_tree
            .clone()
            .into_iter()
            .fold(self, |mut old, rect| {
                old.underlay(rect);
                old
            })
    }
}

impl LayoutOverride {
    pub fn new(rect: RectLTRB) -> Self {
        Self {
            rect_tree: OrdSet::new(),
            bbox: rect,
        }
    }
    pub fn contains(&self, point: &Pos<f64>) -> bool {
        self.bbox.contains(point) && self.rect_tree.iter().any(|rect| rect.contains(point))
    }

    pub fn underlay(&mut self, rect: RectLTRB) {
        if rect.is_completely_wrapped(&self.bbox) {
            //NOTE rect 完全包裹  bb外框

            self.bbox = rect;
            self.rect_tree.clear();
            self.rect_tree.insert(rect);
        } else if rect.is_completely_disjoint(&self.bbox) {
            //NOTE rect 完全不相交  bb外框
            self.rect_tree.insert(rect);

            self.bbox = self.bbox.union(rect);
        } else {
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
