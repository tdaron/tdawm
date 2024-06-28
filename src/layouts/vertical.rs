use std::{cell::RefCell, rc::Rc};

use log::trace;

use crate::{tdawm::TDAWmError, workspace::Workspace, x11::X11Adapter};

use super::Layout;

pub struct VerticalLayout;

impl VerticalLayout {
    pub fn init() -> VerticalLayout {
        return VerticalLayout {};
    }
}

impl Layout for VerticalLayout {
    fn layout(
        &mut self,
        server: &mut X11Adapter,
        current_workspace: &mut Rc<RefCell<Workspace>>,
        workspaces: &mut Vec<Rc<RefCell<Workspace>>>,
    ) -> Result<(), TDAWmError> {
        trace!("computing layout..");
        let screen = server
            .screens
            .first()
            .ok_or_else(|| TDAWmError::NoScreenFound)?;

        let ws = current_workspace.borrow();
        let length = ws.windows.len() as u32;
        if length == 0 {
            // not any windows
            return Ok(());
        }
        // Each window will get 100%/nbr of windows height and 100% width
        let window_height = screen.height / length;
        for (i, window) in ws.windows.iter().enumerate() {
            server.resize_window(*window, screen.width, window_height);
            server.move_window(
                *window,
                screen.x as i32,
                screen.y as i32 + window_height as i32 * i as i32,
            );
            server.show_window(*window);
        }

        Ok(())
    }

    fn id(&self) -> String {
        String::from("vertical")
    }
}
