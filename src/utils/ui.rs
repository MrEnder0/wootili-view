use eframe::egui::{self, SelectableLabel, Ui};
use egui_notify::Toasts;
use image::{imageops::FilterType, DynamicImage, GenericImageView};
use scorched::{log_this, logf, LogData, LogImportance};
use std::{sync::atomic::Ordering, time::Duration};

use crate::{
    save_config_option, utils::capture::CAPTURE_SETTINGS, utils::paths, wooting, ConfigChange,
};

use super::capture;

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
        capture::CAPTURE_SETTINGS_RELOAD.store(true, Ordering::Relaxed);
        *current = new;
    }
}

pub fn clean_logs_button(ui: &mut Ui, toasts: &mut Toasts) {
    if ui
        .button("Clean Logs")
        .on_hover_text("Cleans the logs folder")
        .clicked()
    {
        match std::fs::remove_dir_all(paths::logging_path()) {
            Ok(_) => {
                log_this(LogData {
                    importance: scorched::LogImportance::Info,
                    message: "Logs folder has been cleaned".to_string(),
                });
                toasts
                    .info("Logs folder has been cleaned")
                    .duration(Some(Duration::from_secs(3)));
            }
            Err(e) => {
                logf!(Error, "Failed to clean logs folder: {}", e);
                toasts
                    .error(format!("Failed to clean logs folder: {}", e))
                    .duration(Some(Duration::from_secs(5)));
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
                .duration(Some(std::time::Duration::from_secs(1)));

            capture::CAPTURE_LOCK.store(true, Ordering::Relaxed);
            wooting::reconnect_device();
            capture::CAPTURE_LOCK.store(false, Ordering::Relaxed);

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
