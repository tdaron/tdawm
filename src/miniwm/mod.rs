use core::slice;
use std::{
    collections::BTreeSet,
    ffi::{CString, NulError},
    mem::zeroed,
};

use thiserror::Error;
use x11::{
    xinerama,
    xlib::{self, CurrentTime, RevertToNone},
};

pub type Window = u64;

#[derive(Error, Debug)]
pub enum MiniWMError {
    #[error("display {0} not found")]
    DisplayNotFound(String),
    #[error("{0}")]
    NulString(#[from] NulError),
    #[error("screen not found")]
    ScreenNotFound,
}

pub struct MiniWM {
    display: *mut xlib::Display,
    windows: BTreeSet<Window>,
}

impl MiniWM {
    pub fn new(display_name: &str) -> Result<Self, MiniWMError> {
        let display: *mut xlib::Display =
            unsafe { xlib::XOpenDisplay(CString::new(display_name)?.as_ptr()) };
        if display.is_null() {
            return Err(MiniWMError::DisplayNotFound(display_name.into()));
        }
        let windows = BTreeSet::new();
        Ok(MiniWM { display, windows })
    }
    pub fn init(&self) -> Result<(), MiniWMError> {
        unsafe {
            xlib::XSelectInput(
                self.display,
                xlib::XDefaultRootWindow(self.display),
                xlib::SubstructureRedirectMask | xlib::SubstructureNotifyMask,
            );
        }
        Ok(())
    }
    pub fn run(&mut self) -> Result<(), MiniWMError> {
        let mut event: xlib::XEvent = unsafe { zeroed() };
        loop {
            unsafe {
                xlib::XNextEvent(self.display, &mut event);
                match event.get_type() {
                    xlib::MapRequest => {
                        self.create_window(event)?;
                    }
                    xlib::UnmapNotify => {
                        self.remove_window(event)?;
                    }
                    _ => {
                        println!("unknown event {:?}", event);
                    }
                }
            }
        }
    }

    fn create_window(&mut self, event: xlib::XEvent) -> Result<(), MiniWMError> {
        println!("creating a window");
        let event: xlib::XMapRequestEvent = From::from(event);
        unsafe { xlib::XMapWindow(self.display, event.window) };
        unsafe {
            xlib::XSetInputFocus(self.display, event.window, RevertToNone, CurrentTime);
        }
        self.windows.insert(event.window);
        self.horizontal_layout()
    }
    fn remove_window(&mut self, event: xlib::XEvent) -> Result<(), MiniWMError> {
        println!("removing a window");
        let event: xlib::XUnmapEvent = From::from(event);
        self.windows.remove(&event.window);
        self.horizontal_layout()
    }

    fn get_screen_size(&self) -> Result<(i16, i16), MiniWMError> {
        unsafe {
            let mut num: i32 = 0;
            let screen_pointers = xinerama::XineramaQueryScreens(self.display, &mut num);
            let screens = slice::from_raw_parts(screen_pointers, num as usize).to_vec();
            let screen = screens.get(0); // get the first screen. No multi display yet.
            if let Some(screen) = screen {
                Ok((screen.width, screen.height))
            } else {
                Err(MiniWMError::ScreenNotFound)
            }
        }
    }

    fn vertical_layout(&self) -> Result<(), MiniWMError> {
        if self.windows.is_empty() {
            return Ok(());
        }
        let (width, height) = self.get_screen_size()?;
        let mut start = 0;
        let win_width = width as i32 / self.windows.len() as i32;
        self.windows.iter().for_each(|window| {
            self.move_window(*window, start, 0_i32);
            self.resize_window(*window, win_width as u32, height as u32);
            start += win_width;
        });
        Ok(())
    }

    fn horizontal_layout(&self) -> Result<(), MiniWMError> {
        if self.windows.is_empty() {
            return Ok(());
        }
        let (width, height) = self.get_screen_size()?;
        let mut start = 0;
        let h_gasp: i32 = 15;
        let win_height =
            (height as i32 - h_gasp * self.windows.len() as i32) / self.windows.len() as i32;
        self.windows.iter().for_each(|window| {
            self.move_window(*window, 0_i32, start);
            self.resize_window(*window, width as u32, win_height as u32);
            start += win_height + h_gasp;
        });
        Ok(())
    }

    fn move_window(&self, window: u64, x: i32, y: i32) {
        unsafe { xlib::XMoveWindow(self.display, window, x, y) };
    }

    fn resize_window(&self, window: u64, width: u32, height: u32) {
        unsafe { xlib::XResizeWindow(self.display, window, width, height) };
    }
}
