use xcb::{randr, x};

macro_rules! event_wrappers {
    ( $( $w:ident $e:path ),+ $(,)? ) => {
        $(
            #[derive(Debug)]
            pub struct $w(pub(crate) $e);

            impl std::ops::Deref for $w {
                type Target = $e;
                fn deref(&self) -> &Self::Target {
                    &self.0
                }
            }
        )*
    };
}

event_wrappers! {
    KeyPress            x::KeyPressEvent,
    KeyRelease          x::KeyReleaseEvent,
    ButtonPress         x::ButtonPressEvent,
    ButtonRelease       x::ButtonReleaseEvent,
    MotionNotify        x::MotionNotifyEvent,
    EnterNotify         x::EnterNotifyEvent,
    LeaveNotify         x::LeaveNotifyEvent,
    FocusIn             x::FocusInEvent,
    FocusOut            x::FocusOutEvent,
    KeymapNotify        x::KeymapNotifyEvent,
    Expose              x::ExposeEvent,
    GraphicsExposure    x::GraphicsExposureEvent,
    NoExposure          x::NoExposureEvent,
    VisibilityNotify    x::VisibilityNotifyEvent,
    CreateNotify        x::CreateNotifyEvent,
    DestroyNotify       x::DestroyNotifyEvent,
    UnmapNotify         x::UnmapNotifyEvent,
    MapNotify           x::MapNotifyEvent,
    MapRequest          x::MapRequestEvent,
    ReparentNotify      x::ReparentNotifyEvent,
    ConfigureNotify     x::ConfigureNotifyEvent,
    ConfigureRequest    x::ConfigureRequestEvent,
    GravityNotify       x::GravityNotifyEvent,
    ResizeRequest       x::ResizeRequestEvent,
    CirculateNotify     x::CirculateNotifyEvent,
    CirculateRequest    x::CirculateRequestEvent,
    PropertyNotify      x::PropertyNotifyEvent,
    SelectionClear      x::SelectionClearEvent,
    SelectionRequest    x::SelectionRequestEvent,
    SelectionNotify     x::SelectionNotifyEvent,
    ColormapNotify      x::ColormapNotifyEvent,
    ClientMessage       x::ClientMessageEvent,
    MappingNotify       x::MappingNotifyEvent,
    ScreenChangeNotify  randr::ScreenChangeNotifyEvent,
    Notify              randr::NotifyEvent,
}
