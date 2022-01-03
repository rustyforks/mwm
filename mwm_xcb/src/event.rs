use crate::component::XWinId;
use crate::{Point, Region};

// #[derive(Debug)]
// pub struct ClientMessage {
//     /// The ID of the window that sent the message
//     pub id: XWinId,
//     /// The data type being set
//     pub dtype: String,
//     /// The data itself
//     pub data: ClientMessageData,
// }

// #[derive(Debug)]
// pub struct PropertyNotify {
//     /// The ID of the window that had a property changed
//     pub id: XWinId,
//     /// The property that changed
//     pub atom: String,
//     /// Is this window the root window?
//     pub is_root: bool,
// }

// #[derive(Debug)]
// pub struct RandrNotify;

// #[derive(Debug)]
// pub struct ScreenChange;

#[derive(Debug)]
pub struct ButtonPress {
    pub detail: xcb::Button,
    pub time: xcb::Timestamp,
    pub root: XWinId,
    pub event: XWinId,
    pub child: XWinId,
    pub root_pos: Point,
    pub event_pos: Point,
    pub state: u16,
    pub same_screen: bool,
}

#[derive(Debug)]
pub struct ButtonRelease {
    pub detail: xcb::Button,
    pub time: xcb::Timestamp,
    pub root: XWinId,
    pub event: XWinId,
    pub child: XWinId,
    pub root_pos: Point,
    pub event_pos: Point,
    pub state: u16,
    pub same_screen: bool,
}

#[derive(Debug)]
pub struct ConfigureNotify {
    pub event: XWinId,
    pub window: XWinId,
    pub above_sibling: Option<XWinId>,
    pub region: Region,
    pub border_width: u16,
    pub override_redirect: bool,
}

#[derive(Debug)]
pub struct ConfigureRequest {
    pub stack_mode: u8,
    pub parent: XWinId,
    pub window: XWinId,
    pub sibling: Option<XWinId>,
    pub region: Region,
    pub border_width: u16,
    pub value_mask: u16,
}

#[derive(Debug)]
pub struct CreateNotify {
    pub parent: XWinId,
    pub window: XWinId,
    pub region: Region,
    pub border_width: u16,
    pub override_redirect: bool,
}

#[derive(Debug)]
pub struct DestroyNotify {
    pub event: XWinId,
    pub window: XWinId,
}

#[derive(Debug)]
pub struct EnterNotify {
    pub detail: u8,
    pub time: xcb::Timestamp,
    pub root: XWinId,
    pub event: XWinId,
    pub child: XWinId,
    pub root_pos: Point,
    pub event_pos: Point,
    pub state: u16,
    pub mode: u8,
    pub same_screen_focus: u8,
}

#[derive(Debug)]
pub struct FocusIn {
    pub detail: u8,
    pub event: XWinId,
    pub mode: u8,
}

#[derive(Debug)]
pub struct FocusOut {
    pub detail: u8,
    pub event: XWinId,
    pub mode: u8,
}

#[derive(Debug)]
pub struct KeyPress {
    pub detail: xcb::Keycode,
    pub time: xcb::Timestamp,
    pub root: XWinId,
    pub event: XWinId,
    pub child: XWinId,
    pub root_pos: Point,
    pub event_pos: Point,
    pub state: u16,
    pub same_screen: bool,
}

#[derive(Debug)]
pub struct LeaveNotify {
    pub detail: u8,
    pub time: xcb::Timestamp,
    pub root: XWinId,
    pub event: XWinId,
    pub child: XWinId,
    pub root_pos: Point,
    pub event_pos: Point,
    pub state: u16,
    pub mode: u8,
    pub same_screen_focus: u8,
}

#[derive(Debug)]
pub struct MapNotify {
    pub event: XWinId,
    pub window: XWinId,
    pub override_redirect: bool,
}

#[derive(Debug)]
pub struct MapRequest {
    pub parent: XWinId,
    pub window: XWinId,
}

#[derive(Debug)]
pub struct MotionNotify {
    pub detail: xcb::Keycode,
    pub time: xcb::Timestamp,
    pub root: XWinId,
    pub event: XWinId,
    pub child: XWinId,
    pub root_pos: Point,
    pub event_pos: Point,
    pub state: u16,
    pub same_screen: bool,
}

// NOTE Not an XCB event, this is our virtual event used to initially add a
// single detected screen at when th WM is started. This should be eventually
// replaced with properly parsed XrandR events.
#[derive(Debug)]
pub struct ScreenAdded {
    pub name: String,
    pub region: Region,
}

#[derive(Debug)]
pub struct UnmapNotify {
    pub event: XWinId,
    pub window: XWinId,
    pub from_configure: bool,
}
