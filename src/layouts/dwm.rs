use crate::tdawm::WindowId;

use super::Layout;

pub struct DWMLayout {
    master: WindowId,
}
impl DWMLayout {
    pub fn init() -> Self {
        DWMLayout { master: 0 }
    }
}
impl Layout for DWMLayout {
    fn id(&self) -> String {
        "dwm".into()
    }

    fn set_master(&mut self, window: WindowId) {
        self.master = window;
    }
    fn layout(
        &mut self,
        server: &mut crate::x11::X11Adapter,
        ctx: &mut crate::tdawm::Context,
    ) -> Result<(), crate::tdawm::TDAWmError> {
        for screen in ctx.screens.iter_mut() {
            let ws = screen.current_workspace();
            let length = ws.iter_normal_windows(&ctx.windows_by_id).count() as u32;
            if length == 0 {
                // not any windows
                return Ok(());
            }
            if length == 1 {
                //first and the only window
                let window = ws.iter_normal_windows(&ctx.windows_by_id).next().unwrap();
                server.move_window(window.id, screen.x as i32, screen.y as i32);
                server.resize_window(window.id, screen.width, screen.height);
                server.show_window(window.id);
            } else {
                let not_master_height = screen.height / (length - 1);
                let width = screen.width / 2;
                let mut i = 0;
                for window in ws.iter_normal_windows(&ctx.windows_by_id) {
                    if window.id != self.master {
                        //not master
                        let x = screen.x as i32 + width as i32;
                        let y = screen.y as i32 + ((i as u32) * not_master_height) as i32;
                        i += 1;
                        server.move_window(window.id, x, y);
                        server.resize_window(window.id, width, not_master_height);
                    } else {
                        //master
                        server.move_window(window.id, screen.x as i32, screen.y as i32);
                        server.resize_window(window.id, width, screen.height);
                    }
                    server.show_window(window.id);
                }
            }
        }
        Ok(())
    }
}
