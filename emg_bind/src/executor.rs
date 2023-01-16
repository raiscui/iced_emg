/*
 * @Author: Rais
 * @Date: 2022-08-10 15:00:01
 * @LastEditTime: 2022-08-19 16:44:30
 * @LastEditors: Rais
 * @Description:
 */
//! Choose your preferred executor to power your application.
pub use crate::runtime::Executor;

/// A default cross-platform executor.
///
/// - On native platforms, it will use:
///   - `iced_futures::backend::native::tokio` when the `tokio` feature is enabled.
///   - `iced_futures::backend::native::async-std` when the `async-std` feature is
///     enabled.
///   - `iced_futures::backend::native::smol` when the `smol` feature is enabled.
///   - `iced_futures::backend::native::thread_pool` otherwise.
///
/// - On Wasm, it will use `iced_futures::backend::wasm::wasm_bindgen`.
pub type Default = emg_futures::backend::default::Executor;
