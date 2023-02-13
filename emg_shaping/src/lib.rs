#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![warn(clippy::nursery)]
// ────────────────────────────────────────────────────────────────────────────────
#![feature(trait_upcasting)]
#![feature(specialization)]
// #![feature(min_specialization)]
#![feature(auto_traits)]
#![feature(negative_impls)]
// #![feature(trivial_bounds)]
#[cfg(test)]
mod test;
// ────────────────────────────────────────────────────────────────────────────────

mod impl_shaping;
mod shape_of_use;
mod shapers;
mod shaping_use;
pub use impl_shaping::ShapingUseNoWarper;
pub use impl_shaping::ShapingWhoNoWarper;
pub use shape_of_use::ShapingUseDyn;
pub use shapers::EqShaping;
pub use shapers::EqShapingWithDebug;
pub use shapers::Shaper;
pub use shapers::ShaperFor;
pub use shapers::Shaping;
pub use shapers::ShapingAny;
pub use shapers::ShapingDyn;
pub use shapers::ShapingUseAny;
pub use shapers::ShapingWithDebug;
pub use shaping_use::ShapingUse;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let x = 2 + 2;
        assert_eq!(x, 4);
    }
}
