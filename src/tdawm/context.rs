use std::collections::HashMap;

use super::{Screen, Window, WindowId};

pub struct Context {
    pub screens: Vec<Screen>,
    pub windows_by_id: HashMap<WindowId, Window>,
}

impl Context {
    pub fn focused_screen_mut(&mut self, mouse_position: (i16, i16)) -> &mut Screen {
        let index = self.focused_screen_index(mouse_position);
        self.screens.get_mut(index).unwrap()
    }
    pub fn focused_screen(&self, mouse_position: (i16, i16)) -> &Screen {
        self.screens
            .get(self.focused_screen_index(mouse_position))
            .unwrap()
    }
    fn focused_screen_index(&self, mouse_position: (i16, i16)) -> usize {
        let (mouse_x, mouse_y) = mouse_position;
        return self
            .screens
            .iter()
            .enumerate()
            .find(|(_, screen)| {
                // Is the mouse inside the screen rect
                screen.x < mouse_x
                    && screen.x + screen.width as i16 > mouse_x
                    && screen.y < mouse_y
                    && screen.y + screen.height as i16 > mouse_y
            })
            .map_or(0, |(i, _)| i);
    }
}
