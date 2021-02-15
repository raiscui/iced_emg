#![feature(unboxed_closures, fn_traits, thread_local)]
#![feature(specialization)]

#[cfg(not(target_arch = "wasm32"))]
pub use iced as runtime;

#[cfg(target_arch = "wasm32")]
pub use iced_web as runtime;
#[cfg(target_arch = "wasm32")]
pub use uuid::Uuid;

mod graph_store;
mod layer;
mod realtime_update;
mod update_use;
pub use graph_store::*;
pub use layer::Layer;
pub use realtime_update::RTUpdateFor;
pub use realtime_update::RealTimeUpdater;
pub use realtime_update::RealTimeUpdaterFor;
pub use update_use::UpdateUse;
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
