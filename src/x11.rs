use core::slice;
use std::{
    ffi::{CString, NulError},
    mem::zeroed,
};

use log::{info, trace};
use thiserror::Error;
use x11::{xinerama, xlib};

use crate::tdawm::{self, Window};

#[derive(Debug)]
pub struct Screen {
    pub width: u32,
    pub height: u32,
    pub x: i16,
    pub y: i16,
}

pub struct X11Adapter {
    display: *mut xlib::Display,
    pub screens: Vec<Screen>,
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
        Ok(X11Adapter {
            display,
            screens: vec![],
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
    pub fn focus_window(&self, window: Window) {
        trace!("focusing window {}", window);
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
        trace!("grabbing window {} events", window);
        unsafe {
            xlib::XSelectInput(self.display, window, xlib::EnterWindowMask);
        }
    }
    pub fn move_window(&self, window: Window, x: i32, y: i32) {
        trace!("moving window {} to ({}, {})", window, x, y);
        unsafe { xlib::XMoveWindow(self.display, window, x, y) };
    }

    pub fn resize_window(&self, window: Window, width: u32, height: u32) {
        trace!("resizing window {} to {}x{}", window, width, height);
        unsafe { xlib::XResizeWindow(self.display, window, width, height) };
    }
    pub fn hide_window(&self, window: Window) {
        unsafe { xlib::XUnmapWindow(self.display, window) };
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

    pub fn show_window(&self, window: Window) {
        unsafe { xlib::XMapWindow(self.display, window) };
    }
    pub fn ewmh_set_current_desktop(&self, index: usize) {
        let data: u32 = index as u32;
        let format = 32;
        unsafe {
            let prop = xlib::XInternAtom(
                self.display,
                "_NET_CURRENT_DESKTOP\0".as_ptr() as *const i8,
                0,
            );
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
