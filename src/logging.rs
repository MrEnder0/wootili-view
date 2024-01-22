use std::{env::home_dir, path::PathBuf};

pub fn logging_path() -> PathBuf {
    match home_dir() {
        Some(path) => path
            .join("AppData")
            .join("Local")
            .join("Wootili-View")
            .join("logs"),
        None => std::env::current_dir().unwrap(),
    }
}
