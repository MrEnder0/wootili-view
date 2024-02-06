use eframe::egui::{self, Hyperlink, SelectableLabel, Ui};
use egui_notify::Toasts;
use image::{imageops::FilterType, DynamicImage, GenericImageView};
use reqwest::header::{HeaderMap, USER_AGENT};
use scorched::{log_this, LogData};
use std::sync::OnceLock;

use crate::{capture::CAPTURE_SETTINGS, save_config_option, wooting, ConfigChange};

pub fn downscale_label(
    ui: &mut Ui,
    current: &mut FilterType,
    new: FilterType,
    label: &str,
    hover_text: &str,
    toasts: &mut Toasts,
) {
    if ui
        .add(SelectableLabel::new(*current == new, label))
        .on_hover_text(hover_text)
        .clicked()
    {
        save_config_option(ConfigChange::DownscaleMethod(new), toasts);
        CAPTURE_SETTINGS.write().unwrap().downscale_method = new;
        *crate::capture::CAPTURE_SETTINGS_RELOAD.write().unwrap() = true;
        *current = new;
    }
}

pub fn rgb_preview(ui: &mut egui::Ui, frame_rgb_size: (u32, u32), resized_capture: DynamicImage) {
    if frame_rgb_size == resized_capture.dimensions() {
        ui.heading("Preview Lighting");
        for y in 0..frame_rgb_size.1 {
            ui.horizontal(|ui| {
                for x in 0..frame_rgb_size.0 {
                    let color: egui::Color32 = {
                        let image::Rgba([r, g, b, _]) = resized_capture.get_pixel(x, y);
                        egui::Color32::from_rgb(r, g, b)
                    };

                    let size = egui::Vec2::new(10.0, 10.0);
                    let rect = ui.allocate_space(size);
                    ui.painter().rect_filled(rect.1, 1.0, color);
                }
            });
        }

        ui.separator();
    }
}

pub fn display_device_info(
    ui: &mut egui::Ui,
    toasts: &mut Toasts,
    device_name: &mut String,
    device_creation: &mut String,
    init: &mut bool,
) {
    ui.horizontal(|ui| {
        ui.heading("Device Info");
        if ui.add(egui::Button::new("Refresh")).on_hover_text("Refreshes the device info, devices should instantly be picked up automatically, but if you have multiple wooting devices plugged in or you want to force refresh you can with this.").clicked() {
            toasts
                .info("Refreshing Device Info")
                .set_duration(Some(std::time::Duration::from_secs(1)));

            *crate::capture::CAPTURE_LOCK.write().unwrap() = true;
            wooting::reconnect_device();
            std::thread::sleep(std::time::Duration::from_millis(100));
            *crate::capture::CAPTURE_LOCK.write().unwrap() = false;

            *device_name = wooting::get_device_name();
            *device_creation = wooting::get_device_creation();
            *init = true;
        }
    });
    ui.add(egui::Label::new(format!("Name: {}", device_name,)));
    ui.label(format!("Creation: {}", device_creation));
}

pub fn display_lighting_dimensions(ui: &mut egui::Ui, frame_rgb_size: (u32, u32)) {
    let lighting_dimensions = if frame_rgb_size.0 == 0 && frame_rgb_size.1 == 0 {
        "Unknown".to_string()
    } else {
        format!("{}x{}", frame_rgb_size.0, frame_rgb_size.1)
    };
    ui.add(egui::Label::new(format!(
        "Lighting Dimensions: {}",
        lighting_dimensions
    )));
}

fn get_lastest_ver() -> String {
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
                importance: scorched::LogImportance::Warning,
                message: "Failed to get lastest version info".to_string(),
            });
            return "Unknown".to_string();
        }
    };

    let content = match response.text() {
        Ok(content) => content,
        Err(_) => {
            log_this(LogData {
                importance: scorched::LogImportance::Warning,
                message: "Unable to read lastest version info".to_string(),
            });
            return "Unknown".to_string();
        }
    };

    let json = match serde_json::from_str::<serde_json::Value>(&content) {
        Ok(json) => json,
        Err(_) => {
            log_this(LogData {
                importance: scorched::LogImportance::Warning,
                message: "Unable to parse version data into json".to_string(),
            });
            return "Unknown".to_string();
        }
    };

    let tag_name = match json["tag_name"].as_str() {
        Some(tag_name) => tag_name,
        None => {
            log_this(LogData {
                importance: scorched::LogImportance::Warning,
                message: "Unable to get version info from json".to_string(),
            });
            return "Unknown".to_string();
        }
    };

    log_this(LogData {
        importance: scorched::LogImportance::Info,
        message: format!("Successfully got lastest version info: {}", tag_name),
    });

    tag_name.to_string()
}

static LATEST_VER: OnceLock<String> = OnceLock::new();

pub fn version_footer(ui: &mut egui::Ui, check_for_updates: bool) {
    ui.horizontal(|ui| {
        ui.hyperlink_to(
            format!("Wootili-View {}", env!("CARGO_PKG_VERSION")),
            format!(
                "https://github.com/MrEnder0/wootili-view/releases/tag/{}",
                env!("CARGO_PKG_VERSION")
            ),
        );

        if !check_for_updates {
            return;
        }

        let latest_ver = LATEST_VER.get_or_init(get_lastest_ver);

        if latest_ver == "Unknown" {
            ui.separator();
            ui.label("Failed to check for updates").on_hover_text(
                "Failed to check for updates, try checking your internet connection",
            );
        } else if latest_ver != env!("CARGO_PKG_VERSION") {
            ui.separator();
            ui.add(Hyperlink::from_label_and_url(
                format!("Update Available: {}", latest_ver),
                format!(
                    "https://github.com/MrEnder0/wootili-view/releases/tag/{}",
                    latest_ver
                ),
            ));
        }
    });
}
