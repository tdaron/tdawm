use std::{error::Error, fs, path::Path};

use execute::{shell, Execute};
use log::{error, trace};

use crate::tdawm::TDAWm;

mod config;
mod tdawm;

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
