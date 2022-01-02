use std::sync::atomic::{AtomicU8, Ordering};

use anyhow::{bail, Result};

/// Constant offset of the RANDR event(s?)
///
/// Gets updated when a connection is made, reset to 0xff when a connection is
/// severed.
static RANDR_BASE: AtomicU8 = AtomicU8::new(0xff);

pub fn set_randr_base(randr_base: u8) -> Result<()> {
    match RANDR_BASE.compare_exchange(0xff, randr_base, Ordering::AcqRel, Ordering::Acquire) {
        Ok(_) => Ok(()),
        Err(_) => bail!("an active XConn already exists"),
    }
}

pub fn reset_randr_base() {
    RANDR_BASE.store(0xff, Ordering::Release);
}

macro_rules! xcb_event_types {
    ( $($variant:ident = $value:path),* $(,)? ) => {
        #[repr(u8)]
        #[derive(Debug, Clone, Copy)]
        pub enum XcbEventType {
            $( $variant = $value, )*
            RandrNotify,
        }

        impl TryFrom<u8> for XcbEventType {
            type Error = u8;

            fn try_from(etype: u8) -> Result<Self, Self::Error> {
                // Why? Why is this getting masked. Penrose and the XCB docs do this but there is no explanation.
                let etype = etype & !0x80;

                match etype {
                    $( $value => Ok(XcbEventType::$variant), )*
                    unknown => {

                        // RandR notification type isn't constant so we can't match on it directly
                        let randr_base = RANDR_BASE.load(Ordering::Acquire);
                        if randr_base != 0xff {
                            match unknown.checked_sub(randr_base) {
                                Some(xcb::randr::NOTIFY) => return Ok(XcbEventType::RandrNotify),
                                _ => {},
                            }
                        }
                        // RANDR is not loaded, unconditionally return an error
                        Err(unknown)
                    }
                }
            }
        }
    };
}

// This the enum that xcb itself should've defined but noo, C doesn't have to
// list the values a magic constant can have, they can just have seemingly
// unrelated `#define`s thrown into a file and users should just _know_ that
// they're related.
xcb_event_types! {
    KeyPress = xcb::xproto::KEY_PRESS,
    KeyRelease = xcb::xproto::KEY_RELEASE,
    ButtonPress = xcb::xproto::BUTTON_PRESS,
    ButtonRelease = xcb::xproto::BUTTON_RELEASE,
    MotionNotify = xcb::xproto::MOTION_NOTIFY,
    EnterNotify = xcb::xproto::ENTER_NOTIFY,
    LeaveNotify = xcb::xproto::LEAVE_NOTIFY,
    FocusIn = xcb::xproto::FOCUS_IN,
    FocusOut = xcb::xproto::FOCUS_OUT,
    KeymapNotify = xcb::xproto::KEYMAP_NOTIFY,
    Expose = xcb::xproto::EXPOSE,
    GraphicsExposure = xcb::xproto::GRAPHICS_EXPOSURE,
    NoExposure = xcb::xproto::NO_EXPOSURE,
    VisibilityNotify = xcb::xproto::VISIBILITY_NOTIFY,
    CreateNotify = xcb::xproto::CREATE_NOTIFY,
    DestroyNotify = xcb::xproto::DESTROY_NOTIFY,
    UnmapNotify = xcb::xproto::UNMAP_NOTIFY,
    MapNotify = xcb::xproto::MAP_NOTIFY,
    MapRequest = xcb::xproto::MAP_REQUEST,
    ReparentNotify = xcb::xproto::REPARENT_NOTIFY,
    ConfigureNotify = xcb::xproto::CONFIGURE_NOTIFY,
    ConfigureRequest = xcb::xproto::CONFIGURE_REQUEST,
    GravityNotify = xcb::xproto::GRAVITY_NOTIFY,
    ResizeRequest = xcb::xproto::RESIZE_REQUEST,
    CirculateNotify = xcb::xproto::CIRCULATE_NOTIFY,
    CirculateRequest = xcb::xproto::CIRCULATE_REQUEST,
    PropertyNotify = xcb::xproto::PROPERTY_NOTIFY,
    SelectionClear = xcb::xproto::SELECTION_CLEAR,
    SelectionRequest = xcb::xproto::SELECTION_REQUEST,
    SelectionNotify = xcb::xproto::SELECTION_NOTIFY,
    ColormapNotify = xcb::xproto::COLORMAP_NOTIFY,
    ClientMessage = xcb::xproto::CLIENT_MESSAGE,
    MappingNotify = xcb::xproto::MAPPING_NOTIFY,
    Generic = xcb::xproto::GE_GENERIC,

    // requires the `randr` feature of `xcb` enabled
    ScreenChangeNotify = xcb::randr::SCREEN_CHANGE_NOTIFY,
}
