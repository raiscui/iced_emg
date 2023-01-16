/// The interaction of a mouse cursor.
#[derive(Default,Debug, Eq, PartialEq, Clone, Copy, PartialOrd, Ord)]
pub enum Interaction {
    #[default]
    Idle,
    Pointer,
    Grab,
    Text,
    Crosshair,
    Working,
    Grabbing,
    ResizingHorizontally,
    ResizingVertically,
}

