use std::collections::HashMap;

use super::{Window, WindowId, Workspace};

pub struct Context {
    pub workspaces: Vec<Workspace>,
    pub current_workspace_id: usize,
    pub windows_by_id: HashMap<WindowId, Window>,
}
