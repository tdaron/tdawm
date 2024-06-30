use log::trace;

use crate::{tdawm::Context, tdawm::TDAWmError, x11::X11Adapter};

use super::Layout;

pub struct HorizontalLayout;

impl HorizontalLayout {
    pub fn init() -> HorizontalLayout {
        return HorizontalLayout {};
    }
}

impl Layout for HorizontalLayout {
    fn layout(&mut self, server: &mut X11Adapter, ctx: &mut Context) -> Result<(), TDAWmError> {
        trace!("computing layout..");
        let screen = server
            .screens
            .first()
            .ok_or_else(|| TDAWmError::NoScreenFound)?;

        let ws = ctx.workspaces.get(ctx.current_workspace_id).unwrap();
        let length = ws.iter_normal_windows(&ctx.windows_by_id).count() as u32;
        if length == 0 {
            // not any windows
            return Ok(());
        }
        // Each window will get 100%/nbr of windows width and 100% height
        let window_width = screen.width / length;
        for (i, window) in ws.iter_normal_windows(&ctx.windows_by_id).enumerate() {
            server.resize_window(window.id, window_width, screen.height);
            server.move_window(
                window.id,
                screen.x as i32 + window_width as i32 * i as i32,
                screen.y as i32,
            );
            server.show_window(window.id);
        }

        Ok(())
    }
    fn id(&self) -> String {
        String::from("horizontal")
    }
}
