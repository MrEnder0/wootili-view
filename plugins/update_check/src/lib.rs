use reqwest::header::{HeaderMap, USER_AGENT};
use scorched::{log_this, set_logging_path, LogData, LogImportance};

#[no_mangle]
pub extern "C" fn get_lastest_ver(log_path: String) -> Option<String> {
    set_logging_path(log_path.as_str());

    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, "Wootili-View Version Check".parse().unwrap());

    let client = reqwest::blocking::Client::builder()
        .default_headers(headers)
        .build()
        .unwrap();

    let response = match client
        .get("https://api.github.com/repos/MrEnder0/Wootili-view/releases/latest")
        .send()
    {
        Ok(response) => response,
        Err(_) => {
            log_this(LogData {
                importance: LogImportance::Warning,
                message: "Failed to get lastest version info".to_string(),
            });
            return None;
        }
    };

    let content = match response.text() {
        Ok(content) => content,
        Err(_) => {
            log_this(LogData {
                importance: LogImportance::Warning,
                message: "Unable to read lastest version info".to_string(),
            });
            return None;
        }
    };

    let json = match serde_json::from_str::<serde_json::Value>(&content) {
        Ok(json) => json,
        Err(_) => {
            log_this(LogData {
                importance: LogImportance::Warning,
                message: "Unable to parse version data into json".to_string(),
            });
            return None;
        }
    };

    let tag_name = match json["tag_name"].as_str() {
        Some(tag_name) => tag_name,
        None => {
            log_this(LogData {
                importance: LogImportance::Warning,
                message: "Unable to get version info from json".to_string(),
            });
            return None;
        }
    };

    log_this(LogData {
        importance: LogImportance::Info,
        message: format!("Successfully got lastest version info: {}", tag_name),
    });

    Some(tag_name.to_string())
}
