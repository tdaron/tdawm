use std::error::Error;

use env_logger::Env;
use execute::{shell, Execute};
use log::{error, trace};

use crate::tdawm::TDAWm;

mod config;
mod tdawm;

fn main() {
    let env = Env::default().filter_or("LOG_LEVEL", "info");
    env_logger::init_from_env(env);
    if let Err(e) = run() {
        error!("{}", e);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let display_name = std::env::var("DISPLAY")?;
    let user_config: config::Config = config::load_config()?;
    trace!("running startup");
    for cmd in user_config.startup.iter() {
        trace!("executing {}", cmd);
        let mut command = shell(cmd);
        command.execute()?;
    }
    let mut wm = TDAWm::new(&display_name, user_config)?;
    wm.init()?;
    wm.run()?;
    Ok(())
}
