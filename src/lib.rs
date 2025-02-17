use std::{
    collections::HashMap,
    error::Error,
    fs::read,
    process::{Command, Stdio},
};

use nix::sys::inotify::{AddWatchFlags, InitFlags, Inotify};
use sha2::{Digest, Sha256};

struct FileRef {
    path: String,
    hash: [u8; 32],
}

fn hash_file(path: &str) -> Result<[u8; 32], Box<dyn Error>> {
    let mut hasher = Sha256::new();
    let b = read(&path).map_err(|e| format!("error reading {}: {:?}", path, e))?;
    hasher.update(b);
    let hash = hasher.finalize();
    Ok(hash.try_into().unwrap())
}

pub fn watch(paths: &[String], then: String) -> Result<(), Box<dyn Error>> {
    let mut watch_descriptor_to_file_ref = HashMap::new();

    let inotify = Inotify::init(InitFlags::empty()).unwrap();

    let flags = AddWatchFlags::IN_MODIFY | AddWatchFlags::IN_ONESHOT;

    for path in paths {
        let watch_descriptor = inotify.add_watch(path.as_str(), flags).map_err(|e| {
            format!(
                "Unable to watch {:?}, does the file exist? error={:?}",
                path, e
            )
        })?;

        watch_descriptor_to_file_ref.insert(
            watch_descriptor,
            FileRef {
                path: path.to_string(),
                hash: hash_file(&path)?,
            },
        );
    }

    loop {
        let events = inotify.read_events().unwrap();

        let mut paths = Vec::new();
        for event in events {
            let FileRef { path, hash } = watch_descriptor_to_file_ref.remove(&event.wd).unwrap();

            let watch_descriptor = inotify.add_watch(path.as_str(), flags).map_err(|e| {
                format!(
                    "Unable to watch {:?}, does the file exist? error={:?}",
                    path, e
                )
            })?;

            let new_hash = hash_file(&path)?;
            if hash != new_hash {
                paths.push(path.to_string());
            }

            watch_descriptor_to_file_ref.insert(
                watch_descriptor,
                FileRef {
                    path,
                    hash: new_hash,
                },
            );
        }

        for path in &paths {
            let now = chrono::Local::now();
            let now = now.format("%Y-%m-%d %I:%M:%S %p");

            println!("{}: running \"{} {}\"", now, then, path);

            Command::new(&then)
                .arg(path)
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .spawn()
                .unwrap()
                .wait()
                .unwrap();
        }

        if !paths.is_empty() {
            println!("Waiting for changes...");
        }
    }
}
