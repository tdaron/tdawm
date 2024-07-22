use std::collections::{BTreeSet, HashMap};

use log::error;

use crate::tdawm::WindowType;

use super::{Window, WindowId};
#[derive(Debug)]
pub struct Workspace {
    // simple ordered set of windows
    pub windows: BTreeSet<WindowId>,
}

impl Workspace {
    pub fn new() -> Workspace {
        Workspace {
            windows: BTreeSet::new(),
        }
    }
    pub fn iter_normal_windows<'a>(
        &'a self,
        windows: &'a HashMap<WindowId, Window>,
    ) -> impl Iterator<Item = &'a Window> + 'a {
        self.windows
            .iter()
            .map(|w_id| {
                error!("id: {}, {:?}", w_id, windows.get(&w_id));
                windows.get(&w_id).unwrap()
            })
            .filter(move |w| matches!(w.window_type, WindowType::Normal))
    }
    pub fn add_window(&mut self, window: WindowId) {
        self.windows.insert(window);
    }
    pub fn remove_window(&mut self, window: &WindowId) {
        self.windows.remove(window);
    }
}
