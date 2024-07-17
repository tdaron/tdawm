use super::{WindowId, Workspace};

#[derive(Debug)]
pub struct Screen {
    pub width: u32,
    pub height: u32,
    pub x: i16,
    pub y: i16,
    pub workspaces: Vec<Workspace>,
    pub current_workspace_id: usize,
    pub focused_window: WindowId,
}

impl Screen {
    pub fn new_screen(width: u32, height: u32, x: i16, y: i16) -> Self {
        Self {
            width,
            height,
            x,
            y,
            workspaces: std::iter::repeat_with(Workspace::new).take(10).collect(),
            current_workspace_id: 0,
            focused_window: 0,
        }
    }
    pub fn window_workspace(&self, id: WindowId) -> Option<&Workspace> {
        self.workspaces.iter().find(|w| w.windows.contains(&id))
    }
    pub fn has_window_visible(&self, id: WindowId) -> bool {
        if self
            .current_workspace()
            .windows
            .iter()
            .filter(|w| **w == id)
            .count()
            != 0
        {
            return true;
        }
        false
    }
    pub fn current_workspace_mut(&mut self) -> &mut Workspace {
        self.workspaces.get_mut(self.current_workspace_id).unwrap()
    }
    pub fn current_workspace(&self) -> &Workspace {
        self.workspaces.get(self.current_workspace_id).unwrap()
    }
}
