use crate::config;
use crate::workspace::Workspace;
use crate::x11;
use ::x11::xlib;
use log::error;
use log::trace;
use log::{debug, info};
use std::{cell::RefCell, process::Command, rc::Rc};
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
    // Using a RefCell is necessary as Workspace can be mutable.
    workspaces: Vec<Rc<RefCell<Workspace>>>,
    current_workspace: Rc<RefCell<Workspace>>,
}
impl TDAWm {
    pub fn new(mut server: x11::X11Adapter) -> Result<TDAWm, TDAWmError> {
        server.init();
        server.grab_key(xlib::AnyKey, xlib::ControlMask);
        let mut workspaces = Vec::new();
        for _ in 0..10 {
            let workspace_ref = Rc::new(RefCell::new(Workspace::new()));
            workspaces.push(workspace_ref.clone());
        }
        // we can unwrap as workspaces number (10) is static so a first will exist
        let current_workspace = workspaces.first().unwrap().clone();
        let t = TDAWm {
            server,
            workspaces,
            current_workspace,
        };
        Ok(t)
    }
    pub fn run(&mut self) -> Result<(), TDAWmError> {
        loop {
            let event = self.server.next_event();
            match event.get_type() {
                // Window created
                xlib::MapRequest => {
                    self.register_window(event)?;
                }
                // Window deleted
                xlib::UnmapNotify => {
                    self.unregister_window(event)?;
                }

                xlib::KeyPress => {
                    self.handle_keypress(event)?;
                }

                // When cursor enters a window
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
        // (we have to ask x11 to send us events we want)
        // then, theses focus events (for all windows) will be treated in run
        // main loop to automatically focus whichever window your cursor is on
        self.server.grab_window_enter_event(event.window);
        self.current_workspace.borrow_mut().add_window(event.window);
        self.layout()?;
        Ok(())
    }
    fn unregister_window(&mut self, event: xlib::XEvent) -> Result<(), TDAWmError> {
        let event: xlib::XMapRequestEvent = From::from(event);
        info!("unregistering new window with id {}", event.window);
        self.current_workspace
            .borrow_mut()
            .remove_window(&event.window);
        self.layout()?;
        Ok(())
    }

    fn handle_keypress(&mut self, event: xlib::XEvent) -> Result<(), TDAWmError> {
        // converting event to good type
        let event: xlib::XKeyEvent = From::from(event);

        if event.keycode == 36 {
            //enter
            debug!("starting alacritty");
            Command::new("alacritty")
                .spawn()
                .expect("failed to execute process");
        }

        // Number keys at the top of the keyboard
        if event.keycode >= 10 && event.keycode <= 19 {
            let wc_id = event.keycode as u32 - 10;
            trace!("switching to workspace {}", wc_id);
            self.switch_workspace(wc_id as usize)?;
        }
        Ok(())
    }
    fn layout(&mut self) -> Result<(), TDAWmError> {
        trace!("computing layout..");
        let screen = self
            .server
            .screens
            .first()
            .ok_or_else(|| TDAWmError::NoScreenFound)?;

        let ws = self.current_workspace.borrow();
        let length = ws.windows.len() as u32;
        if length == 0 {
            // not any windows
            return Ok(());
        }
        // Each window will get 100%/nbr of windows width and 100% height
        let window_width = screen.width / length;
        for (i, window) in ws.windows.iter().enumerate() {
            self.server
                .resize_window(*window, window_width, screen.height);
            self.server.move_window(
                *window,
                screen.x as i32 + window_width as i32 * i as i32,
                screen.y as i32,
            );
            self.server.show_window(*window);
        }
        Ok(())
    }
    fn switch_workspace(&mut self, index: usize) -> Result<(), TDAWmError> {
        for window in self.current_workspace.borrow().windows.iter() {
            self.server.hide_window(*window);
        }
        if let Some(ws) = self.workspaces.get(index) {
            self.current_workspace = ws.clone();
        }
        self.layout()
    }
}
