use std::collections::BTreeSet;
use std::process::Command;

use crate::config;
use crate::x11;
use ::x11::xlib;
use log::{debug, info};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TDAWmError {}

struct Workspace {
    windows: BTreeSet<Window>,
    id: u32,
}

impl Workspace {
    fn new(id: u32) -> Self {
        Workspace {
            windows: BTreeSet::new(),
            id,
        }
    }

    fn add_window(&mut self, window: Window) {
        self.windows.insert(window);
    }

    fn remove_window(&mut self, window: &Window) {
        self.windows.remove(window);
    }
}

pub type Window = u64;
pub type Keycode = i32;
pub struct TDAWm {
    server: x11::X11Adapter,
    _config: config::Config,
}
impl TDAWm {
    pub fn new(server: x11::X11Adapter, config: config::Config) -> Result<TDAWm, TDAWmError> {
        server.init();
        server.grab_key(xlib::AnyKey, xlib::ControlMask);
        let t = TDAWm {
            server,
            _config: config,
        };
        Ok(t)
    }
    pub fn run(&mut self) -> Result<(), TDAWmError> {
        loop {
            let event = self.server.next_event();
            match event.get_type() {
                xlib::MapRequest => {
                    self.register_window(event)?;
                }
                xlib::KeyPress => {
                    self.handle_keypress(event)?;
                }
                _ => {
                    debug!("unknown event {:?}", event);
                    continue;
                }
            }
        }
    }

    fn register_window(&mut self, event: xlib::XEvent) -> Result<(), TDAWmError> {
        // converting to good event type to access properties
        let event: xlib::XMapRequestEvent = From::from(event);
        info!("registering new window with id {}", event.window);

        self.server.put_window_on_top(event.window);
        self.server.focus_window(event.window);

        // ask x11 to send event when a cursor enter a window.
        // then, theses events (for all windows) will be treated in run
        // main loop to automatically focus whichever window your cursor is on
        self.server.grab_window_enter_event(event.window);
        Ok(())
    }

    fn handle_keypress(&self, event: xlib::XEvent) -> Result<(), TDAWmError> {
        // converting event to good type
        let event: xlib::XKeyEvent = From::from(event);

        if event.keycode == 36 {
            //enter
            debug!("starting alacritty");
            Command::new("alacritty")
                .spawn()
                .expect("failed to execute process");
        }

        Ok(())
    }
}
