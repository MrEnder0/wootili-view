#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod utils;

use eframe::{egui, emath::Rangef};
use egui_notify::Toasts;
use image::imageops::FilterType;
use scorched::{logf, LogData, LogImportance};
use std::{
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};
use utils::{
    capture::*,
    config::*,
    plugins::{get_available_plugins, update_check_ui, Plugin},
    ui::*,
    wooting,
};
use xcap::Monitor;

pub static CLOSE_APP: AtomicBool = AtomicBool::new(false);

fn main() -> Result<(), eframe::Error> {
    scorched::set_logging_path(
        format!("{}/", utils::paths::logging_path().as_path().display()).as_str(),
    );

    if !config_exists() {
        gen_config();
    }

    utils::wooting::update_rgb();

    CAPTURE_LOCK.store(true, Ordering::Relaxed);

    // Screen thread, captures the screen and sends it to the device
    std::thread::spawn(|| {
        capture();
    });

    while CLOSE_APP.load(Ordering::Relaxed) == false {
        eframe::run_native(
            "Wootili-View",
            eframe::NativeOptions {
                centered: true,
                ..Default::default()
            },
            Box::new(move |_cc| Ok(Box::<MyApp>::default())),
        )?;
    }

    Ok(())
}

struct MyApp {
    toasts: Toasts,
    is_startup: bool,
    plugins: Vec<Plugin>,
    device_name: String,
    brightness: u8,
    reduce_bright_effects: bool,
    screen: usize,
    display_rgb_preview: bool,
    downscale_method: FilterType,
    frame_limit: (u8, u8),
    red_shift_fix: bool,
    highlight_wasd: bool,
    dark_mode: bool,
    check_updates: bool,
    device_creation: String,
    device_version: String,
    rgb_size: (u32, u32),
    next_frame: Duration,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            toasts: Toasts::default(),
            is_startup: true,
            plugins: get_available_plugins(),
            device_name: wooting::get_device_name(),
            brightness: 100,
            reduce_bright_effects: false,
            screen: 0,
            display_rgb_preview: true,
            downscale_method: FilterType::Triangle,
            frame_limit: (60, 15), // (UI, Capture)
            red_shift_fix: false,
            highlight_wasd: false,
            dark_mode: true,
            check_updates: true,
            device_creation: wooting::get_device_creation(0),
            device_version: wooting::get_device_version(),
            rgb_size: wooting::get_rgb_size().unwrap(),
            next_frame: Duration::from_secs(0),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.is_startup {
            if !cfg!(windows) {
                self.toasts
                    .error("This application is not supported on your operating system")
                    .duration(Some(Duration::from_secs(120)));
            }
            logf!(Info, "Connected to device Name: {}", self.device_name);
            match self.device_name.as_str() {
                "N/A" => {
                    self.toasts
                        .error("No Wooting Device Found")
                        .duration(Some(Duration::from_secs(5)));
                }
                _ => {
                    self.toasts
                        .success(format!("Connected to {}", self.device_name))
                        .duration(Some(Duration::from_secs(3)));
                }
            };

            let config = match read_config() {
                Some(config) => config,
                None => {
                    reset_config();

                    self.toasts
                        .warning("Config file has been reset due to a config format error")
                        .duration(Some(Duration::from_secs(5)));

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
            self.highlight_wasd = config.highlight_wasd;
            self.dark_mode = config.dark_mode;
            self.check_updates = config.check_updates;

            *CAPTURE_SETTINGS.write().unwrap() = CaptureSettings {
                screen_index: self.screen,
                downscale_method: self.downscale_method,
                capture_frame_limit: self.frame_limit.1.into(),
                reduce_bright_effects: self.reduce_bright_effects,
                red_shift_fix: self.red_shift_fix,
                highlight_wasd: self.highlight_wasd,
                brightness: self.brightness,
                device_name: self.device_name.clone(),
                rgb_size: wooting::get_rgb_size().unwrap_or((0, 0)),
                display_rgb_preview: self.display_rgb_preview,
            };

            CAPTURE_SETTINGS_RELOAD.store(true, Ordering::Relaxed);
            CAPTURE_LOCK.store(false, Ordering::Relaxed);

            if self.dark_mode {
                ctx.set_visuals(egui::Visuals::dark());
            } else {
                ctx.set_visuals(egui::Visuals::light());
            }

            self.is_startup = false;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Settings");
            ui.separator();

            ui.heading("Visual");
            if ui.add(egui::Slider::new(&mut self.brightness, 50..=150).text("Brightness")).on_hover_text("Adjusts the brightness of the lighting").changed() {
                save_config_option(ConfigChange::Brightness(self.brightness), &mut self.toasts);
                CAPTURE_SETTINGS.write().unwrap().brightness = self.brightness;
                CAPTURE_SETTINGS_RELOAD.store(true, Ordering::Relaxed);
            }
            if ui.add(egui::Slider::new(&mut self.screen, 0..=Monitor::all().unwrap().len() - 1).text("Screen")).on_hover_text("Select the screen to capture").changed() {
                save_config_option(ConfigChange::Screen(self.screen), &mut self.toasts);
                CAPTURE_SETTINGS.write().unwrap().screen_index = self.screen;
                CAPTURE_SETTINGS_RELOAD.store(true, Ordering::Relaxed);
            }
            if ui.checkbox(&mut self.reduce_bright_effects, "Reduce Bright Effects").on_hover_text("Reduces brightness when the screen is very bright").changed() {
                save_config_option(ConfigChange::ReduceBrightEffects(self.reduce_bright_effects), &mut self.toasts);
                CAPTURE_SETTINGS.write().unwrap().reduce_bright_effects = self.reduce_bright_effects;
                CAPTURE_SETTINGS_RELOAD.store(true, Ordering::Relaxed);
            }
            if ui.checkbox(&mut self.red_shift_fix, "Red Shift Fix").on_hover_text("Fixes the red shift/hue issue on some Wooting keyboards due to the stock keycaps or from custom switches like the Geon Raptor HE").changed() {
                save_config_option(ConfigChange::RedShiftFix(self.red_shift_fix), &mut self.toasts);
                CAPTURE_SETTINGS.write().unwrap().red_shift_fix = self.red_shift_fix;
                CAPTURE_SETTINGS_RELOAD.store(true, Ordering::Relaxed);
            }
            if ui.checkbox(&mut self.highlight_wasd, "Highlight WASD").on_hover_text("Highlights the WASD keys to be able to see easily while gaming").changed() {
                save_config_option(ConfigChange::HighlightWASD(self.highlight_wasd), &mut self.toasts);
                CAPTURE_SETTINGS.write().unwrap().highlight_wasd = self.highlight_wasd;
                CAPTURE_SETTINGS_RELOAD.store(true, Ordering::Relaxed);
            }
            ui.menu_button("Downscale Method", |ui| {
                downscale_label(ui, &mut self.downscale_method, FilterType::Nearest, "Nearest", "Fast and picks on up on small details but is inconsistent, can completly mask elements on screen", &mut self.toasts);
                downscale_label(ui, &mut self.downscale_method, FilterType::Triangle, "Triangle", "Overall good results and is fast, best speed to quality ratio (Default)", &mut self.toasts);
                downscale_label(ui, &mut self.downscale_method, FilterType::Gaussian, "Gaussian", "Fast but gives poor results", &mut self.toasts);
                downscale_label(ui, &mut self.downscale_method, FilterType::CatmullRom, "CatmullRom", "Good results but is slow, similar results to Lanczos3", &mut self.toasts);
                downscale_label(ui, &mut self.downscale_method, FilterType::Lanczos3, "Lanczos3", "Gives the best results but is slowest", &mut self.toasts);
                ui.separator();
                ui.label("Note: The downscale methods are sorted in order by quality and performance, the default is triangle.");
            });
            ui.separator();

            ui.heading("Performance");
            if ui.add(egui::Slider::new(&mut self.frame_limit.0, 25..=144).text("UI FPS Cap")).on_hover_text("Limits the FPS of the UI, this will not effect the responsivness of your device's RGB but can help overall system performance").changed() {
                save_config_option(ConfigChange::FrameLimit(self.frame_limit), &mut self.toasts);
            }
            if ui.add(egui::Slider::new(&mut self.frame_limit.1, 1..=60).text("RGB FPS Cap")).on_hover_text("Limits the FPS of the RGB for rendering on the device, note it is likely that having this number super large will not result at the desired FPS due to the speed of the RGB lights on the device").changed() {
                save_config_option(ConfigChange::FrameLimit(self.frame_limit), &mut self.toasts);
                CAPTURE_SETTINGS.write().unwrap().capture_frame_limit = self.frame_limit.1.into();
                CAPTURE_SETTINGS_RELOAD.store(true, Ordering::Relaxed);
            }

            let frame_rgb_size = self.rgb_size;

            let allow_preview = frame_rgb_size.0 != 0 && frame_rgb_size.1 != 0;
            if ui.add_enabled(allow_preview, egui::Checkbox::new(&mut self.display_rgb_preview, "Display RGB Preview")).on_hover_text("Displays a preview of the lighting, this can be disabled to improve performance").changed() {
                save_config_option(ConfigChange::DisplayRgbPreview(self.display_rgb_preview), &mut self.toasts);
            }
            ui.separator();

            ui.heading("Application");
            ui.horizontal(|ui| {
                ui.label("Loaded Plugins:").on_hover_ui(|ui| {
                    ui.horizontal(|ui| {
                        ui.label("Plugins: ");
                        for plugin in self.plugins.iter() {
                            ui.label(plugin.name.clone());
                        }
                    });
                });
                ui.label(format!("{}", self.plugins.len()));
            });

            if ui.checkbox(&mut self.dark_mode, "Darkmode").on_hover_text("Enables darkmode").changed() {
                save_config_option(ConfigChange::Darkmode(self.dark_mode), &mut self.toasts);

                if self.dark_mode {
                    ctx.set_visuals(egui::Visuals::dark());
                } else {
                    ctx.set_visuals(egui::Visuals::light());
                }
            }

            if let Some(_) = self
                .plugins
                .iter()
                .find(|plugin| plugin.name == "update_check")
            {
                if ui.checkbox(&mut self.check_updates, "Check for Updates").on_hover_text("Checks for updates on startup, wont apply to current session").changed() {
                    save_config_option(ConfigChange::CheckUpdates(self.check_updates), &mut self.toasts);
                }
            }

            if ui.button("Reset Config").on_hover_text("Warning: Resets the config to the default values").clicked() {
                reset_config();
                self.toasts
                    .info("Config file has been reset")
                    .duration(Some(Duration::from_secs(1)));

                let new_config = read_config().unwrap();

                self.brightness = new_config.brightness;
                self.reduce_bright_effects = new_config.reduce_bright_effects;
                self.screen = new_config.screen;
                self.display_rgb_preview = new_config.display_rgb_preview;
                self.downscale_method = downscale_index_to_filter(new_config.downscale_method_index);
                self.frame_limit = new_config.frame_limit;
                self.red_shift_fix = new_config.red_shift_fix;
                self.highlight_wasd = new_config.highlight_wasd;
                self.dark_mode = new_config.dark_mode;
                self.check_updates = new_config.check_updates;

                if self.dark_mode {
                    ctx.set_visuals(egui::Visuals::dark());
                } else {
                    ctx.set_visuals(egui::Visuals::light());
                }

                save_config_option(
                    ConfigChange::AllConfigOptions(new_config),
                    &mut self.toasts,
                );

                *CAPTURE_SETTINGS.write().unwrap() = CaptureSettings {
                    screen_index: self.screen,
                    downscale_method: self.downscale_method,
                    capture_frame_limit: self.frame_limit.1.into(),
                    reduce_bright_effects: self.reduce_bright_effects,
                    red_shift_fix: self.red_shift_fix,
                    highlight_wasd: self.highlight_wasd,
                    brightness: self.brightness,
                    device_name: self.device_name.clone(),
                    rgb_size: wooting::get_rgb_size().unwrap_or((0, 0)),
                    display_rgb_preview: self.display_rgb_preview,
                };

                CAPTURE_SETTINGS_RELOAD.store(true, Ordering::Relaxed);
            }

            clean_logs_button(ui, &mut self.toasts);

            // Rewriten plugin rendering
            //write code that looks threw self.plugins for a plugin struct with the name being file_import and then pass the lib to import_file_ui
            if let Some(plugin) = self
                .plugins
                .iter()
                .find(|plugin| plugin.name == "update_check")
            {
                egui::TopBottomPanel::bottom("footer").show(ctx, |ui| {
                    update_check_ui(plugin.lib.clone(), ui, utils::paths::logging_path().to_string_lossy().to_string());
                });
            }

            if !self.is_startup {
                egui::SidePanel::right("lighting_preview_panel").width_range(Rangef::new((self.rgb_size.0 * 15) as f32, (self.rgb_size.0 * 22) as f32)).show(ctx, |ui| {
                    if self.display_rgb_preview {
                        match CAPTURE_PREVIEW.read().unwrap().clone() {
                            Some(preview) => {
                                rgb_preview(ui, frame_rgb_size, preview.clone());
                            }
                            None => {
                                ui.heading("No Preview Available");
                            }
                        }
                        //rgb_preview(ui, frame_rgb_size, CAPTURE_PREVIEW.read().unwrap().clone());
                    }
                    display_device_info(ui, &mut self.toasts, &mut self.device_name, &mut self.device_creation, &mut self.device_version, &mut self.is_startup, frame_rgb_size);
                });
            }
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
                ConfigChange::HighlightWASD(self.highlight_wasd),
                ConfigChange::Darkmode(self.dark_mode),
                ConfigChange::CheckUpdates(self.check_updates),
            ]),
            &mut self.toasts,
        );
        wooting::exit_rgb();
    }
}
