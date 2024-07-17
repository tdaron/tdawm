use std::collections::HashMap;

use super::{Screen, Window, WindowId};

pub struct Context {
    pub screens: Vec<Screen>,
    pub windows_by_id: HashMap<WindowId, Window>,
    pub focused_screen: usize,
}

impl Context {
    pub fn focused_screen_mut(&mut self) -> &mut Screen {
        self.screens.get_mut(self.focused_screen).unwrap()
    }
    pub fn focused_screen(&self) -> &Screen {
        self.screens.get(self.focused_screen).unwrap()
    }
}
