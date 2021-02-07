#![feature(unboxed_closures, fn_traits)]

#[cfg(not(target_arch = "wasm32"))]
pub use iced as runtime;

#[cfg(target_arch = "wasm32")]
pub use iced_web as runtime;
pub use uuid::Uuid;

mod graph_store;
mod layer;
pub use graph_store::*;
pub use layer::Layer;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
