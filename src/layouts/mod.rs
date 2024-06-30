use crate::{tdawm::Context, tdawm::TDAWmError, x11::X11Adapter};

mod horizontal;
pub use horizontal::*;
mod vertical;
pub use vertical::*;

pub trait Layout {
    fn id(&self) -> String;
    fn layout(&mut self, server: &mut X11Adapter, context: &mut Context) -> Result<(), TDAWmError>;
}
