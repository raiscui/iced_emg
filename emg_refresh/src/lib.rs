#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![warn(clippy::nursery)]
// ────────────────────────────────────────────────────────────────────────────────
// #![feature(specialization)]
#![feature(min_specialization)]
#![feature(auto_traits)]
#![feature(negative_impls)]

mod impl_refresh;
mod refresh_use;
mod refreshers;
pub use impl_refresh::RefreshUseNoWarper;
pub use impl_refresh::RefreshWhoNoWarper;
pub use refresh_use::RefreshUseFor;
pub use refreshers::RefreshFor;
pub use refreshers::Refresher;
pub use refreshers::RefresherFor;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
