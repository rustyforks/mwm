mod atom;
mod diagnostic;
pub mod event;
mod plugin;
mod xcb_event_systems;
mod xcb_request_systems;
mod xconn;

pub use plugin::XcbPlugin;

pub mod component {
    use std::fmt::{self, Debug};

    use bevy_ecs::component::Component;

    use crate::Region;

    /// Wrapper for [`xcb::x::Window`] implementing the `Component` trait
    #[derive(Component, Clone, Copy)]
    pub struct Window(pub xcb::x::Window);

    impl Debug for Window {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            Debug::fmt(&self.0, f)
        }
    }

    impl PartialEq<xcb::x::Window> for Window {
        fn eq(&self, other: &xcb::x::Window) -> bool {
            self.0 == *other
        }
    }

    /// Marks windows, layers, workspaces, screens,... as focused
    #[derive(Component, Debug)]
    pub struct IsFocused;

    /// Marks windows without the `override_redirect` flag - windows that should
    /// be managed by the window manager
    #[derive(Component, Debug)]
    pub struct IsManaged;

    /// Holds Region the window last reported as it's preffered dimensions, gets
    /// inserted by CreateNotify events and updated by ConfigureRequest events
    #[derive(Component, Debug)]
    pub struct PrefferedSize(pub Region);

    /// Holds the window's last reported preffered border width, gets
    /// inserted by CreateNotify events and updated by ConfigureRequest events
    #[derive(Component, Debug)]
    pub struct PrefferedBorder(pub u16);

    /// Marks windows which are mapped.
    #[derive(Component, Debug)]
    pub struct IsMapped;

    /// Current window or screen size
    #[derive(Component, Debug)]
    pub struct Size(pub Region);

    #[derive(Component, Debug)]
    pub struct Border(pub u16);
}

/// Requests are either components or events which are generated in the `Update`
/// stage and read in the `PostUpdate` stage and turned into XCB requests
pub mod request {
    use bevy_ecs::component::Component;

    use crate::Region;

    /// Requests the marked window entity to be mapped or unmapped
    #[derive(Component, Debug)]
    pub enum RequestMap {
        Map,
        Unmap,
    }

    /// Requests the marked window entity to be resized and moved
    #[derive(Component, Debug)]
    pub struct RequestSize(pub Region);

    /// Requests the marked window entity to have a border set
    #[derive(Component, Debug)]
    pub struct RequestBorder(pub u16);
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
