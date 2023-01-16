use bitflags::bitflags;
use std::path::PathBuf;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct EventFlag: u32 {
        const MOVED =               1<<0;
        const RESIZED =             1<<1;
        const CLOSE_REQUESTED =     1<<2;
        const FOCUSED =             1<<3 ;
        const UNFOCUSED =           1<<4 ;
        const FILE_HOVERED =        1<<5 ;
        const FILE_DROPPED =        1<<6 ;
        const FILES_HOVERED_LEFT =  1<<7 ;

    }
}

/// A window-related event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    /// A window was moved.
    Moved {
        /// The new logical x location of the window
        x: i32,
        /// The new logical y location of the window
        y: i32,
    },

    /// A window was resized.
    Resized {
        /// The new logical width of the window
        width: u32,
        /// The new logical height of the window
        height: u32,
    },

    /// The user has requested for the window to close.
    ///
    /// Usually, you will want to terminate the execution whenever this event
    /// occurs.
    CloseRequested,

    /// A window was focused.
    Focused,

    /// A window was unfocused.
    Unfocused,

    /// A file is being hovered over the window.
    ///
    /// When the user hovers multiple files at once, this event will be emitted
    /// for each file separately.
    FileHovered(PathBuf),

    /// A file has beend dropped into the window.
    ///
    /// When the user drops multiple files at once, this event will be emitted
    /// for each file separately.
    FileDropped(PathBuf),

    /// A file was hovered, but has exited the window.
    ///
    /// There will be a single `FilesHoveredLeft` event triggered even if
    /// multiple files were hovered.
    FilesHoveredLeft,
}
