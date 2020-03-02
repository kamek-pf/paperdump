mod config;

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::{env, process, thread};

use anyhow::Result;
use chrono::Local;
use crossbeam::channel::{select, unbounded as channel, Receiver, Sender};
use leptess::LepTess;
use log::{debug, error, info, trace, warn};
use nanoid::nanoid;
use notify::{
    Event, EventKind, RecommendedWatcher, RecursiveMode, Result as NotifyResult, Watcher,
};

use config::Config;

fn main() -> Result<()> {
    // Set default log level
    let env_filters = env::var("RUST_LOG").unwrap_or_else(|_| String::from("paperdump=info"));
    pretty_env_logger::formatted_builder()
        .parse_filters(&env_filters)
        .init();

    let config = Config::init()?;
    debug!("Config: {:?}", config);
    let cfg = Arc::new(config);

    // Setup file system event notifications
    let (tx, rx) = channel();
    let mut watcher = fs_watcher(tx)?;
    watcher.watch(&cfg.watch_folder, RecursiveMode::NonRecursive)?;

    // Store handles to pending jobs and track their state
    let (state_tx, state_rx) = channel();
    let mut handles: HashMap<PathBuf, Sender<DocumentEvent>> = HashMap::new();

    loop {
        debug!("Pending tasks: {}", handles.len());
        select! {
            // Events received when files are created or modified
            recv(rx) -> msg => match msg {
                Ok(event) => match event {
                    // A file was created, spawn a new task
                    DocumentEvent::Create(path) => {
                        let (file_tx, file_rx) = channel();
                        handles.insert(path.clone(), file_tx);
                        file_watcher(path, cfg.clone(), file_rx, state_tx.clone());
                    },
                    // A chunk was received, forward it to the corresponding task
                    DocumentEvent::Chunk(path) => if let Some(tx) = handles.get(&path) {
                        if let Err(e) = tx.send(DocumentEvent::Chunk(path)) {
                            warn!("Received a chunk but no corresponding task was found: {}", e);
                        }
                    }
                    // We don't care about the rest here
                    _ => (),
                },
                Err(error) => {
                    error!("File system event channel failed: {}", error);
                    return Err(error.into());
                }
            },

            // Events received when we're done processing files
            recv(state_rx) -> msg => match msg {
                Ok(event) => match event {
                    DocumentEvent::Done(path) => if handles.remove(&path).is_some() {
                        info!("{:?} processed successfuly", path);
                    },
                    DocumentEvent::Failed(path) => if handles.remove(&path).is_some() {
                        error!("{:?} could not be processed", path);
                    },
                    _ => ()
                }
                Err(error) => {
                    error!("Files state channel failed: {}", error);
                    return Err(error.into());
                }
            }
        }
    }
}

#[derive(Debug)]
enum DocumentEvent {
    Create(PathBuf),
    Chunk(PathBuf),
    Done(PathBuf),
    Failed(PathBuf),
}

// Watch all FS events
fn fs_watcher(tx: Sender<DocumentEvent>) -> Result<RecommendedWatcher> {
    let watcher = Watcher::new_immediate(move |res: NotifyResult<Event>| {
        trace!("File system event: {:?}", res);
        match res {
            Ok(event) => match event.kind {
                // Create events should only have one path associated with them
                EventKind::Create(_) => {
                    if let Some(path) = event.paths.get(0) {
                        tx.send(DocumentEvent::Create(path.clone()))
                            .expect("Failed to forward file creation event");
                    }
                }
                EventKind::Modify(_) => {
                    if let Some(path) = event.paths.get(0) {
                        tx.send(DocumentEvent::Chunk(path.clone()))
                            .expect("Failed to forward file modification event");
                    }
                }
                _ => (),
            },
            Err(e) => {
                error!("Filesystem watcher error: {}", e);
                process::exit(-1);
            }
        }
    })?;

    Ok(watcher)
}

// Watch a single files and process it when upload is done
fn file_watcher(
    path: PathBuf,
    cfg: Arc<Config>,
    rx: Receiver<DocumentEvent>,
    state_tx: Sender<DocumentEvent>,
) {
    thread::spawn(move || {
        // When the timeout is reached, we consider the file to be fully uploaded
        while rx.recv_timeout(cfg.process_delay).is_ok() {
            trace!("[{:?}] Chunk ...", path);
        }

        // Try to run Tesseract on the input
        let text = LepTess::new(None, &cfg.language).ok().and_then(|mut lt| {
            let str_path = path.to_str()?;
            lt.set_image(str_path);
            let output = lt.get_utf8_text().ok()?;

            Some(output)
        });

        let id = nanoid!(7, &nanoid::alphabet::SAFE);
        let date = Local::now().format("%Y-%m-%d-%H:%M:%S").to_string();
        let output_name = date + "-" + &id;

        let success = text
            // Write the output to a file
            .and_then(|data| {
                let ocr_output = output_name.clone() + ".txt";
                let output_path = cfg.destination_folder.join(ocr_output);
                File::create(&output_path)
                    .ok()
                    .map(|mut file| file.write_all(data.as_bytes()).ok());

                fs::metadata(&output_path).ok().map(|meta| meta.len() > 0)
            })
            // Move the original next the the the Tesseract output
            .and_then(|ocr_ok| {
                if ocr_ok {
                    path.file_name()
                        .and_then(|os_name| os_name.to_str())
                        .map(|name| {
                            let destination =
                                Path::new(&cfg.destination_folder).join(Path::new(&name));

                            let res = fs::rename(path.clone(), destination).ok();
                            res.is_some()
                        })
                } else {
                    Some(false)
                }
            })
            .unwrap_or(false);

        if success {
            state_tx
                .send(DocumentEvent::Done(path))
                .expect("Failed to forward file state event");
        } else {
            state_tx
                .send(DocumentEvent::Failed(path))
                .expect("Failed to forward file state event");
        }
    });
}
