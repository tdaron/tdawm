use std::{error::Error, fs, path::Path};

use execute::{shell, Execute};
use log::{error, info, trace};

use crate::tdawm::TDAWm;

mod config;
mod tdawm;
mod workspace;
mod x11;

fn main() {
    let path = Path::new("/tmp/tdawm_log.txt");
    if path.exists() {
        fs::remove_file(path).unwrap();
    }
    let _log2 = log2::open(path.to_str().unwrap())
        .module(true)
        .tee(true)
        .rotate(5)
        .start();
    if let Err(e) = run() {
        error!("{}", e);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let display_name = std::env::var("DISPLAY")?;
    let user_config: config::Config = config::load_config()?;
    info!("running startup");
    for cmd in user_config.startup.iter() {
        info!("executing {}", cmd);
        let mut command = shell(cmd);
        command.execute()?;
    }
    let adapter = x11::X11Adapter::new(&display_name)?;
    let mut wm = TDAWm::new(adapter)?;
    // if let Ok(_) = env::var("XEPHYR") {
    //     // wm.set_modifier_to_control();
    // }
    wm.run()?;
    Ok(())
}
