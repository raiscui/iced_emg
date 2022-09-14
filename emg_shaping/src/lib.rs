#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![warn(clippy::nursery)]
// ────────────────────────────────────────────────────────────────────────────────
#![feature(specialization)]
// #![feature(min_specialization)]
#![feature(auto_traits)]
#![feature(negative_impls)]
// #![feature(trivial_bounds)]
#[cfg(test)]
mod test;
// ────────────────────────────────────────────────────────────────────────────────

mod impl_refresh;
mod refreshers;
mod shape_of_use;
mod shaping_use;
pub use impl_refresh::ShapingUseNoWarper;
pub use impl_refresh::ShapingWhoNoWarper;
pub use refreshers::EqShaping;
pub use refreshers::EqShapingWithDebug;
pub use refreshers::Shaper;
pub use refreshers::ShaperFor;
pub use refreshers::Shaping;
pub use refreshers::ShapingWithDebug;
pub use shape_of_use::ShapeOfUse;
pub use shaping_use::ShapingUse;
// pub use refreshers::TryRefreshFor;
pub use refreshers::TryShapingUse;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let x = 2 + 2;
        assert_eq!(x, 4);
    }
}
