/*
 * @Author: Rais
 * @Date: 2022-08-13 13:23:36
 * @LastEditTime: 2022-08-13 13:25:13
 * @LastEditors: Rais
 * @Description:
 */
//! Run commands and subscriptions.

/// A native runtime with a generic executor and receiver of results.
///
/// It can be used by shells to easily spawn a [`Command`] or track a
/// [`Subscription`].
///
/// [`Command`]: crate::Command
/// [`Subscription`]: crate::Subscription
pub type FutureRuntime<Executor, Receiver, Message> =
    emg_futures::FutureRuntime<Executor, Receiver, Message>;
