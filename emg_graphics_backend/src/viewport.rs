/*
 * @Author: Rais
 * @Date: 2022-09-02 20:24:27
 * @LastEditTime: 2022-09-03 14:09:59
 * @LastEditors: Rais
 * @Description:
 */

use emg_common::na;
use emg_state::CloneStateVar;
/// A viewing region for displaying computer graphics.
#[derive(Debug, Clone)]
pub struct Viewport {
    physical_size: na::Vector2<u32>,
    logical_size: na::Vector2<f32>,
    scale_factor: f64,
    // projection: Transformation,
}

impl Viewport {
    /// Creates a new [`Viewport`] with the given physical dimensions and scale
    /// factor.
    pub fn with_physical_size(size: na::Vector2<u32>, scale_factor: f64) -> Viewport {
        let res = Viewport {
            physical_size: size,
            logical_size: na::Vector2::<f32>::new(
                (size.x as f64 / scale_factor) as f32,
                (size.y as f64 / scale_factor) as f32,
            ),
            scale_factor,
            // projection: Transformation::orthographic(size.width, size.height),
        };
        //TODO when multiple window, will have multiple global_size
        res.setup_global_size();
        res
    }

    fn setup_global_size(&self) {
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

    /// Returns the scale factor of the [`Viewport`].
    pub fn scale_factor(&self) -> f64 {
        self.scale_factor
    }

    // /// Returns the projection transformation of the [`Viewport`].
    // pub fn projection(&self) -> Transformation {
    //     self.projection
    // }
}
