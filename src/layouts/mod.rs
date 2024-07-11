use crate::{
    tdawm::Context,
    tdawm::{TDAWmError, WindowId},
    x11::X11Adapter,
};

mod horizontal;
pub use horizontal::*;
mod vertical;
pub use vertical::*;
mod dwm;
pub use dwm::*;

pub trait Layout {
    fn id(&self) -> String;
    fn layout(&mut self, server: &mut X11Adapter, context: &mut Context) -> Result<(), TDAWmError>;
    fn set_master(&mut self, _window: WindowId) {}
}
