/*
 * @Author: Rais
 * @Date: 2022-09-02 20:24:27
 * @LastEditTime: 2023-02-01 00:18:11
 * @LastEditors: Rais
 * @Description:
 */

use std::{cell::Cell, rc::Rc};

use emg_common::na;
use emg_state::{state_lit::StateVarLit, CloneState, StateAnchor};
/// A viewing region for displaying computer graphics.
#[derive(Debug, Clone)]
pub struct Viewport {
    physical_size: na::Vector2<u32>,
    logical_size: na::Vector2<f32>,
    vp_scale_factor: StateVarLit<f64>,
    // projection: Transformation,
}

impl Viewport {
    /// Creates a new [`Viewport`] with the given physical dimensions and scale
    /// factor.
    pub fn new(size: na::Vector2<u32>, scale_factor: f64) -> Viewport {
        let res = Viewport {
            physical_size: size,
            logical_size: (size.cast::<f64>() / scale_factor).cast(),
            // logical_size: na::Vector2::<f32>::new(
            //     (size.x as f64 / scale_factor) as f32,
            //     (size.y as f64 / scale_factor) as f32,
            // ),
            vp_scale_factor: StateVarLit::new(scale_factor),
            // projection: Transformation::orthographic(size.width, size.height),
        };
        //TODO when multiple window, will have multiple global_size
        res.setup_global_size();
        res
    }
    pub fn with_physical_size(&self, size: na::Vector2<u32>, vp_scale_factor: f64) -> Self {
        self.vp_scale_factor.set(vp_scale_factor);

        let res = Viewport {
            physical_size: size,
            logical_size: (size.cast::<f64>() / vp_scale_factor).cast(),
            // logical_size: na::Vector2::<f32>::new(
            //     (size.x as f64 / scale_factor) as f32,
            //     (size.y as f64 / scale_factor) as f32,
            // ),
            vp_scale_factor: self.vp_scale_factor.clone(),
            // projection: Transformation::orthographic(size.width, size.height),
        };
        //TODO when multiple window, will have multiple global_size
        res.setup_global_size();
        res
    }

    fn setup_global_size(&self) {
        //TODO 考虑是否可以 clone global_width  global_height sa 到 vp?
        //emg layout work on logical_size
        emg_layout::global_width().set(self.logical_size.x as f64);
        emg_layout::global_height().set(self.logical_size.y as f64);
    }

    /// Returns the physical size of the [`Viewport`].
    pub fn physical_size(&self) -> na::Vector2<u32> {
        self.physical_size
    }

    /// Returns the physical width of the [`Viewport`].
    pub fn physical_width(&self) -> u32 {
        self.physical_size.x
    }

    /// Returns the physical height of the [`Viewport`].
    pub fn physical_height(&self) -> u32 {
        self.physical_size.y
    }

    /// Returns the logical size of the [`Viewport`].
    pub fn logical_size(&self) -> na::Vector2<f32> {
        self.logical_size
    }

    // /// Returns the projection transformation of the [`Viewport`].
    // pub fn projection(&self) -> Transformation {
    //     self.projection
    // }

    pub fn vp_scale_factor_sv(&self) -> StateVarLit<f64> {
        self.vp_scale_factor.clone()
    }
    pub fn vp_scale_factor_sa(&self) -> StateAnchor<f64> {
        self.vp_scale_factor.watch()
    }
    pub fn vp_scale_factor(&self) -> f64 {
        self.vp_scale_factor.get()
    }
}
