use core::slice;
use std::{
    collections::HashMap,
    ffi::{c_long, c_uchar, CStr, CString, NulError},
    mem::zeroed,
    ptr,
};

use log::{error, info, trace};
use thiserror::Error;
use x11::{
    xinerama,
    xlib::{self, Atom},
};

use crate::tdawm::Window;
use crate::tdawm::{self, WindowType};
#[derive(Debug)]
pub struct Screen {
    pub width: u32,
    pub height: u32,
    pub x: i16,
    pub y: i16,
}

pub struct X11Adapter {
    pub display: *mut xlib::Display,
    pub screens: Vec<Screen>,
    pub atom_manager: AtomManager,
}
#[derive(Debug, Error)]
pub enum X11Error {
    #[error("display {0} not found")]
    DisplayNotFound(String),
    #[error("{0}")]
    NulString(#[from] NulError),
}

impl X11Adapter {
    pub fn new(display_name: &str) -> Result<X11Adapter, X11Error> {
        let display: *mut xlib::Display =
            unsafe { xlib::XOpenDisplay(CString::new(display_name)?.as_ptr()) };
        if display.is_null() {
            return Err(X11Error::DisplayNotFound(display_name.into()));
        }
        let am = AtomManager::new();

        Ok(X11Adapter {
            display,
            screens: vec![],
            atom_manager: am,
        })
    }
    pub fn init(&mut self) {
        info!("registering to x11 as a window manager");
        unsafe {
            // https://tronche.com/gui/x/xlib/event-handling/XSelectInput.html
            xlib::XSelectInput(
                self.display,
                xlib::XDefaultRootWindow(self.display),
                xlib::SubstructureRedirectMask
                    | xlib::SubstructureNotifyMask
                    | xlib::StructureNotifyMask
                    | xlib::EnterWindowMask,
            );
        }
        self.load_screens();
    }
    pub fn next_event(&self) -> xlib::XEvent {
        let mut event: xlib::XEvent = unsafe { zeroed() };
        unsafe {
            xlib::XNextEvent(self.display, &mut event);
        }
        event
    }
    pub fn grab_key(&self, keycode: tdawm::Keycode, modifier: u32) {
        trace!("grabbing key {} with modifier {}", keycode, modifier);
        unsafe {
            xlib::XGrabKey(
                self.display,
                keycode,
                modifier,
                xlib::XDefaultRootWindow(self.display),
                0,
                xlib::GrabModeAsync,
                xlib::GrabModeAsync,
            );
        }
    }

    pub fn ungrab_key(&self, keycode: tdawm::Keycode, modifier: u32) {
        trace!("ungrabbing key {} with modifier {}", keycode, modifier);
        unsafe {
            xlib::XUngrabKey(
                self.display,
                keycode,
                modifier,
                xlib::XDefaultRootWindow(self.display),
            );
        }
    }
    pub fn focus_window(&self, window: &Window) {
        trace!("focusing window {}", window.id);
        unsafe {
            xlib::XSetInputFocus(
                self.display,
                window.id,
                xlib::RevertToNone,
                xlib::CurrentTime,
            );
        }
    }
    pub fn put_window_on_top(&self, window: &Window) {
        trace!("putting window {} on top", window.id);
        unsafe {
            xlib::XMapRaised(self.display, window.id);
        }
    }
    pub fn grab_window_events(&self, window: &Window) {
        trace!("grabbing window {} events", window.id);
        unsafe {
            xlib::XSelectInput(
                self.display,
                window.id,
                xlib::EnterWindowMask | xlib::PropertyChangeMask,
            );
        }
    }
    pub fn move_window(&self, window: &Window, x: i32, y: i32) {
        trace!("moving window {} to ({}, {})", window.id, x, y);
        unsafe { xlib::XMoveWindow(self.display, window.id, x, y) };
    }

    pub fn resize_window(&self, window: &Window, width: u32, height: u32) {
        trace!("resizing window {} to {}x{}", window.id, width, height);
        unsafe { xlib::XResizeWindow(self.display, window.id, width, height) };
    }
    pub fn hide_window(&self, window: &Window) {
        unsafe { xlib::XUnmapWindow(self.display, window.id) };
    }
    pub fn load_screens(&mut self) {
        info!("loading screens");
        let mut num: i32 = 0;
        unsafe {
            let screen_pointers = xinerama::XineramaQueryScreens(self.display, &mut num);
            let screens = slice::from_raw_parts(screen_pointers, num as usize).to_vec();
            for screen in screens.iter() {
                self.screens.push(Screen {
                    width: screen.width as u32,
                    height: screen.height as u32,
                    x: screen.x_org,
                    y: screen.y_org,
                });
                trace!("found screen: {:?}", self.screens.last().unwrap());
            }
        }
    }

    pub fn show_window(&self, window: &Window) {
        unsafe { xlib::XMapWindow(self.display, window.id) };
    }
    pub fn ewmh_set_current_desktop(&mut self, index: usize) {
        let data: u32 = index as u32;
        let format = 32;
        unsafe {
            let prop = self
                .atom_manager
                .get_atom("_NET_CURRENT_DESKTOP", self.display);
            xlib::XChangeProperty(
                self.display,
                xlib::XDefaultRootWindow(self.display),
                prop,
                xlib::XA_CARDINAL,
                format,
                xlib::PropModeReplace,
                &data as *const u32 as *const u8,
                1,
            );
        }
    }
}

pub trait EWMH {
    fn get_window_type(&self, server: &mut X11Adapter) -> WindowType;
}

impl EWMH for Window {
    fn get_window_type(&self, server: &mut X11Adapter) -> WindowType {
        let mut actual_type_return: Atom = 0;
        let mut actual_format_return: i32 = 0;
        let mut nitems_return: u64 = 0;
        let mut bytes_after_return: u64 = 0;
        let mut prop_return: *mut c_uchar = ptr::null_mut();
        let window_type: i64;
        unsafe {
            let net_wm_window_type_atom = server
                .atom_manager
                .get_atom("_NET_WM_WINDOW_TYPE", server.display);

            // window_type = 1;
            if xlib::XGetWindowProperty(
                server.display,
                self.id,
                net_wm_window_type_atom,
                0,
                1,
                0,
                xlib::AnyPropertyType as u64,
                &mut actual_type_return,
                &mut actual_format_return,
                &mut nitems_return,
                &mut bytes_after_return,
                &mut prop_return,
            ) != xlib::Success as i32
            {
                return WindowType::Normal;
            }
            window_type = *(prop_return as *const c_long);
            let net_wm_window_dock_atom = server
                .atom_manager
                .get_atom("_NET_WM_WINDOW_TYPE_DOCK", server.display);

            if window_type as u64 == net_wm_window_dock_atom {
                return WindowType::Dock;
            }
            return WindowType::Normal;
        }
    }
}

pub struct AtomManager {
    atoms: HashMap<&'static str, u64>,
}

impl AtomManager {
    fn new() -> AtomManager {
        AtomManager {
            atoms: HashMap::new(),
        }
    }
    pub fn get_atom(&mut self, name: &'static str, display: *mut xlib::_XDisplay) -> u64 {
        if self.atoms.contains_key(name) {
            return *self.atoms.get(name).unwrap();
        }
        unsafe {
            let atom = xlib::XInternAtom(display, format!("{}\0", name).as_ptr() as *const i8, 0);
            self.atoms.insert(name, atom);
            return atom;
        }
    }
    pub fn identify(&self, atom: u64, display: *mut xlib::_XDisplay) -> &'static str {
        unsafe {
            let val = xlib::XGetAtomName(display, atom);
            let c_str = CStr::from_ptr(val);
            return c_str.to_str().unwrap();
        }
    }
}
