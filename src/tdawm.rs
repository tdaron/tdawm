use std::process::Command;

use crate::config;
use crate::x11;
use ::x11::xlib;
use log::error;
use log::trace;
use log::{debug, info};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TDAWmError {
    #[error("No screen")]
    NoScreenFound,
}

pub type Window = u64;
pub type Keycode = i32;
pub struct TDAWm {
    server: x11::X11Adapter,
    windows: Vec<Window>,
    _config: config::Config,
}
impl TDAWm {
    pub fn new(mut server: x11::X11Adapter, config: config::Config) -> Result<TDAWm, TDAWmError> {
        server.init();
        server.grab_key(xlib::AnyKey, xlib::ControlMask);
        let t = TDAWm {
            server,
            _config: config,
            windows: vec![],
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
                xlib::UnmapNotify => {
                    self.unregister_window(event)?;
                }
                xlib::KeyPress => {
                    self.handle_keypress(event)?;
                }
                xlib::EnterNotify => {
                    let event: xlib::XEnterWindowEvent = From::from(event);
                    self.server.focus_window(event.window)
                }
                _ => {
                    // debug!("unknown event {:?}", event);
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
        self.windows.push(event.window);
        self.layout()?;
        Ok(())
    }
    fn unregister_window(&mut self, event: xlib::XEvent) -> Result<(), TDAWmError> {
        let event: xlib::XMapRequestEvent = From::from(event);
        info!("unregistering new window with id {}", event.window);
        if let Some(index) = self.windows.iter().position(|w| *w == event.window) {
            self.windows.remove(index);
        } else {
            // we could return an error instead of just logging it
            // but as we don't know anything about this window
            // this error would be useless to handle.
            error!(
                "Tried to unregister unexisting window with id {}",
                event.window
            );
        }
        self.layout()?;
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
    fn layout(&mut self) -> Result<(), TDAWmError> {
        let screen = self
            .server
            .screens
            .first()
            .ok_or_else(|| TDAWmError::NoScreenFound)?;

        let length = self.windows.len() as u32;
        if length == 0 {
            // not any windows
            return Ok(());
        }
        let window_width = screen.width / length;
        //ATM this will simply put each window fullscreen
        for (i, window) in self.windows.iter().enumerate() {
            self.server
                .resize_window(*window, window_width, screen.height);
            self.server.move_window(
                *window,
                screen.x as i32 + window_width as i32 * i as i32,
                screen.y as i32,
            );
        }
        trace!("computing layout..");
        Ok(())
    }
}
