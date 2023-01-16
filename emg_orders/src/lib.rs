#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![warn(clippy::nursery)]
// ────────────────────────────────────────────────────────────────────────────────
// #![feature(generic_associated_types)]
#![feature(unboxed_closures)]
mod orders;
pub use orders::*;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
