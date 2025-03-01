use std::{cmp::Ordering as cmp, sync::OnceLock};

use eframe::egui;
use reqwest::header::{HeaderMap, USER_AGENT};
use scorched::{log_this, logf, set_logging_path, LogData, LogImportance};

static LATEST_VER: OnceLock<Option<String>> = OnceLock::new();

#[no_mangle]
pub extern "C" fn update_check_ui(ui: &mut egui::Ui, log_path: String) {
    ui.horizontal(|ui| {
        ui.hyperlink_to(
            format!("Wootili-View {}", env!("CARGO_PKG_VERSION")),
            format!(
                "https://github.com/MrEnder0/wootili-view/releases/tag/{}",
                env!("CARGO_PKG_VERSION")
            ),
        );

        let latest_ver = match LATEST_VER.get() {
            Some(ver) => ver.clone(),
            None => {
                let ver = get_lastest_ver(format!("{}/", log_path));

                let _ = LATEST_VER.set(ver.clone());

                ver
            }
        };

        if latest_ver.is_none() {
            ui.separator();
            ui.label("Failed to check for updates").on_hover_text(
                "Failed to check for updates, try checking your internet connection",
            );

            return;
        }

        let version_cmp = match ver_cmp::compare_versions(
            env!("CARGO_PKG_VERSION"),
            &<std::option::Option<std::string::String> as Clone>::clone(&latest_ver).unwrap() as &str,
        ) {
            Ok(version_cmp) => version_cmp,
            Err(_) => {
                log_this(LogData {
                    importance: scorched::LogImportance::Error,
                    message: "Failed to compare versions, this is likly due to a version format error".to_string(),
                });
                return;
            }
        };

        match version_cmp {
            cmp::Less => {
                ui.separator();
                ui.label("New Version Available").on_hover_ui(|ui| {
                    ui.label(format!("New version available: {}", <std::option::Option<std::string::String> as Clone>::clone(&latest_ver).unwrap()));
                    ui.hyperlink_to("Download", format!(
                        "https://github.com/MrEnder0/wootili-view/releases/tag/{}",
                        <std::option::Option<std::string::String> as Clone>::clone(&latest_ver).unwrap()
                    ));
                });
            }
            cmp::Greater => {
                ui.separator();
                ui.label("Developer Build").highlight().on_hover_text("This build has been detected as unpublished meaning this is most likely a developer build or a pulled release; this build may be unstable or have unfinished/broken features.");
            }
            _ => {}
        }
    });
}

#[no_mangle]
pub extern "C" fn get_lastest_ver(log_path: String) -> Option<String> {
    set_logging_path(log_path.as_str());

    match get_version_info() {
        Ok(version) => {
            logf!(Info, "Successfully fetched lastest version: {}", version);

            Some(version)
        }
        Err(err) => {
            logf!(
                Error,
                "Failed to get latest version because of the following error: {}",
                err
            );

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
