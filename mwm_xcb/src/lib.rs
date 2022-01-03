mod atom;
mod diagnostic;
pub mod event;
mod plugin;
mod xcb_event_systems;
mod xcb_event_type;
mod xcb_request_systems;
mod xconn;

pub use plugin::XcbPlugin;

pub mod component {
    use crate::Region;

    /// Newtype around `xcb_window_t` type which is just an alias and doesn't
    /// provide any type safety
    #[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
    pub struct XWinId(xcb::ffi::xproto::xcb_window_t);

    impl XWinId {
        pub(crate) fn from_raw(raw: xcb::ffi::xproto::xcb_window_t) -> XWinId {
            XWinId::from_raw_nullable(raw).expect("attempted to create NULL window id")
        }

        pub(crate) fn from_raw_nullable(raw: xcb::ffi::xproto::xcb_window_t) -> Option<XWinId> {
            if raw == xcb::base::NONE {
                None
            } else {
                Some(XWinId(raw))
            }
        }

        pub fn as_raw(self) -> xcb::ffi::xproto::xcb_window_t {
            self.0
        }
    }


    /// Marks windows, layers, workspaces, screens,... as focused
    pub struct IsFocused;

    /// Marks windows without the `override_redirect` flag - windows that should
    /// be managed by the window manager
    pub struct IsManaged;

    /// Holds Region the window last reported as it's preffered dimensions, gets
    /// inserted by CreateNotify events and updated by ConfigureRequest events
    pub struct PrefferedSize(pub Region);

    /// Marks windows which are mapped.
    pub struct IsMapped;

    /// Current window or screen size
    pub struct Size(pub Region);
}

/// Requests are either components or events which are generated in the `Update`
/// stage and read in the `PostUpdate` stage and turned into XCB requests
pub mod request {
    use crate::Region;

    /// Requests the marked window entity to be mapped or unmapped
    pub enum RequestMap {
        Map,
        Unmap,
    }

    /// Requests the marked window entity to be resized and moved
    pub struct RequestConfigure(pub Region);
}


#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct Region {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
}

impl Region {
    pub fn is_empty(&self) -> bool {
        self.w == 0 || self.h == 0
    }

    pub fn relative_center(&self) -> Point {
        let Region { w, h, .. } = *self;
        Point {
            x: i32::try_from(w / 2).unwrap(),
            y: i32::try_from(h / 2).unwrap(),
        }
    }
}

// #[derive(Debug, Clone)]
// pub enum ClientMessageData {
//     U8(Vec<u8>),
//     U16(Vec<u16>),
//     U32(Vec<u32>),
// }

/// X key-code with a modifier mask
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct KeyCode {
    /// modifier key bit mask
    pub mask: u16,
    /// X key code
    pub code: u8,
}

// impl KeyCode {
//     fn from_event(k: &xcb::KeyPressEvent) -> KeyCode {
//         KeyCode { mask: k.state(), code: k.detail() }
//     }
// }

// #[derive(Debug, Clone, Copy)]
// pub enum MouseEvent {
//     Press {
//         id: XWinId,
//         btn: MouseButton,
//         // TODO - this event could potentially use position too
//     },
//     Release {
//         btn: MouseButton,
//     },
//     Move {
//         // TODO - figure out what the events actually tell us
//     },
// }

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum MouseButton {
    Left = 1,
    Right = 2,
    Middle = 3,
    ScrollUp = 4,
    ScrollDown = 5,
}

// impl MouseButton {
//     fn from_detail(detail: u8) -> Option<MouseButton> {
//         match detail {
//             1 => Some(MouseButton::Left),
//             2 => Some(MouseButton::Right),
//             3 => Some(MouseButton::Middle),
//             4 => Some(MouseButton::ScrollUp),
//             5 => Some(MouseButton::ScrollDown),
//             _ => {
//                 warn!("received mouse event with an invalid button
// {detail}");                 None
//             },
//         }
//     }
// }

#[derive(Debug)]
pub(crate) struct Output {
    pub name: String,
    pub region: Region,
}
