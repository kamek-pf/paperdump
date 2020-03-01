mod config;

use std::path::Path;
use std::{process, thread};

use anyhow::Result;
use crossbeam_channel::unbounded as channel;
use log::{debug, error};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};

use config::Config;

fn main() -> Result<()> {
    pretty_env_logger::init();
    let config = Config::init()?;
    debug!("Config: {:?}", config);

    initialize_watcher(&config.watch_folder)?;
    // let (tx, rx) = channel();

    // loop {
    //     match rx.recv() {}
    // }
    Ok(())
}

fn initialize_watcher(path: &Path) -> Result<()> {
    // Initialize watcher, setup event forwarding
    let mut watcher: RecommendedWatcher = Watcher::new_immediate(|res| match res {
        Ok(event) => println!("event: {:?}", event),
        Err(e) => {
            error!("Filesystem watcher error: {}", e);
            process::exit(-1);
        }
    })?;

    // Set folder to monitor
    watcher.watch(path, RecursiveMode::NonRecursive)?;

    Ok(())
}
