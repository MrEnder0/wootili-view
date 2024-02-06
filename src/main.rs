#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod capture;
mod config;
mod paths;
mod ui;
mod wooting;

use capture::{capture, CAPTURE_PREVIEW, CAPTURE_SETTINGS};
use config::*;
use eframe::egui;
use egui_notify::Toasts;
use image::imageops::FilterType;
use scorched::{logf, LogData, LogImportance};
use std::time::Duration;
use ui::*;
use xcap::Monitor;

use crate::capture::{CaptureSettings, CAPTURE_LOCK, CAPTURE_SETTINGS_RELOAD};

fn main() -> Result<(), eframe::Error> {
    scorched::set_logging_path(format!("{}/", paths::logging_path().as_path().display()).as_str());

    if !config_exists() {
        gen_config();
    }

    wooting::update_rgb();

    *CAPTURE_LOCK.write().unwrap() = true;

    // Screen thread, captures the screen and stores it in the static SCREEN
    std::thread::spawn(|| {
        capture();
    });

    eframe::run_native(
        "Wootili-View",
        eframe::NativeOptions::default(),
        Box::new(move |_cc| Box::<MyApp>::default()),
    )
}

struct MyApp {
    toasts: Toasts,
    init: bool,
    device_name: String,
    brightness: u8,
    reduce_bright_effects: bool,
    screen: usize,
    display_rgb_preview: bool,
    downscale_method: FilterType,
    frame_limit: (u8, u8),
    red_shift_fix: bool,
    dark_mode: bool,
    check_updates: bool,
    device_creation: String,
    rgb_size: (u32, u32),
    next_frame: Duration,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            toasts: Toasts::default(),
            init: true,
            device_name: wooting::get_device_name(),
            brightness: 100,
            reduce_bright_effects: false,
            screen: 0,
            display_rgb_preview: true,
            downscale_method: FilterType::Triangle,
            frame_limit: (60, 15), // (UI, Capture)
            red_shift_fix: false,
            dark_mode: true,
            check_updates: true,
            device_creation: wooting::get_device_creation(),
            rgb_size: wooting::get_rgb_size().unwrap(),
            next_frame: Duration::from_secs(0),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.init {
            if !cfg!(windows) {
                self.toasts
                    .error("This application is not supported on your operating system")
                    .set_duration(Some(Duration::from_secs(120)));
            }
            logf!(Info, "Connected to device Name: {}", self.device_name);
            match self.device_name.as_str() {
                "N/A" => {
                    self.toasts
                        .error("No Wooting Device Found")
                        .set_duration(Some(Duration::from_secs(5)));
                }
                _ => {
                    self.toasts
                        .success(format!("Connected to {}", self.device_name))
                        .set_duration(Some(Duration::from_secs(3)));
                }
            };

            let config = match read_config() {
                Some(config) => config,
                None => {
                    reset_config();

                    self.toasts
                        .warning("Config file has been reset due to a config format error")
                        .set_duration(Some(Duration::from_secs(5)));

                    read_config().unwrap()
                }
            };

            self.brightness = config.brightness;
            self.reduce_bright_effects = config.reduce_bright_effects;
            self.screen = config.screen;
            self.display_rgb_preview = config.display_rgb_preview;
            self.downscale_method = downscale_index_to_filter(config.downscale_method_index);
            self.frame_limit = config.frame_limit;
            self.red_shift_fix = config.red_shift_fix;
            self.dark_mode = config.dark_mode;
            self.check_updates = config.check_updates;

            *CAPTURE_SETTINGS.write().unwrap() = CaptureSettings {
                screen_index: self.screen,
                downscale_method: self.downscale_method,
                capture_frame_limit: self.frame_limit.1.into(),
                reduce_bright_effects: self.reduce_bright_effects,
                red_shift_fix: self.red_shift_fix,
                brightness: self.brightness,
                device_name: self.device_name.clone(),
                rgb_size: wooting::get_rgb_size().unwrap_or((0, 0)),
                display_rgb_preview: self.display_rgb_preview,
            };

            *CAPTURE_SETTINGS_RELOAD.write().unwrap() = true;
            *CAPTURE_LOCK.write().unwrap() = false;

            if self.dark_mode {
                ctx.set_visuals(egui::Visuals::dark());
            } else {
                ctx.set_visuals(egui::Visuals::light());
            }

            self.init = false;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Settings");
            ui.separator();

            ui.heading("Visual");
            if ui.add(egui::Slider::new(&mut self.brightness, 50..=150).text("Brightness")).on_hover_text("Adjusts the brightness of the lighting").changed() {
                save_config_option(ConfigChange::Brightness(self.brightness), &mut self.toasts);
                CAPTURE_SETTINGS.write().unwrap().brightness = self.brightness;
                *CAPTURE_SETTINGS_RELOAD.write().unwrap() = true;
            }
            if ui.add(egui::Slider::new(&mut self.screen, 0..=Monitor::all().unwrap().len() - 1).text("Screen")).on_hover_text("Select the screen to capture").changed() {
                save_config_option(ConfigChange::Screen(self.screen), &mut self.toasts);
                CAPTURE_SETTINGS.write().unwrap().screen_index = self.screen;
                *CAPTURE_SETTINGS_RELOAD.write().unwrap() = true;
            }
            if ui.checkbox(&mut self.reduce_bright_effects, "Reduce Bright Effects").on_hover_text("Reduces brightness when the screen is very bright").changed() {
                save_config_option(ConfigChange::ReduceBrightEffects(self.reduce_bright_effects), &mut self.toasts);
                CAPTURE_SETTINGS.write().unwrap().reduce_bright_effects = self.reduce_bright_effects;
                *CAPTURE_SETTINGS_RELOAD.write().unwrap() = true;
            }
            if ui.checkbox(&mut self.red_shift_fix, "Red Shift Fix").on_hover_text("Fixes the red shift/hue issue on some Wooting keyboards due to the stock keycaps or from custom switches like the Geon Raptor HE").changed() {
                save_config_option(ConfigChange::RedShiftFix(self.red_shift_fix), &mut self.toasts);
                CAPTURE_SETTINGS.write().unwrap().red_shift_fix = self.red_shift_fix;
                *CAPTURE_SETTINGS_RELOAD.write().unwrap() = true;
            }
            ui.menu_button("Downscale Method", |ui| {
                downscale_label(ui, &mut self.downscale_method, FilterType::Nearest, "Nearest", "Fast and picks on up on small details but is inconsistent", &mut self.toasts);
                downscale_label(ui, &mut self.downscale_method, FilterType::Triangle, "Triangle", "Overall good results and is fast, best speed to quality ratio", &mut self.toasts);
                downscale_label(ui, &mut self.downscale_method, FilterType::Gaussian, "Gaussian", "Fast but gives poor results", &mut self.toasts);
                downscale_label(ui, &mut self.downscale_method, FilterType::CatmullRom, "CatmullRom", "Good results but is slow, similar results to Lanczos3", &mut self.toasts);
                downscale_label(ui, &mut self.downscale_method, FilterType::Lanczos3, "Lanczos3", "Gives the best results but is very slow", &mut self.toasts);
            });
            ui.separator();

            ui.heading("Performance");
            if ui.add(egui::Slider::new(&mut self.frame_limit.0, 30..=120).text("UI FPS cap")).on_hover_text("Limits the FPS of the UI").changed() {
                save_config_option(ConfigChange::FrameLimit(self.frame_limit), &mut self.toasts);
            }
            if ui.add(egui::Slider::new(&mut self.frame_limit.1, 1..=60).text("Screen capture FPS cap")).on_hover_text("Limits the FPS of the screen capture for rendering on the device").changed() {
                save_config_option(ConfigChange::FrameLimit(self.frame_limit), &mut self.toasts);
                CAPTURE_SETTINGS.write().unwrap().capture_frame_limit = self.frame_limit.1.into();
                *CAPTURE_SETTINGS_RELOAD.write().unwrap() = true;
            }

            let frame_rgb_size = self.rgb_size;

            let allow_preview = frame_rgb_size.0 != 0 && frame_rgb_size.1 != 0;
            if ui.add_enabled(allow_preview, egui::Checkbox::new(&mut self.display_rgb_preview, "Display RGB Preview")).on_hover_text("Displays a preview of the lighting, this can be disabled to improve performance").changed() {
                save_config_option(ConfigChange::DisplayRgbPreview(self.display_rgb_preview), &mut self.toasts);
            }
            ui.separator();

            ui.heading("Application");
            if ui.checkbox(&mut self.dark_mode, "Darkmode").on_hover_text("Enables darkmode").changed() {
                save_config_option(ConfigChange::Darkmode(self.dark_mode), &mut self.toasts);

                if self.dark_mode {
                    ctx.set_visuals(egui::Visuals::dark());
                } else {
                    ctx.set_visuals(egui::Visuals::light());
                }
            }
            if ui.checkbox(&mut self.check_updates, "Check for Updates").on_hover_text("Checks for updates on startup, wont apply to current session").changed() {
                save_config_option(ConfigChange::CheckUpdates(self.check_updates), &mut self.toasts);
            }
            if ui.button("Reset Config").on_hover_text("Resets the config to the default values").clicked() {
                reset_config();
                self.toasts
                    .info("Config file has been reset")
                    .set_duration(Some(Duration::from_secs(1)));

                let new_config = read_config().unwrap();

                self.brightness = new_config.brightness;
                self.reduce_bright_effects = new_config.reduce_bright_effects;
                self.screen = new_config.screen;
                self.display_rgb_preview = new_config.display_rgb_preview;
                self.downscale_method = downscale_index_to_filter(new_config.downscale_method_index);
                self.frame_limit = new_config.frame_limit;
                self.red_shift_fix = new_config.red_shift_fix;
                self.dark_mode = new_config.dark_mode;
                self.check_updates = new_config.check_updates;

                *CAPTURE_SETTINGS_RELOAD.write().unwrap() = true;

                if self.dark_mode {
                    ctx.set_visuals(egui::Visuals::dark());
                } else {
                    ctx.set_visuals(egui::Visuals::light());
                }
            }
            if ui.button("Clean Logs").on_hover_text("Cleans the logs folder").clicked() {
                match std::fs::remove_dir_all(paths::logging_path()) {
                    Ok(_) => {
                        logf!(Info, "Logs folder has been cleaned");
                        self.toasts
                            .info("Logs folder has been cleaned")
                            .set_duration(Some(Duration::from_secs(3)));
                    }
                    Err(e) => {
                        logf!(Error, "Failed to clean logs folder: {}", e);
                        self.toasts
                            .error(format!("Failed to clean logs folder: {}", e))
                            .set_duration(Some(Duration::from_secs(5)));
                    }
                }
            }

            egui::TopBottomPanel::bottom("footer").show(ctx, |ui| {
                version_footer(ui, self.check_updates);
            });

            egui::SidePanel::right("lighting_preview_panel").show(ctx, |ui| {
                if self.display_rgb_preview {
                    rgb_preview(ui, frame_rgb_size, CAPTURE_PREVIEW.read().unwrap().clone());
                }
                display_device_info(ui, &mut self.toasts, &mut self.device_name, &mut self.device_creation, &mut self.init);
                display_lighting_dimensions(ui, frame_rgb_size);
            });
        });

        self.toasts.show(ctx);

        self.next_frame =
            Duration::from_millis(((1.0 / self.frame_limit.0 as f32) * 1000.0).round() as u64);
        std::thread::sleep(self.next_frame - Duration::from_millis(1));
        ctx.request_repaint()
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        save_config_option(
            ConfigChange::MultipleConfigOptions(vec![
                ConfigChange::Brightness(self.brightness),
                ConfigChange::ReduceBrightEffects(self.reduce_bright_effects),
                ConfigChange::Screen(self.screen),
                ConfigChange::DisplayRgbPreview(self.display_rgb_preview),
                ConfigChange::DownscaleMethod(self.downscale_method),
                ConfigChange::FrameLimit(self.frame_limit),
                ConfigChange::RedShiftFix(self.red_shift_fix),
                ConfigChange::Darkmode(self.dark_mode),
                ConfigChange::CheckUpdates(self.check_updates),
            ]),
            &mut self.toasts,
        );
        wooting::exit_rgb();
    }
}
