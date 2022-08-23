/*
 * @Author: Rais
 * @Date: 2022-08-12 14:17:14
 * @LastEditTime: 2022-08-12 14:21:44
 * @LastEditors: Rais
 * @Description:
 */
//! Access the clipboard.
pub use emg_native::clipboard::Action;
use tracing::warn;

use crate::command::{self, Command};

/// A buffer for short-term storage and transfer within and between
/// applications.
#[allow(missing_debug_implementations)]
pub struct Clipboard {
    state: State,
}

enum State {
    Connected(window_clipboard::Clipboard),
    Unavailable,
}

impl Clipboard {
    /// Creates a new [`Clipboard`] for the given window.
    pub fn connect(window: &winit::window::Window) -> Clipboard {
        let state = window_clipboard::Clipboard::connect(window)
            .ok()
            .map(State::Connected)
            .unwrap_or(State::Unavailable);

        Clipboard { state }
    }

    /// Creates a new [`Clipboard`] that isn't associated with a window.
    /// This clipboard will never contain a copied value.
    pub fn unconnected() -> Clipboard {
        Clipboard {
            state: State::Unavailable,
        }
    }

    /// Reads the current content of the [`Clipboard`] as text.
    pub fn read(&self) -> Option<String> {
        match &self.state {
            State::Connected(clipboard) => clipboard.read().ok(),
            State::Unavailable => None,
        }
    }

    /// Writes the given text contents to the [`Clipboard`].
    pub fn write(&mut self, contents: String) {
        match &mut self.state {
            State::Connected(clipboard) => match clipboard.write(contents) {
                Ok(()) => {}
                Err(error) => {
                    warn!("error writing to clipboard: {}", error)
                }
            },
            State::Unavailable => {}
        }
    }
}

impl emg_native::Clipboard for Clipboard {
    fn read(&self) -> Option<String> {
        self.read()
    }

    fn write(&mut self, contents: String) {
        self.write(contents)
    }
}

/// Read the current contents of the clipboard.
pub fn read<Message>(f: impl Fn(Option<String>) -> Message + 'static) -> Command<Message> {
    Command::single(command::Action::Clipboard(Action::Read(Box::new(f))))
}

/// Write the given contents to the clipboard.
pub fn write<Message>(contents: String) -> Command<Message> {
    Command::single(command::Action::Clipboard(Action::Write(contents)))
}
