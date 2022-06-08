#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![warn(clippy::nursery)]
// ────────────────────────────────────────────────────────────────────────────────
// #![feature(specialization)]
#![feature(min_specialization)]
#![feature(auto_traits)]
#![feature(negative_impls)]
// #![feature(trivial_bounds)]
#[cfg(test)]
mod test;
// ────────────────────────────────────────────────────────────────────────────────

mod impl_refresh;
mod refresh_use;
mod refresh_use_for;
mod refreshers;
pub use impl_refresh::RefreshUseNoWarper;
pub use impl_refresh::RefreshWhoNoWarper;
pub use refresh_use::RefreshUse;
pub use refresh_use_for::RefreshForUse;
pub use refreshers::EqRefreshFor;
pub use refreshers::RefreshFor;
pub use refreshers::Refresher;
pub use refreshers::RefresherFor;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let x = 2 + 2;
        assert_eq!(x, 4);
    }
}
