macro_rules! atoms {
    ( $( $variant_name:ident = $variant_str:expr ),+ $(,)? ) => {
        #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
        pub enum Atom {
            $( $variant_name ),+
        }

        impl Atom {
            pub fn as_str(self) -> &'static str {
                match self {
                    $( Atom::$variant_name => $variant_str ),+
                }
            }

            pub const ALL: &'static [Atom] = &[
                $( Atom::$variant_name ),+
            ];
        }
    };
}

atoms! {
    Manager                      = "MANAGER",
    UTF8String                   = "UTF8_STRING",
    WmClass                      = "WM_CLASS",
    // WmDeleteWindow               = "WM_DELETE_WINDOW",
    WmProtocols                  = "WM_PROTOCOLS",
    // WmState                      = "WM_STATE",
    WmName                       = "WM_NAME",
    // WmTakeFocus                  = "WM_TAKE_FOCUS",
    NetActiveWindow              = "_NET_ACTIVE_WINDOW",
    // NetClientList                = "_NET_CLIENT_LIST",
    // NetCurrentDesktop            = "_NET_CURRENT_DESKTOP",
    // NetDesktopNames              = "_NET_DESKTOP_NAMES",
    // NetNumberOfDesktops          = "_NET_NUMBER_OF_DESKTOPS",
    // NetSupported                 = "_NET_SUPPORTED",
    // NetSupportingWmCheck         = "_NET_SUPPORTING_WM_CHECK",
    // NetSystemTrayOpcode          = "_NET_SYSTEM_TRAY_OPCODE",
    // NetSystemTrayOrientation     = "_NET_SYSTEM_TRAY_ORIENTATION",
    // NetSystemTrayOrientationHorz = "_NET_SYSTEM_TRAY_ORIENTATION_HORZ",
    // NetSystemTrayS0              = "_NET_SYSTEM_TRAY_S0",
    // NetWmDesktop                 = "_NET_WM_DESKTOP",
    NetWmName                    = "_NET_WM_NAME",
    NetWmState                   = "_NET_WM_STATE",
    NetWmStateFullscreen         = "_NET_WM_STATE_FULLSCREEN",
    // NetWmWindowType              = "_NET_WM_WINDOW_TYPE",
    // XEmbed                       = "_XEMBED",
    // XEmbedInfo                   = "_XEMBED_INFO",

    // NetWindowTypeDesktop         = "_NET_WM_WINDOW_TYPE_DESKTOP",
    // NetWindowTypeDock            = "_NET_WM_WINDOW_TYPE_DOCK",
    // NetWindowTypeToolbar         = "_NET_WM_WINDOW_TYPE_TOOLBAR",
    // NetWindowTypeMenu            = "_NET_WM_WINDOW_TYPE_MENU",
    // NetWindowTypeUtility         = "_NET_WM_WINDOW_TYPE_UTILITY",
    // NetWindowTypeSplash          = "_NET_WM_WINDOW_TYPE_SPLASH",
    // NetWindowTypeDialog          = "_NET_WM_WINDOW_TYPE_DIALOG",
    // NetWindowTypeDropdownMenu    = "_NET_WM_WINDOW_TYPE_DROPDOWN_MENU",
    // NetWindowTypePopupMenu       = "_NET_WM_WINDOW_TYPE_POPUP_MENU",
    // NetWindowTypeNotification    = "_NET_WM_WINDOW_TYPE_NOTIFICATION",
    // NetWindowTypeCombo           = "_NET_WM_WINDOW_TYPE_COMBO",
    // NetWindowTypeDnd             = "_NET_WM_WINDOW_TYPE_DND",
    // NetWindowTypeNormal          = "_NET_WM_WINDOW_TYPE_NORMAL",
}
