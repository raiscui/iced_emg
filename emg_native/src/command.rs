/*
 * @Author: Rais
 * @Date: 2022-08-11 14:17:45
 * @LastEditTime: 2022-08-11 14:17:45
 * @LastEditors: Rais
 * @Description:
 */
//! Run asynchronous actions.
mod action;

pub use action::Action;

use emg_futures::MaybeSend;

use std::fmt;
use std::future::Future;

/// A set of asynchronous actions to be performed by some runtime.
pub struct Command<T>(emg_futures::Command<Action<T>>);

impl<T> Command<T> {
    /// Creates an empty [`Command`].
    ///
    /// In other words, a [`Command`] that does nothing.
    pub const fn none() -> Self {
        Self(emg_futures::Command::none())
    }

    /// Creates a [`Command`] that performs a single [`Action`].
    pub const fn single(action: Action<T>) -> Self {
        Self(emg_futures::Command::single(action))
    }

    /// Creates a [`Command`] that performs the action of the given future.
    pub fn perform<A>(
        future: impl Future<Output = T> + 'static + MaybeSend,
        f: impl Fn(T) -> A + 'static + MaybeSend,
    ) -> Command<A> {
        use emg_futures::futures::FutureExt;

        Command::single(Action::Future(Box::pin(future.map(f))))
    }

    /// Creates a [`Command`] that performs the actions of all the given
    /// commands.
    ///
    /// Once this command is run, all the commands will be executed at once.
    pub fn batch(commands: impl IntoIterator<Item = Command<T>>) -> Self {
        Self(emg_futures::Command::batch(
            commands.into_iter().map(|Command(command)| command),
        ))
    }

    /// Applies a transformation to the result of a [`Command`].
    pub fn map<A>(self, f: impl Fn(T) -> A + 'static + MaybeSend + Sync + Clone) -> Command<A>
    where
        T: 'static,
    {
        let Command(command) = self;

        Command(command.map(move |action| action.map(f.clone())))
    }

    /// Returns all of the actions of the [`Command`].
    pub fn actions(self) -> Vec<Action<T>> {
        let Command(command) = self;

        command.actions()
    }
}

impl<T> fmt::Debug for Command<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Command(command) = self;

        command.fmt(f)
    }
}
