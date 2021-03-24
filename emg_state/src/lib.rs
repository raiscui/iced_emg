#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![warn(clippy::nursery)]

pub mod use_state;
pub use use_state::use_state;
pub use use_state::CloneState;
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
