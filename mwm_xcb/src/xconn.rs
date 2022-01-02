#![allow(dead_code)]

use anyhow::{bail, Context, Result};
use log::{error, info};
use rustc_hash::FxHashMap as HashMap;

use crate::atom::Atom;
use crate::component::XWinId;
use crate::{xcb_event_type, KeyCode, MouseButton, Output, Point, Region};

/// data for abstracting communication with the X server via xcb
pub struct XConn {
    conn: xcb::Connection,

    root: XWinId,
    check_win: XWinId,

    // interned atoms
    atoms: HashMap<Atom, xcb::ffi::xproto::xcb_atom_t>,
}

impl Drop for XConn {
    fn drop(&mut self) {
        // release any of the keybindings we are holding on to
        xcb::ungrab_key(
            &self.conn, // xcb connection to X11
            xcb::GRAB_ANY as u8,
            self.root.as_raw(), // the window to ungrab keys for
            xcb::MOD_MASK_ANY as u16,
        );

        // destroy the check window
        xcb::destroy_window(&self.conn, self.check_win.as_raw());

        // mark ourselves as no longer being the active root window
        xcb::delete_property(
            &self.conn,
            self.root.as_raw(),
            self.atom_id(Atom::NetActiveWindow),
        );

        self.flush();

        // remove the RANDR_BASE constant to allow another connection to be created
        xcb_event_type::reset_randr_base();
    }
}

impl XConn {
    /// Establish a new connection to the running X server. Fails if unable to
    /// connect
    pub(super) fn init() -> Result<XConn> {
        let (conn, _) = xcb::Connection::connect(None).context("connecting to X server")?;

        let root = XWinId::from_raw(
            conn.get_setup()
                .roots()
                .next()
                .context("getting handle for screen")?
                .root(),
        );

        let atom_cookies = Atom::ALL
            .iter()
            .map(|atom| xcb::intern_atom(&conn, false, atom.as_str()))
            .collect::<Vec<_>>();

        let check_win = XWinId::from_raw(conn.generate_id());

        let create_window_cookie = xcb::create_window(
            &conn,              // xcb connection to X11
            0,                  // new window's depth
            check_win.as_raw(), // ID to be used for referring to the window
            root.as_raw(),      // parent window
            0,                  // x-coordinate
            0,                  // y-coordinate
            1,                  // width
            1,                  // height
            0,                  // border width
            xcb::xproto::WINDOW_CLASS_INPUT_ONLY as u16,
            xcb::base::COPY_FROM_PARENT, // visual
            &[],                         // configuration (mask, value) list
        );

        let select_input_cookie = xcb::randr::select_input(
            &conn,
            root.as_raw(),
            (xcb::randr::NOTIFY_MASK_CRTC_CHANGE | xcb::randr::NOTIFY_MASK_SCREEN_CHANGE) as u16,
        );

        let randr_base = conn
            .get_extension_data(&mut xcb::randr::id())
            .context("fetching randr extension data")?
            .first_event();
        xcb_event_type::set_randr_base(randr_base)?;

        let atoms = {
            let replies = atom_cookies.into_iter().map(|cookie| {
                cookie
                    .get_reply()
                    .map(|r| r.atom())
                    .context("interning atom")
            });
            Atom::ALL
                .iter()
                .copied()
                .zip(replies)
                .map(|(a, r)| r.map(|r| (a, r)))
                .collect::<Result<_>>()?
        };
        create_window_cookie.request_check()?;
        select_input_cookie.request_check()?;

        Ok(XConn { conn, root, check_win, atoms })
    }

    pub fn root(&self) -> XWinId {
        self.root
    }

    pub fn atom_id(&self, atom: Atom) -> xcb::ffi::xproto::xcb_atom_t {
        *self.atoms.get(&atom).unwrap()
    }

    pub fn wait_for_event(&self) -> Option<xcb::GenericEvent> {
        self.conn.wait_for_event()
    }

    pub fn poll_for_event(&self) -> Option<xcb::GenericEvent> {
        self.conn.poll_for_event()
    }

    pub fn flush(&self) -> bool {
        self.conn.flush()
    }

    pub fn get_window_attributes(
        &self,
        id: XWinId,
    ) -> Result<xcb::xproto::GetWindowAttributesReply> {
        xcb::xproto::get_window_attributes(&self.conn, id.as_raw())
            .get_reply()
            .with_context(|| format!("get attributes for window {id:?}"))
    }

    /// query geometry for Window
    pub fn get_window_geometry(&self, id: XWinId) -> Result<Region> {
        let res = xcb::get_geometry(&self.conn, id.as_raw()).get_reply()?;
        Ok(Region {
            x: res.x().into(),
            y: res.y().into(),
            w: res.width().into(),
            h: res.height().into(),
        })
    }

    pub(crate) fn current_outputs(&self) -> Vec<Output> {
        let reply =
            xcb::randr::get_screen_resources(&self.conn, self.check_win.as_raw()).get_reply();
        let resources = match reply {
            Ok(resources) => resources,
            Err(e) => {
                error!("error reading X screen resources: {}", e);
                return Vec::new();
            },
        };

        resources
            .outputs()
            .iter()
            .filter_map(|&output| {
                let reply = xcb::randr::get_output_info(&self.conn, output, 0).get_reply();
                let output = match reply {
                    Ok(output) => output,
                    Err(e) => {
                        error!("error reading X output info: {}", e);
                        return None;
                    },
                };
                let reply = xcb::randr::get_crtc_info(&self.conn, output.crtc(), 0).get_reply();
                let crtc = match reply {
                    Ok(crtc) => crtc,
                    Err(e) => {
                        error!("error reading X crtc info: {}", e);
                        return None;
                    },
                };

                let name = String::from_utf8_lossy(output.name()).to_string();
                let region = Region {
                    x: crtc.x().into(),
                    y: crtc.y().into(),
                    w: crtc.width().into(),
                    h: crtc.height().into(),
                };
                if region.is_empty() {
                    info!("crtc {} has zero dimensions", output.crtc());
                    return None;
                }

                Some(Output { name, region })
            })
            .collect()
    }

    pub fn cursor_position(&self) -> Option<Point> {
        let reply = xcb::query_pointer(&self.conn, self.root.as_raw())
            .get_reply()
            .ok()?;
        Some(Point {
            x: reply.root_x().into(),
            y: reply.root_y().into(),
        })
    }

    pub fn position_window(&self, id: XWinId, r: Region, border: u32) {
        let Region { x, y, w, h } = r;
        xcb::configure_window(&self.conn, id.as_raw(), &[
            (xcb::CONFIG_WINDOW_X as u16, x.try_into().unwrap()),
            (xcb::CONFIG_WINDOW_Y as u16, y.try_into().unwrap()),
            (xcb::CONFIG_WINDOW_WIDTH as u16, w),
            (xcb::CONFIG_WINDOW_HEIGHT as u16, h),
            (xcb::CONFIG_WINDOW_BORDER_WIDTH as u16, border),
        ]);
    }

    // TODO - rename this function...
    pub fn mark_new_window(&self, id: XWinId) {
        xcb::change_window_attributes(&self.conn, id.as_raw(), &[(
            xcb::CW_EVENT_MASK,
            xcb::EVENT_MASK_ENTER_WINDOW
                | xcb::EVENT_MASK_LEAVE_WINDOW
                | xcb::EVENT_MASK_PROPERTY_CHANGE,
        )]);
    }

    pub fn map_window(&self, id: XWinId) {
        // TODO error handling
        xcb::map_window(&self.conn, id.as_raw());
    }

    pub fn unmap_window(&self, id: XWinId) {
        // TODO error handling
        xcb::unmap_window(&self.conn, id.as_raw());
    }

    pub fn send_client_event(&self, id: XWinId, atom: Atom) -> Result<()> {
        let wm_protocols = self.atom_id(Atom::WmProtocols);
        let data =
            xcb::ClientMessageData::from_data32([self.atom_id(atom), xcb::CURRENT_TIME, 0, 0, 0]);
        let event = xcb::ClientMessageEvent::new(32, id.as_raw(), wm_protocols, data);
        xcb::send_event(
            &self.conn,
            false,
            id.as_raw(),
            xcb::EVENT_MASK_NO_EVENT,
            &event,
        );
        Ok(())
    }

    pub fn focus_client(&self, id: XWinId) {
        // TODO error handling
        xcb::set_input_focus(
            &self.conn,                    // xcb connection to X11
            xcb::INPUT_FOCUS_PARENT as u8, // focus the parent when focus is lost
            id.as_raw(),                   // window to focus
            0,                             /* current time to avoid network race conditions (0
                                            * == current time) */
        );
        xcb::change_property(
            &self.conn,                          // xcb connection to X11
            xcb::PROP_MODE_REPLACE as u8,        // discard current prop and replace
            self.root.as_raw(),                  // window to change prop on
            self.atom_id(Atom::NetActiveWindow), // prop to change
            xcb::xproto::ATOM_WINDOW,            // type of prop
            32,                                  // data format (8/16/32-bit)
            &[id.as_raw()],                      // data
        );
    }

    pub fn set_client_border_color(&self, id: XWinId, color: u32) {
        xcb::change_window_attributes(&self.conn, id.as_raw(), &[(xcb::CW_BORDER_PIXEL, color)]);
    }

    pub fn set_client_fullscreen(&self, id: XWinId, enable_fullscreen: bool) {
        let data;
        xcb::change_property(
            &self.conn,                     // xcb connection to X11
            xcb::PROP_MODE_REPLACE as u8,   // discard current prop and replace
            id.as_raw(),                    // window to change prop on
            self.atom_id(Atom::NetWmState), // prop to change
            xcb::xproto::ATOM_ATOM,         // type of prop
            32,                             // data format (8/16/32-bit)
            if enable_fullscreen {
                data = [self.atom_id(Atom::NetWmStateFullscreen)];
                &data
            } else {
                &[]
            },
        );
    }

    pub fn grab_key(&self, key: KeyCode) {
        info!("grabbing {:?}", key);
        // TODO error handling
        xcb::grab_key(
            &self.conn,                 // xcb connection to X11
            false,                      // don't pass grabbed events through to the client
            self.root.as_raw(),         // the window to grab: in this case the root window
            key.mask,                   // modifiers to grab
            key.code,                   // keycode to grab
            xcb::GRAB_MODE_ASYNC as u8, // don't lock pointer input while grabbing
            xcb::GRAB_MODE_ASYNC as u8, // don't lock keyboard input while grabbing
        );
    }

    pub fn ungrab_key(&self, key: KeyCode) {
        // TODO error handling
        xcb::ungrab_key(
            &self.conn,         // xcb connection to X11
            key.code,           // keycode to grab
            self.root.as_raw(), // the window to grab: in this case the root window
            key.mask,           // modifiers to grab
        );
    }

    pub fn grab_button(&self, button: MouseButton) {
        // TODO error handling
        xcb::grab_button(
            &self.conn,         // xcb connection to X11
            false,              // don't pass grabbed events through to the client
            self.root.as_raw(), // the window to grab: in this case the root window
            (xcb::EVENT_MASK_BUTTON_PRESS
                | xcb::EVENT_MASK_BUTTON_RELEASE
                | xcb::EVENT_MASK_POINTER_MOTION) as u16, // which events are reported to the client
            xcb::GRAB_MODE_ASYNC as u8, // don't lock pointer input while grabbing
            xcb::GRAB_MODE_ASYNC as u8, // don't lock keyboard input while grabbing
            xcb::NONE,          // don't confine the cursor to a specific window
            xcb::NONE,          // don't change the cursor type
            button as u8,       // the button to grab
            xcb::MOD_MASK_1 as u16, // modifiers to grab
        );
    }

    pub fn substructure_redirect(&self) -> Result<()> {
        xcb::change_window_attributes(&self.conn, self.root.as_raw(), &[(
            xcb::CW_EVENT_MASK,
            xcb::EVENT_MASK_PROPERTY_CHANGE
                | xcb::EVENT_MASK_SUBSTRUCTURE_REDIRECT
                | xcb::EVENT_MASK_SUBSTRUCTURE_NOTIFY,
        )])
        .request_check()
        .context("subscribing to substructure_redirect events")
    }

    pub fn warp_cursor_window(&self, id: XWinId) {
        let center = self.get_window_geometry(id).unwrap().relative_center();
        self.warp_cursor(center);
    }

    pub fn warp_cursor(&self, point: Point) {
        // TODO error handling
        xcb::warp_pointer(
            &self.conn,         // xcb connection to X11
            0,                  // source window
            self.root.as_raw(), // destination window
            0,                  // source x
            0,                  // source y
            0,                  // source width
            0,                  // source height
            point.x as i16,     // destination x
            point.y as i16,     // destination y
        );
    }

    pub fn get_string_property(&self, id: XWinId, property: Atom) -> Result<String> {
        let cookie = xcb::get_property(
            &self.conn,             // xcb connection to X11
            false,                  // should the property be deleted
            id.as_raw(),            // target window to query
            self.atom_id(property), // the property we want
            xcb::ATOM_ANY,          // the type of the property
            0,                      // offset in the property to retrieve data from
            1024,                   // how many 32bit multiples of data to retrieve
        );
        let reply = cookie.get_reply().with_context(|| {
            format!("unable to get string property {id:?} for window {property:?}")
        })?;

        Ok(String::from_utf8_lossy(reply.value()).to_string())
    }

    pub fn get_atom_property(&self, id: XWinId, property: Atom) -> Result<Atom> {
        let cookie = xcb::get_property(
            &self.conn,             // xcb connection to X11
            false,                  // should the property be deleted
            id.as_raw(),            // target window to query
            self.atom_id(property), // the property we want
            xcb::ATOM_ANY,          // the type of the property
            0,                      // offset in the property to retrieve data from
            1024,                   // how many 32bit multiples of data to retrieve
        );

        let reply = cookie.get_reply()?;
        if reply.value_len() == 0 {
            bail!("property {property:?} was empty for id {id:?}")
        } else {
            let id = reply.value()[0];
            self.atoms
                .iter()
                .find(|(_, v)| **v == id)
                .map(|(k, _)| *k)
                .with_context(|| format!("atom with id {id} is not supported"))
        }
    }
}
