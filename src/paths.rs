use std::{env::home_dir, path::PathBuf};

use scorched::{log_this, LogData, LogExpect, LogImportance};

pub fn logging_path() -> PathBuf {
    match home_dir() {
        Some(path) => path
            .join("AppData")
            .join("Local")
            .join("Wootili-View")
            .join("logs"),
        None => {
            log_this(LogData {
                importance: LogImportance::Warning,
                message: "Unable to get home directory, defaulting to current path".to_string(),
            });
            std::env::current_dir()
                .log_expect(LogImportance::Error, "Unable to get current directory")
                .join("logs")
        }
    }
}

pub fn config_path() -> PathBuf {
    match home_dir() {
        Some(path) => path.join("AppData").join("Local").join("Wootili-View"),
        None => {
            log_this(LogData {
                importance: LogImportance::Warning,
                message: "Unable to get home directory, defaulting to current path".to_string(),
            });
            std::env::current_dir()
                .log_expect(LogImportance::Error, "Unable to get current directory")
        }
    }
}
