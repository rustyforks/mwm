use anyhow::{Context, Result};
use rustc_hash::FxHashMap as HashMap;

use crate::atom::Atom;

/// data for abstracting communication with the X server via xcb
pub struct XConn {
    pub(crate) conn: xcb::Connection,

    pub(crate) root: xcb::x::Window,
    pub(crate) check_win: xcb::x::Window,

    // interned atoms
    pub(crate) atoms: HashMap<Atom, xcb::x::Atom>,
}

impl Drop for XConn {
    fn drop(&mut self) {
        // release any of the keybindings we are holding on to
        self.conn.send_request(&xcb::x::UngrabKey {
            key: xcb::x::Grab::Any as u8,
            grab_window: self.root,
            modifiers: xcb::x::ModMask::ANY,
        });

        // destroy the check window
        self.conn
            .send_request(&xcb::x::DestroyWindow { window: self.check_win });

        // mark ourselves as no longer being the active root window
        self.conn.send_request(&xcb::x::DeleteProperty {
            window: self.root,
            property: self.atom_id(Atom::NetActiveWindow),
        });

        self.conn.flush().unwrap();
    }
}

impl XConn {
    /// Establish a new connection to the running X server. Fails if unable to
    /// connect
    pub(super) fn init() -> Result<XConn> {
        let (conn, _) =
            xcb::Connection::connect_with_extensions(None, &[xcb::Extension::RandR], &[])
                .context("connecting to X server")?;

        let root = conn
            .get_setup()
            .roots()
            .next()
            .context("getting handle for screen")?
            .root();

        // NOTE the collect is actually very required here as the `xcb::intern_atom` has
        // a side-effect and we need the run the iterator through to send all the
        // requests now so cookies can be forced later with hopefully less
        // latency.
        #[allow(clippy::needless_collect)]
        let atom_cookies = Atom::ALL
            .iter()
            .map(|atom| {
                conn.send_request(&xcb::x::InternAtom {
                    only_if_exists: false,
                    name: atom.as_str().as_bytes(),
                })
            })
            .collect::<Vec<_>>();

        let check_win = conn.generate_id();

        let create_window_cookie = conn.send_request_checked(&xcb::x::CreateWindow {
            depth: 0,
            wid: check_win,
            parent: root,
            x: 0,
            y: 0,
            width: 1,
            height: 1,
            border_width: 0,
            class: xcb::x::WindowClass::InputOnly,
            visual: xcb::x::COPY_FROM_PARENT,
            value_list: &[],
        });

        let select_input_cookie = conn.send_request_checked(&xcb::randr::SelectInput {
            window: root,
            enable: xcb::randr::NotifyMask::CRTC_CHANGE | xcb::randr::NotifyMask::SCREEN_CHANGE,
        });

        let substructure_redirect_cookie =
            conn.send_request_checked(&xcb::x::ChangeWindowAttributes {
                window: root,
                value_list: &[xcb::x::Cw::EventMask(
                    xcb::x::EventMask::PROPERTY_CHANGE
                        | xcb::x::EventMask::SUBSTRUCTURE_REDIRECT
                        | xcb::x::EventMask::SUBSTRUCTURE_NOTIFY,
                )],
            });

        let atoms = {
            let replies = atom_cookies.into_iter().map(|cookie| {
                conn.wait_for_reply(cookie)
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
        conn.check_request(create_window_cookie)
            .context("create check window")?;
        conn.check_request(select_input_cookie)
            .context("select input")?;
        conn.check_request(substructure_redirect_cookie)
            .context("substructure redirect")?;

        Ok(XConn { conn, root, check_win, atoms })
    }

    fn atom_id(&self, atom: Atom) -> xcb::x::Atom {
        *self.atoms.get(&atom).unwrap()
    }
}
