mod atom;
mod diagnostic;
pub mod event;
mod plugin;
mod xcb_event_systems;
mod xcb_request_systems;
mod xconn;

pub use plugin::XcbPlugin;

pub mod component {
    pub use xcb::x::Window;

    use crate::Region;

    /// Marks windows, layers, workspaces, screens,... as focused
    #[derive(Debug)]
    pub struct IsFocused;

    /// Marks windows without the `override_redirect` flag - windows that should
    /// be managed by the window manager
    #[derive(Debug)]
    pub struct IsManaged;

    /// Holds Region the window last reported as it's preffered dimensions, gets
    /// inserted by CreateNotify events and updated by ConfigureRequest events
    #[derive(Debug)]
    pub struct PrefferedSize(pub Region);

    /// Marks windows which are mapped.
    #[derive(Debug)]
    pub struct IsMapped;

    /// Current window or screen size
    #[derive(Debug)]
    pub struct Size(pub Region);
}

/// Requests are either components or events which are generated in the `Update`
/// stage and read in the `PostUpdate` stage and turned into XCB requests
pub mod request {
    use crate::Region;

    /// Requests the marked window entity to be mapped or unmapped
    #[derive(Debug)]
    pub enum RequestMap {
        Map,
        Unmap,
    }

    /// Requests the marked window entity to be resized and moved
    #[derive(Debug)]
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
