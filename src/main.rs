use std::error::Error;

use crate::miniwm::MiniWM;

mod miniwm;
fn main() -> Result<(), Box<dyn Error>> {
    let display_name = std::env::var("DISPLAY")?;

    let mut wm = MiniWM::new(&display_name)?;
    wm.init()?;
    wm.run()?;
    Ok(())
}
