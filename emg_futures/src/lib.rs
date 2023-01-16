// ────────────────────────────────────────────────────────────────────────────────

mod command;
mod future_runtime;
mod maybe_send;

pub mod backend;
pub mod executor;

pub use command::Command;
pub use executor::Executor;
pub use future_runtime::FutureRuntime;
pub use futures;
pub use maybe_send::MaybeSend;
pub use platform::*;

#[cfg(not(target_arch = "wasm32"))]
mod platform {
    /// A boxed static future.
    ///
    /// - On native platforms, it needs a `Send` requirement.
    /// - On the Web platform, it does not need a `Send` requirement.
    pub type BoxFuture<T> = futures::future::BoxFuture<'static, T>;

    /// A boxed static stream.
    ///
    /// - On native platforms, it needs a `Send` requirement.
    /// - On the Web platform, it does not need a `Send` requirement.
    pub type BoxStream<T> = futures::stream::BoxStream<'static, T>;

    /// Boxes a stream.
    ///
    /// - On native platforms, it needs a `Send` requirement.
    /// - On the Web platform, it does not need a `Send` requirement.
    pub fn boxed_stream<T, S>(stream: S) -> BoxStream<T>
    where
        S: futures::Stream<Item = T> + Send + 'static,
    {
        futures::stream::StreamExt::boxed(stream)
    }
}

#[cfg(target_arch = "wasm32")]
mod platform {
    /// A boxed static future.
    ///
    /// - On native platforms, it needs a `Send` requirement.
    /// - On the Web platform, it does not need a `Send` requirement.
    pub type BoxFuture<T> = futures::future::LocalBoxFuture<'static, T>;

    /// A boxed static stream.
    ///
    /// - On native platforms, it needs a `Send` requirement.
    /// - On the Web platform, it does not need a `Send` requirement.
    pub type BoxStream<T> = futures::stream::LocalBoxStream<'static, T>;

    /// Boxes a stream.
    ///
    /// - On native platforms, it needs a `Send` requirement.
    /// - On the Web platform, it does not need a `Send` requirement.
    pub fn boxed_stream<T, S>(stream: S) -> BoxStream<T>
    where
        S: futures::Stream<Item = T> + 'static,
    {
        futures::stream::StreamExt::boxed_local(stream)
    }
}
