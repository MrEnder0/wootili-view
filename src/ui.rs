use eframe::egui::{self, Hyperlink, SelectableLabel, Ui};
use egui_notify::Toasts;
use image::{imageops::FilterType, DynamicImage, GenericImageView};
use lazy_static::lazy_static;
use reqwest::header::{HeaderMap, USER_AGENT};
use scorched::{log_this, LogData};

use crate::{change_config_option, wooting, ConfigChange, DOWNSCALE_METHOD, RGB_SIZE};

pub fn downscale_label(
    ui: &mut Ui,
    current: &mut FilterType,
    new: FilterType,
    label: &str,
    hover_text: &str,
) {
    if ui
        .add(SelectableLabel::new(*current == new, label))
        .on_hover_text(hover_text)
        .clicked()
    {
        change_config_option(ConfigChange::DownscaleMethod(new));
        DOWNSCALE_METHOD.write().unwrap().clone_from(&new);
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
            wooting::reconnect_device();
            *init = true;
            *device_name = wooting::get_device_name();
            *device_creation = wooting::get_device_creation();
            RGB_SIZE.write().unwrap().clone_from(&wooting::get_rgb_size());
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

lazy_static! {
    static ref LATEST_VER: String = {
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

        tag_name.to_string()
    };
}

pub fn version_footer(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.hyperlink_to(
            format!("Wootili-View {}", env!("CARGO_PKG_VERSION")),
            format!(
                "https://github.com/MrEnder0/wootili-view/releases/tag/{}",
                env!("CARGO_PKG_VERSION")
            ),
        );

        if *LATEST_VER == "Unknown" {
            ui.separator();
            ui.label("Failed to check for updates").on_hover_text(
                "Failed to check for updates, try checking your internet connection",
            );
        } else if *LATEST_VER != env!("CARGO_PKG_VERSION") {
            ui.separator();
            ui.add(Hyperlink::from_label_and_url(
                format!("Update Available: {}", *LATEST_VER),
                format!(
                    "https://github.com/MrEnder0/wootili-view/releases/tag/{}",
                    *LATEST_VER
                ),
            ));
        }
    });
}
