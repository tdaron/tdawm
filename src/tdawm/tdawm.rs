// Informations for a better code follow:
// EWMH are some hints for status bar for example.
// https://en.wikipedia.org/wiki/Extended_Window_Manager_Hints

use super::Position;
use super::Size;
use super::Window;
use super::WindowId;
use super::Workspace;
use crate::layouts::HorizontalLayout;
use crate::layouts::Layout;
use crate::layouts::VerticalLayout;
use crate::tdawm::WindowType;
use crate::x11;
use crate::x11::EWMH;
use ::x11::xlib;
use log::error;
use log::trace;
use log::{debug, info};
use std::collections::HashMap;
use std::{cell::RefCell, process::Command, rc::Rc};
use thiserror::Error;
#[derive(Error, Debug)]
pub enum TDAWmError {
    #[error("No screen")]
    NoScreenFound,
}

pub type Keycode = i32;
pub struct TDAWm {
    pub server: x11::X11Adapter,
    // Using a RefCell is necessary as Workspace can be mutable.
    pub workspaces: Vec<Rc<RefCell<Workspace>>>,
    pub current_workspace: Rc<RefCell<Workspace>>,
    pub windows_by_id: HashMap<WindowId, Window>,
    current_layout: Box<dyn Layout>,
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
            windows_by_id: HashMap::new(),
            current_layout: Box::new(crate::layouts::HorizontalLayout::init()),
        };
        Ok(t)
    }
    pub fn run(&mut self) -> Result<(), TDAWmError> {
        self.server.ewmh_set_current_desktop(0);
        loop {
            let event = self.server.next_event();
            match event.get_type() {
                xlib::CreateNotify => {
                    let event: xlib::XCreateWindowEvent = From::from(event);
                    self.server.grab_window_events(event.window as WindowId);
                }
                // Window showed
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
                xlib::ClientMessage => {
                    let event: xlib::XClientMessageEvent = From::from(event);
                    debug!(
                        "Got client message {} from window {}",
                        self.server
                            .atom_manager
                            .identify(event.message_type, self.server.display),
                        event.window
                    );
                }
                xlib::PropertyNotify => {
                    let event: xlib::XPropertyEvent = From::from(event);
                    self.load_window_properties(event.window);
                }
                xlib::ConfigureRequest => {
                    debug!("received configure request event {:?}", event);
                    let event: xlib::XConfigureRequestEvent = From::from(event);
                    if let Some(window) = self.windows_by_id.get_mut(&event.window as &WindowId) {
                        window.fixed_position = Some(Position {
                            x: event.x,
                            y: event.y,
                        });
                        window.fixed_size = Some(Size {
                            x: event.width as u32,
                            y: event.height as u32,
                        });
                        self.layout()?;
                    }
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

        self.server.put_window_on_top(event.window as WindowId);
        self.server.focus_window(event.window as WindowId);

        self.current_workspace
            .borrow_mut()
            .add_window(event.window as WindowId);

        self.windows_by_id
            .insert(event.window as WindowId, Into::<Window>::into(event.window));
        // ask x11 to send event when a cursor enter a window.
        // (we have to ask x11 to send us events we want)
        // then, theses focus events (for all windows) will be treated in run
        // main loop to automatically focus whichever window your cursor is on
        self.server.grab_window_events(event.window as WindowId);

        self.load_window_properties(event.window);
        self.layout()?;
        Ok(())
    }
    fn unregister_window(&mut self, event: xlib::XEvent) -> Result<(), TDAWmError> {
        let event: xlib::XMapRequestEvent = From::from(event);
        info!("unregistering window with id {}", event.window);
        self.current_workspace
            .borrow_mut()
            .remove_window(&event.window.into());
        self.windows_by_id.remove(&event.window as &WindowId);
        self.layout()?;
        Ok(())
    }

    fn handle_keypress(&mut self, event: xlib::XEvent) -> Result<(), TDAWmError> {
        // converting event to good type
        let event: xlib::XKeyEvent = From::from(event);
        trace!("keypress: {}", event.keycode);
        // layout switch on ctrl+p
        // for debug purposes right now.
        if event.keycode == 33 {
            if self.current_layout.id() == "horizontal" {
                self.current_layout = Box::new(VerticalLayout::init());
            } else {
                self.current_layout = Box::new(HorizontalLayout::init());
            }
            self.layout()?;
        }
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

    fn load_window_properties(&mut self, window_id: WindowId) {
        if let Some(window) = self.windows_by_id.get_mut(&window_id) {
            let window_type = window.get_window_type(&mut self.server);
            window.window_type = window_type;
        }
    }

    fn layout(&mut self) -> Result<(), TDAWmError> {
        self.current_layout.layout(
            &mut self.server,
            &mut self.current_workspace,
            &mut self.workspaces,
            &mut self.windows_by_id,
        )?;
        // EWMH compliance. Windows can ask to be always on top
        // for example.
        // https://specifications.freedesktop.org/wm-spec/1.3/ar01s05.html
        for window_id in self.current_workspace.borrow().windows.iter() {
            let window = self.windows_by_id.get(window_id).unwrap();
            match window.window_type {
                WindowType::Dock => {
                    // A dock window can be placed without respecting tiling.
                    if let Some(p) = window.fixed_position {
                        self.server.move_window(*window_id, p.x, p.y);
                    }
                    if let Some(s) = window.fixed_size {
                        self.server.resize_window(*window_id, s.x, s.y);
                    }

                    // Dock windows should always be on top
                    self.server.put_window_on_top(*window_id);
                }
                _ => {}
            }
        }
        Ok(())
    }
    fn switch_workspace(&mut self, index: usize) -> Result<(), TDAWmError> {
        for window_id in self.current_workspace.borrow().windows.iter() {
            self.server.hide_window(*window_id);
        }
        if let Some(ws) = self.workspaces.get(index) {
            self.current_workspace = ws.clone();
        }
        self.server.ewmh_set_current_desktop(index);
        self.layout()
    }
}
