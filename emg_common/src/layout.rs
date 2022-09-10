/*
 * @Author: Rais
 * @Date: 2022-09-09 16:53:34
 * @LastEditTime: 2022-09-10 11:53:57
 * @LastEditors: Rais
 * @Description:
 */

use im_rc::{vector, Vector};
use nalgebra::{Point2, Translation2, Vector2};

use crate::Pos;

#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub struct RectLTRB {
    /// The minimum x coordinate (left edge).
    pub x0: f64,
    /// The minimum y coordinate (top edge in y-down spaces).
    pub y0: f64,
    /// The maximum x coordinate (right edge).
    pub x1: f64,
    /// The maximum y coordinate (bottom edge in y-down spaces).
    pub y1: f64,
}

impl RectLTRB {
    #[inline]
    pub fn from_origin_size(origin: Pos<f64>, w: f64, h: f64) -> Self {
        Self {
            x0: origin.x,
            y0: origin.y,
            x1: origin.x + w,
            y1: origin.y + h,
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
        point.x >= self.x0 && point.x < self.x1 && point.y >= self.y0 && point.y < self.y1
    }

    #[inline]
    pub fn is_completely_wrapped(&self, check: &RectLTRB) -> bool {
        self.x0 <= check.x0 && self.y0 <= check.y0 && self.x1 >= check.x1 && self.y1 >= check.y1
    }
}

#[derive(Clone, PartialEq, Debug, Default)]
pub struct LayoutOverride {
    rect_list: crate::Vector<RectLTRB>,
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
        rhs.rect_list
            .into_iter()
            .fold(self, |old, rect| old.underlay(rect))
    }
}
impl std::ops::Add<&Self> for LayoutOverride {
    type Output = Self;

    fn add(self, rhs: &Self) -> Self::Output {
        rhs.rect_list
            .clone()
            .into_iter()
            .fold(self, |old, rect| old.underlay(rect))
    }
}

impl LayoutOverride {
    pub fn new(rect: RectLTRB) -> Self {
        Self {
            rect_list: vector![rect],
            bbox: rect,
        }
    }
    pub fn contains(&self, point: &Pos<f64>) -> bool {
        self.bbox.contains(point) && self.rect_list.iter().any(|rect| rect.contains(point))
    }
    pub fn underlay(mut self, rect: RectLTRB) -> Self {
        if rect.is_completely_wrapped(&self.bbox) {
            //NOTE rect 完全包裹  bb外框

            Self {
                rect_list: vector![rect],
                bbox: rect,
            }
        } else {
            //NOTE rect 完全包裹 交集

            if !self
                .rect_list
                .iter()
                .any(|any_rect| any_rect.is_completely_wrapped(&rect))
            {
                self.rect_list.retain(|sr| !rect.is_completely_wrapped(sr));
                self.rect_list.push_back(rect);
            }

            let bbox = self.bbox.union(rect);

            Self {
                rect_list: self.rect_list,
                bbox,
            }
        }
    }
}
