use reqwest::header::{HeaderMap, USER_AGENT};
use scorched::{logf, set_logging_path, LogData, LogImportance};

#[no_mangle]
pub extern "C" fn get_lastest_ver(log_path: String) -> Option<String> {
    set_logging_path(log_path.as_str());

    match get_version_info() {
        Ok(version) => {
            logf!(Info, "Successfully fetched lastest version: {}", version);

            Some(version)
        }
        Err(err) => {
            logf!(Error, "Failed to get latest version because of the following error: {}", err);

            None
        }
    }
}

fn get_version_info() -> Result<String, Box<dyn std::error::Error>> {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, "Wootili-View Version Check".parse().unwrap());

    let client = reqwest::blocking::Client::builder()
        .default_headers(headers)
        .build()
        .unwrap();

    let content = client
        .get("https://api.github.com/repos/MrEnder0/Wootili-view/releases/latest")
        .send()?
        .text()?;

    let json = serde_json::from_str::<serde_json::Value>(&content)?;

    match json["tag_name"].as_str() {
        Some(tag_name) => Ok(tag_name.to_string()),
        None => Err("Failed to get tag_name element in json".into()),
    }
}
