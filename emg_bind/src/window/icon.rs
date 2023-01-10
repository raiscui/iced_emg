//! Attach an icon to the window of your application.
use std::fmt;
use std::io;

/// The icon of a window.
#[derive(Debug, Clone)]
pub struct Icon(emg_winit::winit::window::Icon);

impl Icon {
    /// Creates an icon from 32bpp RGBA data.
    pub fn from_rgba(rgba: Vec<u8>, width: u32, height: u32) -> Result<Self, Error> {
        let raw = emg_winit::winit::window::Icon::from_rgba(rgba, width, height)?;

        Ok(Self(raw))
    }
}

/// An error produced when using `Icon::from_rgba` with invalid arguments.
#[derive(Debug)]
pub enum Error {
    /// The provided RGBA data isn't divisble by 4.
    ///
    /// Therefore, it cannot be safely interpreted as 32bpp RGBA pixels.
    InvalidData {
        /// The length of the provided RGBA data.
        byte_count: usize,
    },

    /// The number of RGBA pixels does not match the provided dimensions.
    DimensionsMismatch {
        /// The provided width.
        width: u32,
        /// The provided height.
        height: u32,
        /// The amount of pixels of the provided RGBA data.
        pixel_count: usize,
    },

    /// The underlying OS failed to create the icon.
    Os(io::Error),
}

impl From<emg_winit::winit::window::BadIcon> for Error {
    fn from(error: emg_winit::winit::window::BadIcon) -> Self {
        use emg_winit::winit::window::BadIcon;

        match error {
            BadIcon::ByteCountNotDivisibleBy4 { byte_count } => Self::InvalidData { byte_count },
            BadIcon::DimensionsVsPixelCount {
                width,
                height,
                pixel_count,
                ..
            } => Self::DimensionsMismatch {
                width,
                height,
                pixel_count,
            },
            BadIcon::OsError(os_error) => Self::Os(os_error),
        }
    }
}

impl From<Icon> for emg_winit::winit::window::Icon {
    fn from(icon: Icon) -> Self {
        icon.0
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidData { byte_count } => {
                write!(
                    f,
                    "The provided RGBA data (with length {byte_count:?}) isn't divisble by \
                4. Therefore, it cannot be safely interpreted as 32bpp RGBA \
                pixels.",
                )
            }
            Self::DimensionsMismatch {
                width,
                height,
                pixel_count,
            } => {
                write!(
                    f,
                    "The number of RGBA pixels ({pixel_count:?}) does not match the provided \
                dimensions ({width:?}x{height:?}).",
                )
            }
            Self::Os(e) => write!(
                f,
                "The underlying OS failed to create the window \
                icon: {e:?}"
            ),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self)
    }
}
