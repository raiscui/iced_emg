#![feature(unboxed_closures, fn_traits, thread_local)]
#![feature(specialization)]

#[cfg(not(target_arch = "wasm32"))]
pub use iced as runtime;

#[cfg(target_arch = "wasm32")]
pub use iced_web as runtime;
#[cfg(target_arch = "wasm32")]
pub use uuid::Uuid;

mod graph_store;
mod impl_refresh;
mod layer;
mod refresh_use;
mod refreshers;
pub use graph_store::*;
pub use layer::Layer;
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
