use std::{
    ffi::{CString, NulError},
    mem::zeroed,
};

use log::trace;
use thiserror::Error;
use x11::xlib;

use crate::tdawm::{self, Window};
pub struct X11Adapter {
    display: *mut xlib::Display,
}
#[derive(Debug, Error)]
pub enum X11Error {
    #[error("display {0} not found")]
    DisplayNotFound(String),
    #[error("{0}")]
    NulString(#[from] NulError),
}

impl X11Adapter {
    pub fn new(display_name: &str) -> Result<Self, X11Error> {
        let display: *mut xlib::Display =
            unsafe { xlib::XOpenDisplay(CString::new(display_name)?.as_ptr()) };
        if display.is_null() {
            return Err(X11Error::DisplayNotFound(display_name.into()));
        }
        Ok(X11Adapter { display })
    }
    pub fn init(&self) {
        trace!("registering to x11 as a window manager");
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
    }
    pub fn next_event(&self) -> xlib::XEvent {
        let mut event: xlib::XEvent = unsafe { zeroed() };
        unsafe {
            xlib::XNextEvent(self.display, &mut event);
        }
        event
    }
    pub fn grab_key(&self, keycode: tdawm::Keycode, modifier: u32) {
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
        unsafe {
            xlib::XUngrabKey(
                self.display,
                keycode,
                modifier,
                xlib::XDefaultRootWindow(self.display),
            );
        }
    }
    pub fn focus_window(&self, window: Window) {
        unsafe {
            xlib::XSetInputFocus(self.display, window, xlib::RevertToNone, xlib::CurrentTime);
        }
    }
    pub fn put_window_on_top(&self, window: Window) {
        trace!("putting window {} on top", window);
        unsafe {
            xlib::XMapRaised(self.display, window);
        }
    }
    pub fn grab_window_enter_event(&self, window: Window) {
        unsafe {
            xlib::XSelectInput(self.display, window, xlib::EnterWindowMask);
        }
    }
}
