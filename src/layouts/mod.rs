use std::{cell::RefCell, rc::Rc};

use crate::{tdawm::TDAWmError, workspace::Workspace, x11::X11Adapter};

mod horizontal;
pub use horizontal::*;
mod vertical;
pub use vertical::*;

pub trait Layout {
    fn id(&self) -> String;
    fn layout(
        &mut self,
        server: &mut X11Adapter,
        current_workspace: &mut Rc<RefCell<Workspace>>,
        workspaces: &mut Vec<Rc<RefCell<Workspace>>>,
    ) -> Result<(), TDAWmError>;
}
