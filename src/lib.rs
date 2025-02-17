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

fn hash_file(p: &str) -> Result<[u8; 32], Box<dyn Error>> {
    let mut hasher = Sha256::new();
    let b = read(&p).map_err(|e| format!("error reading {}: {:?}", p, e))?;
    hasher.update(b);
    let hash = hasher.finalize();
    Ok(hash.try_into().unwrap())
}

pub fn watch(files: &[String], then: String) -> Result<(), Box<dyn Error>> {
    let mut wd_to_file_ref = HashMap::new();

    let inotify = Inotify::init(InitFlags::empty()).unwrap();

    let flags = AddWatchFlags::IN_MODIFY | AddWatchFlags::IN_ONESHOT;

    for p in files {
        let wd = inotify.add_watch(p.as_str(), flags).map_err(|e| {
            format!(
                "Unable to watch {:?}, does the file exist? error={:?}",
                p, e
            )
        })?;

        wd_to_file_ref.insert(
            wd,
            FileRef {
                path: p.to_string(),
                hash: hash_file(&p)?,
            },
        );
    }

    loop {
        let events = inotify.read_events().unwrap();

        let mut paths = Vec::new();
        for event in events {
            let FileRef { path, hash } = wd_to_file_ref.remove(&event.wd).unwrap();

            let wd = inotify.add_watch(path.as_str(), flags).map_err(|e| {
                format!(
                    "Unable to watch {:?}, does the file exist? error={:?}",
                    path, e
                )
            })?;

            let new_hash = hash_file(&path)?;
            if hash != new_hash {
                paths.push(path.to_string());
            }

            wd_to_file_ref.insert(
                wd,
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
