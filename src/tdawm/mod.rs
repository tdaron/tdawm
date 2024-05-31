use core::slice;
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    ffi::{CString, NulError},
    mem::zeroed,
    process::Command,
    ptr::null_mut,
    rc::Rc,
};

use crate::config;
use log::{debug, info, trace};
use thiserror::Error;
use x11::{
    xinerama,
    xlib::{self, CurrentTime, RevertToNone, XDrawString, XFontStruct},
};
pub type Window = u64;

#[derive(Error, Debug)]
pub enum TDAWmError {
    #[error("display {0} not found")]
    DisplayNotFound(String),
    #[error("{0}")]
    NulString(#[from] NulError),
    #[error("screen not found")]
    ScreenNotFound,
}

unsafe extern "C" fn error_handler(_: *mut xlib::Display, event: *mut xlib::XErrorEvent) -> i32 {
    // Set the error flag if BadWindow error occurs
    if (*event).error_code == xlib::BadWindow {
        0
    } else {
        panic!("{:?}", event);
    }
}

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
pub struct TDAWm {
    display: *mut xlib::Display,
    windows: BTreeSet<Window>,
    workspaces: BTreeMap<u32, Rc<RefCell<Workspace>>>,
    current_workspace: Rc<RefCell<Workspace>>,
    status_bar: Window,
    _config: config::Config,
}

impl TDAWm {
    pub fn new(display_name: &str, config: config::Config) -> Result<Self, TDAWmError> {
        let display: *mut xlib::Display =
            unsafe { xlib::XOpenDisplay(CString::new(display_name)?.as_ptr()) };
        if display.is_null() {
            return Err(TDAWmError::DisplayNotFound(display_name.into()));
        }
        let windows = BTreeSet::new();

        let mut workspaces = BTreeMap::new();
        let workspace = Rc::new(RefCell::new(Workspace::new(0)));
        let current_workspace = Rc::clone(&workspace);
        workspaces.insert(0, workspace);
        Ok(TDAWm {
            display,
            windows,
            workspaces,
            current_workspace,
            status_bar: 0,
            _config: config,
        })
    }
    pub fn init(&mut self) -> Result<(), TDAWmError> {
        info!("initializing tdawm");
        let (width, _) = self.get_screen_size()?;
        unsafe {
            trace!("getting inputs from x11");
            xlib::XSelectInput(
                self.display,
                xlib::XDefaultRootWindow(self.display),
                xlib::SubstructureRedirectMask
                    | xlib::SubstructureNotifyMask
                    | xlib::StructureNotifyMask
                    | xlib::EnterWindowMask,
            );

            trace!("setting cursor");

            // https://tronche.com/gui/x/xlib/appendix/b/
            const XC_LEFT_PTR: u32 = 68; // Value for left_ptr cursor
            let cursor = xlib::XCreateFontCursor(self.display, XC_LEFT_PTR);
            xlib::XDefineCursor(self.display, xlib::XDefaultRootWindow(self.display), cursor);

            trace!("creating status bar");
            let window = xlib::XCreateSimpleWindow(
                self.display,
                xlib::XDefaultRootWindow(self.display),
                0,
                0,
                width as u32,
                20, //height
                0,
                xlib::XBlackPixel(self.display, 0),
                xlib::XWhitePixel(self.display, 0),
            );
            xlib::XMapWindow(self.display, window);
            self.status_bar = window;
            println!("status {}", window);
        }
        trace!("grabbing keys");
        self.grab_keys();
        self.layout()?;
        trace!("setting error handler");
        unsafe {
            xlib::XSetErrorHandler(Some(error_handler));
        }
        Ok(())
    }
    pub fn run(&mut self) -> Result<(), TDAWmError> {
        let mut event: xlib::XEvent = unsafe { zeroed() };
        info!("waiting for events");
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
                        self.handle_keypress(event)?;
                    }
                    xlib::EnterNotify => {
                        self.focus_from_cursor(event);
                    }
                    xlib::ConfigureNotify => self.grab_keys(), //called when root window is changed by feh for example
                    _ => {
                        debug!("unknown event {:?}", event);
                        continue;
                    }
                }
            }
        }
    }

    fn create_window(&mut self, event: xlib::XEvent) -> Result<(), TDAWmError> {
        let event: xlib::XMapRequestEvent = From::from(event);
        info!("creating a window with id {}", event.window);
        unsafe { xlib::XMapRaised(self.display, event.window) };
        unsafe {
            //focus newly created window
            xlib::XSetInputFocus(self.display, event.window, RevertToNone, CurrentTime);
        }
        unsafe {
            //get event of pointer going inside the window
            //to focus it
            xlib::XSelectInput(self.display, event.window, xlib::EnterWindowMask);
        }
        self.current_workspace
            .borrow_mut()
            .add_window(event.window as Window);
        self.windows.insert(event.window as Window);
        self.layout()
    }
    fn remove_window(&mut self, event: xlib::XEvent) -> Result<(), TDAWmError> {
        let event: xlib::XUnmapEvent = From::from(event);
        info!("removing window with id {}", event.window);
        self.current_workspace
            .borrow_mut()
            .remove_window(&event.window);
        self.windows.remove(&event.window);
        self.layout()
    }

    fn get_screen_size(&self) -> Result<(i16, i16), TDAWmError> {
        unsafe {
            let mut num: i32 = 0;
            let screen_pointers = xinerama::XineramaQueryScreens(self.display, &mut num);
            let screens = slice::from_raw_parts(screen_pointers, num as usize).to_vec();
            let screen = screens.get(0); // get the first screen. No multi display yet.
            if let Some(screen) = screen {
                Ok((screen.width, screen.height))
            } else {
                Err(TDAWmError::ScreenNotFound)
            }
        }
    }

    fn change_workspace(&mut self, wc_id: u32) -> Result<(), TDAWmError> {
        self.current_workspace
            .borrow()
            .windows
            .iter()
            .for_each(|window| {
                self.hide_window(*window);
            });
        let opt_ws = self.workspaces.get(&wc_id);
        if let Some(ws) = opt_ws {
            self.current_workspace = Rc::clone(&ws);
        } else {
            let workspace = Rc::new(RefCell::new(Workspace::new(wc_id)));
            self.workspaces.insert(wc_id, Rc::clone(&workspace));
            self.current_workspace = Rc::clone(&workspace);
        }
        info!("Switched to workspace {}", wc_id);
        self.layout()
    }

    fn get_gc_window(&self, window: Window) -> *mut xlib::_XGC {
        unsafe {
            let gc = xlib::XCreateGC(self.display, window, 0, std::ptr::null_mut());
            return gc;
        }
    }
    fn draw_bar(&self) {
        let (width, _) = self.get_screen_size().unwrap();
        let gc = self.get_gc_window(self.status_bar);
        let text = format!(
            "Current workspace: {}",
            self.current_workspace.borrow().id + 1
        );
        let c_text = &CString::new(text.clone()).unwrap();
        unsafe {
            //TODO: replace this with not ugly x11 fonts
            let font =
                CString::new("-misc-fixed-medium-r-semicondensed--13-120-75-75-c-60-iso8859-1")
                    .unwrap();
            let font_info: *mut XFontStruct = xlib::XLoadQueryFont(self.display, font.as_ptr());
            if font_info == std::ptr::null_mut() {
                println!("ERROR: font not found");
                return;
            }

            xlib::XSetFont(self.display, gc, (*font_info).fid);
            xlib::XSetForeground(self.display, gc, xlib::XWhitePixel(self.display, 0));
            xlib::XFillRectangle(self.display, self.status_bar, gc, 0, 0, width as u32, 20);
            xlib::XSetForeground(self.display, gc, xlib::XBlackPixel(self.display, 0));
            XDrawString(
                self.display,
                self.status_bar,
                gc,
                10,
                16,
                c_text.as_ptr(),
                text.len() as i32,
            );
        }
    }
    fn layout(&self) -> Result<(), TDAWmError> {
        let ws = self.current_workspace.borrow();
        if ws.windows.is_empty() {
            self.draw_bar();
            return Ok(());
        }
        let bar_size: i32 = 20;
        let (width, height) = self.get_screen_size()?;
        let mut start = bar_size;
        let height = height as i32 - bar_size;
        let h_gasp: i32 = 0;
        let win_height =
            (height as i32 - h_gasp * ws.windows.len() as i32) / ws.windows.len() as i32;
        ws.windows.iter().for_each(|window| {
            self.move_window(*window, 0_i32, start);
            self.resize_window(*window, width as u32, win_height as u32);
            self.show_window(*window);
            start += win_height + h_gasp;
        });
        self.draw_bar();
        Ok(())
    }

    fn move_window(&self, window: u64, x: i32, y: i32) {
        unsafe { xlib::XMoveWindow(self.display, window, x, y) };
    }

    fn resize_window(&self, window: u64, width: u32, height: u32) {
        unsafe { xlib::XResizeWindow(self.display, window, width, height) };
    }

    fn handle_keypress(&mut self, event: xlib::XEvent) -> Result<(), TDAWmError> {
        let event: xlib::XKeyEvent = From::from(event);
        if event.keycode == 36 {
            // ctrl+enter
            debug!("starting alacritty");
            Command::new("alacritty")
                .spawn() // Spawns the command as a new process.
                .expect("failed to execute process");
        }
        if event.keycode >= 10 && event.keycode <= 19 {
            let wc_id = event.keycode as u32 - 10;
            self.change_workspace(wc_id)?;
        }
        if event.keycode == 33 {
            //ctrl+p
            Command::new("rofi")
                .arg("-show")
                .arg("drun")
                .arg("-show-icons")
                .spawn() // Spawns the command as a new process.
                .expect("failed to execute process");
        }
        debug!("got control+{}", event.keycode);
        Ok(())
    }

    fn grab_keys(&self) {
        unsafe {
            xlib::XGrabKey(
                self.display,
                xlib::XKeysymToKeycode(self.display, x11::keysym::XK_Return as u64) as i32,
                xlib::Mod4Mask,
                xlib::XDefaultRootWindow(self.display),
                0,
                xlib::GrabModeAsync,
                xlib::GrabModeAsync,
            );
            xlib::XGrabKey(
                self.display,
                xlib::XKeysymToKeycode(self.display, x11::keysym::XK_P as u64) as i32,
                xlib::Mod4Mask,
                xlib::XDefaultRootWindow(self.display),
                0,
                xlib::GrabModeAsync,
                xlib::GrabModeAsync,
            );
        }
    }

    fn focus_from_cursor(&self, event: xlib::XEvent) {
        let mut event: xlib::XEnterWindowEvent = From::from(event);
        unsafe {
            let mut window_attributes: xlib::XWindowAttributes = std::mem::zeroed();
            let r = xlib::XGetWindowAttributes(self.display, event.window, &mut window_attributes);
            if r == xlib::BadWindow.into() {
                event.window = xlib::XDefaultRootWindow(self.display);
            }
            xlib::XSetInputFocus(self.display, event.window, RevertToNone, CurrentTime);
        }
    }

    fn show_window(&self, window: u64) {
        unsafe { xlib::XMapWindow(self.display, window) };
    }
    fn hide_window(&self, window: u64) {
        println!("hiding window {}", window);
        unsafe { xlib::XUnmapWindow(self.display, window) };
    }
}
