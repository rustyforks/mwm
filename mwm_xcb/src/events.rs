use crate::{ClientMessageData, KeyCode, MouseButton, Point, Region, XWinId};


#[derive(Debug, Clone)]
pub struct ButtonPress {
    /// Internal ID of the window that was clicked.
    pub id: XWinId,
    /// Relevant mouse button.
    pub btn: MouseButton,
}

#[derive(Debug, Clone)]
pub struct ButtonRelease {
    /// Relevant mouse button.
    pub btn: MouseButton,
}

#[derive(Debug, Clone)]
pub struct ClientMessage {
    /// The ID of the window that sent the message
    pub id: XWinId,
    /// The data type being set
    pub dtype: String,
    /// The data itself
    pub data: ClientMessageData,
}

// #[derive(Debug, Clone)]
// pub struct ConfigureNotify {
//     pub event: XWinId,
//     pub window: XWinId,
//     pub above_sibling: Option<XWinId>,
//     pub region: Region,
//     pub border_width: u16,
//     pub override_redirect: bool,
//     pub is_root: bool,
// }

#[derive(Debug, Clone)]
pub struct ConfigureRequest {
    pub stack_mode: u8,
    pub parent: XWinId,
    pub window: XWinId,
    pub sibling: Option<XWinId>,
    pub region: Region,
    pub border_width: u16,
    pub value_mask: u16,
    pub is_root: bool,
}

#[derive(Debug, Clone)]
pub struct CreateNotify {
    pub parent: XWinId,
    pub window: XWinId,
    pub region: Region,
    pub border_width: u16,
    pub override_redirect: bool,
}

#[derive(Debug, Clone)]
pub struct DestroyNotify {
    pub event: XWinId,
    pub window: XWinId,
}

#[derive(Debug, Clone)]
pub struct Enter {
    /// The ID of the window that was entered
    pub id: XWinId,
    // /// Absolute coordinate of the event
    // pub rpt: Point,
    // /// Coordinate of the event relative to top-left of the window itself
    // pub wpt: Point,
}

#[derive(Debug, Clone)]
pub struct FocusIn {
    /// The ID of the window that gained focus
    pub id: XWinId,
}

#[derive(Debug, Clone)]
pub struct FocusOut {
    /// The ID of the window that lost focus
    pub id: XWinId,
}

#[derive(Debug, Clone)]
pub struct KeyPress {
    /// Received key.
    pub key: KeyCode,
}

#[derive(Debug, Clone)]
pub struct Leave {
    /// The ID of the window that was left
    pub id: XWinId,
    // /// Absolute coordinate of the event
    // pub rpt: Point,
    // /// Coordinate of the event relative to top-left of the window itself
    // pub wpt: Point,
}

#[derive(Debug, Clone)]
pub struct MapRequest {
    pub parent: XWinId,
    pub window: XWinId,
}

#[derive(Debug, Clone)]
pub struct MotionNotify {
    /// Internal ID of the window that was moved across.
    pub id: XWinId,
    /// Absolute coordinate of the event.
    pub rpt: Point,
    /// Coordinate of the event relative to top-left of the window itself.
    pub wpt: Point,
}

#[derive(Debug, Clone)]
pub struct PropertyNotify {
    /// The ID of the window that had a property changed
    pub id: XWinId,
    /// The property that changed
    pub atom: String,
    /// Is this window the root window?
    pub is_root: bool,
}

#[derive(Debug, Clone)]
pub struct RandrNotify;

#[derive(Debug, Clone)]
pub struct ScreenChange;
