use eframe::egui::{self, SelectableLabel, Ui};
use egui_notify::Toasts;
use image::{imageops::FilterType, DynamicImage, GenericImageView};
use scorched::{log_this, logf, LogData, LogImportance};
use std::{cmp::Ordering, sync::OnceLock, time::Duration};

use crate::{capture::CAPTURE_SETTINGS, paths, save_config_option, wooting, ConfigChange};

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
        crate::capture::CAPTURE_SETTINGS_RELOAD.store(true, std::sync::atomic::Ordering::Relaxed);
        *current = new;
    }
}

pub fn clean_logs_button(ui: &mut Ui, toasts: &mut Toasts) {
    if ui.button("Clean Logs").on_hover_text("Cleans the logs folder").clicked() {
        match std::fs::remove_dir_all(paths::logging_path()) {
            Ok(_) => {
                log_this(LogData {
                    importance: scorched::LogImportance::Info,
                    message: "Logs folder has been cleaned".to_string(),
                });
                toasts
                    .info("Logs folder has been cleaned")
                    .set_duration(Some(Duration::from_secs(3)));
            }
            Err(e) => {
                logf!(Error, "Failed to clean logs folder: {}", e);
                toasts
                    .error(format!("Failed to clean logs folder: {}", e))
                    .set_duration(Some(Duration::from_secs(5)));
            }
        }
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
    device_version: &mut String,
    init: &mut bool,
    frame_rgb_size: (u32, u32),
) {
    ui.horizontal(|ui| {
        ui.heading("Device Info");
        if ui.add(egui::Button::new("Refresh")).on_hover_text("Refreshes the device info, devices should instantly be picked up automatically, but if you have multiple wooting devices plugged in or you want to force refresh you can with this.").clicked() {
            toasts
                .info("Refreshing Device Info")
                .set_duration(Some(std::time::Duration::from_secs(1)));

            crate::capture::CAPTURE_LOCK.store(true, std::sync::atomic::Ordering::Relaxed);
            wooting::reconnect_device();
            crate::capture::CAPTURE_LOCK.store(false, std::sync::atomic::Ordering::Relaxed);

            *device_name = wooting::get_device_name();
            *device_creation = wooting::get_device_creation(0);
            *init = true;
        }
    });
    ui.add(egui::Label::new(format!("Name: {}", device_name,)));
    ui.label(format!("Creation: {}", device_creation)).on_hover_text("This is the manufacture date found on your device's board; this may differ from when you received the device");
    ui.horizontal(|ui| {
        ui.label(format!("Firmware Version: {}", device_version));
        /*
        if device_version.1 {
            ui.label(" (Unsupported)").highlight().on_hover_text("This firmware version is unsupported, please downgrade to a supported version for the time being (<=2.8.0)");
        }
        */
    });

    display_lighting_dimensions(ui, frame_rgb_size);
}

fn display_lighting_dimensions(ui: &mut egui::Ui, frame_rgb_size: (u32, u32)) {
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

static LATEST_VER: OnceLock<Option<String>> = OnceLock::new();

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

        let latest_ver = match LATEST_VER.get() {
            Some(ver) => ver.clone(),
            None => {
                let ver = call_dynamic_get_lastest_ver(format!("{}/", paths::logging_path().as_path().display()));

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
            Ordering::Less => {
                ui.separator();
                ui.label("New Version Available").on_hover_ui(|ui| {
                    ui.label(format!("New version available: {}", <std::option::Option<std::string::String> as Clone>::clone(&latest_ver).unwrap()));
                    ui.hyperlink_to("Download", format!(
                        "https://github.com/MrEnder0/wootili-view/releases/tag/{}",
                        <std::option::Option<std::string::String> as Clone>::clone(&latest_ver).unwrap()
                    ));
                });
            }
            Ordering::Greater => {
                ui.separator();
                ui.label("Developer Build").highlight().on_hover_text("This build has been detected as unpublished meaning this is most likley a developer build or a pulled release, this build may be unstable or have unfinished/broken features");
            }
            _ => {}
        }
    });
}

fn call_dynamic_get_lastest_ver(log_path: String) -> Option<String> {
    unsafe {
        let lib = match libloading::Library::new("update_check") {
            Ok(lib) => lib,
            Err(_) => {
                log_this(LogData {
                    importance: scorched::LogImportance::Warning,
                    message: "Failed to load update_check cdylib".to_string(),
                });
                return None;
            }
        };
        let get_lastest_ver: libloading::Symbol<extern "C" fn(String) -> Option<String>> =
            match lib.get("get_lastest_ver".as_bytes()) {
                Ok(func) => func,
                Err(_) => {
                    log_this(LogData {
                        importance: scorched::LogImportance::Error,
                        message: "Failed to get get_lastest_ver function from cdylib".to_string(),
                    });
                    return None;
                }
            };

        match get_lastest_ver(log_path) {
            Some(ver) => Some(ver),
            None => {
                log_this(LogData {
                    importance: scorched::LogImportance::Warning,
                    message: "Cdylib failed to get the lastest version".to_string(),
                });
                None
            }
        }
    }
}
