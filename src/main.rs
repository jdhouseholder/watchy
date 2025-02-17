use std::{
    collections::HashMap,
    fs::read,
    process::{Command, Stdio},
};

use clap::Parser;
use nix::sys::inotify::{AddWatchFlags, InitFlags, Inotify};
use sha2::{Digest, Sha256};

#[derive(Parser, Debug)]
#[command(version, about, about = "Watches a set of files and runs a command with the file name passed as an argument on change.", long_about = None)]
struct Args {
    #[arg(long)]
    watch: Vec<String>,

    #[arg(long)]
    then: String,
}

struct FileRef {
    path: String,
    hash: [u8; 32],
}

fn hash_file(p: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    let b = read(&p).unwrap();
    hasher.update(b);
    let hash = hasher.finalize();
    hash.try_into().unwrap()
}

fn main() {
    let mut args = Args::parse();

    let mut wd_to_file_ref = HashMap::new();

    let inotify = Inotify::init(InitFlags::empty()).unwrap();

    args.watch.dedup();

    let flags = AddWatchFlags::IN_MODIFY | AddWatchFlags::IN_ONESHOT;

    for p in args.watch {
        let wd = inotify.add_watch(p.as_str(), flags).unwrap();

        wd_to_file_ref.insert(
            wd,
            FileRef {
                path: p.to_string(),
                hash: hash_file(&p),
            },
        );
    }

    loop {
        let events = inotify.read_events().unwrap();

        let mut paths = Vec::new();
        for event in events {
            let FileRef { path, hash } = wd_to_file_ref.remove(&event.wd).unwrap();

            let wd = inotify.add_watch(path.as_str(), flags).unwrap();

            let new_hash = hash_file(&path);
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

            println!("{}: running \"{} {}\"", now, args.then, path);

            Command::new(&args.then)
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
