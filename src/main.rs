use std::error::Error;

use env_logger::Env;

use crate::tdawm::TDAWm;

mod tdawm;
fn main() -> Result<(), Box<dyn Error>> {
    let env = Env::default().filter_or("LOG_LEVEL", "info");
    env_logger::init_from_env(env);
    let display_name = std::env::var("DISPLAY")?;
    let mut wm = TDAWm::new(&display_name)?;
    wm.init()?;
    wm.run()?;
    Ok(())
}
