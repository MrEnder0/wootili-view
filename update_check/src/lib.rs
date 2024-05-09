use reqwest::header::{HeaderMap, USER_AGENT};

#[no_mangle]
pub extern "C" fn get_lastest_ver() -> Option<String> {
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
            return None
        }
    };

    let content = match response.text() {
        Ok(content) => content,
        Err(_) => {
            return None
        }
    };

    let json = match serde_json::from_str::<serde_json::Value>(&content) {
        Ok(json) => json,
        Err(_) => {
            return None
        }
    };

    let tag_name = match json["tag_name"].as_str() {
        Some(tag_name) => tag_name,
        None => {
            return None
        }
    };

    Some(tag_name.to_string())
}