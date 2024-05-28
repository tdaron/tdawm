use core::slice;
use std::{
    collections::BTreeSet,
    ffi::{CString, NulError},
    mem::zeroed,
    process::Command,
};

use thiserror::Error;
use x11::{
    xinerama,
    xlib::{self, AnyModifier, CurrentTime, RevertToNone, XKeyPressedEvent},
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
                xlib::SubstructureRedirectMask
                    | xlib::SubstructureNotifyMask
                    | xlib::StructureNotifyMask
                    | xlib::EnterWindowMask,
            );
        }
        Command::new("feh")
            .arg("--bg-scale")
            .arg("/usr/share/backgrounds/gruvbox/astronaut.jpg")
            .spawn() // Spawns the command as a new process.
            .expect("failed to execute process");
        self.grab_keys();
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
                    xlib::KeyPress => {
                        self.handle_keypress(event);
                    }
                    xlib::EnterNotify => {
                        println!("h {:?}", event);
                        self.focus_from_cursor(event);
                    }
                    xlib::ConfigureNotify => self.grab_keys(), //called when root window is changed by feh for example
                    _ => {
                        // println!("unknown event {:?}", event);
                        continue;
                    }
                }
            }
        }
    }

    fn create_window(&mut self, event: xlib::XEvent) -> Result<(), MiniWMError> {
        let event: xlib::XMapRequestEvent = From::from(event);
        println!("creating a window id  {}", event.window);
        unsafe { xlib::XMapRaised(self.display, event.window) };
        unsafe {
            xlib::XSetInputFocus(self.display, event.window, RevertToNone, CurrentTime);
        }
        unsafe {
            xlib::XSelectInput(self.display, event.window, xlib::EnterWindowMask);
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

    fn handle_keypress(&self, event: xlib::XEvent) {
        let event: xlib::XKeyEvent = From::from(event);
        if event.keycode == 36 {
            // ctrl+enter
            println!("starting alacritty");
            Command::new("alacritty")
                .spawn() // Spawns the command as a new process.
                .expect("failed to execute process");
        }
        println!("got control+{}", event.keycode);
    }

    fn grab_keys(&self) {
        println!("grabbing keys");
        unsafe {
            xlib::XGrabKey(
                self.display,
                xlib::AnyKey,
                xlib::ControlMask,
                xlib::XDefaultRootWindow(self.display),
                0,
                xlib::GrabModeAsync,
                xlib::GrabModeAsync,
            );
        }
    }

    fn focus_from_cursor(&self, event: xlib::XEvent) {
        let event: xlib::XEnterWindowEvent = From::from(event);
        println!("focusing window {}", event.window);
        unsafe {
            xlib::XSetInputFocus(self.display, event.window, RevertToNone, CurrentTime);
        }
    }
}
