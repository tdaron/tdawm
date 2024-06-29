use std::collections::BTreeSet;

use super::Window;
pub struct Workspace {
    // simple ordered set of windows
    pub windows: BTreeSet<Window>,
}

impl Workspace {
    pub fn new() -> Workspace {
        Workspace {
            windows: BTreeSet::new(),
        }
    }
    pub fn add_window(&mut self, window: Window) {
        self.windows.insert(window);
    }
    pub fn remove_window(&mut self, window: &Window) {
        self.windows.remove(window);
    }
    pub fn update_window(&mut self, window: Window) {
        if !self.windows.insert(window) {
            self.windows.remove(&window);
            self.windows.insert(window);
        }
    }
    pub fn count_windows(&self) -> usize {
        self.windows.len()
    }
}
