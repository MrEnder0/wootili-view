#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod paths;
mod ui;
mod wooting;

use config::*;
use eframe::egui;
use egui_notify::Toasts;
use image::{imageops::FilterType, DynamicImage, GenericImageView};
use lazy_static::lazy_static;
use scorched::{logf, LogData, LogImportance};
use screenshots::Screen;
use std::sync::RwLock;
use ui::*;

// Statics for screen thread
lazy_static! {
    static ref SCREEN: RwLock<DynamicImage> = RwLock::new({
        let img = image::ImageBuffer::new(1, 1);
        image::DynamicImage::ImageRgba8(img)
    });
}
static RGB_SIZE: RwLock<(u32, u32)> = RwLock::new((0, 0));
static SCREEN_INDEX: RwLock<usize> = RwLock::new(0);
static DOWNSCALE_METHOD: RwLock<FilterType> = RwLock::new(FilterType::Triangle);
static FRAME_SLEEP: RwLock<u64> = RwLock::new(10);

fn main() -> Result<(), eframe::Error> {
    scorched::set_logging_path(format!("{}/", paths::logging_path().as_path().display()).as_str());

    if !config_exists() {
        gen_config();
    }

    wooting::update_rgb();

    // Screen thread, captures the screen and stores it in the static SCREEN
    std::thread::spawn(|| loop {
        let frame_rgb_size = *RGB_SIZE.read().unwrap();

        let screens = Screen::all().unwrap();
        let capture = screens[*SCREEN_INDEX.read().unwrap()].capture().unwrap();

        let img = image::DynamicImage::ImageRgba8(
            image::ImageBuffer::from_raw(capture.width(), capture.height(), capture.to_vec())
                .unwrap(),
        );
        let resized_capture = img.resize_exact(
            frame_rgb_size.0,
            frame_rgb_size.1,
            *DOWNSCALE_METHOD.read().unwrap(),
        );

        SCREEN.write().unwrap().clone_from(&resized_capture);

        std::thread::sleep(std::time::Duration::from_millis(
            *FRAME_SLEEP.read().unwrap(),
        ));
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
    device_creation: String,
    brightness: u8,
    reduce_bright_effects: bool,
    current_frame_reduce: bool,
    screen: usize,
    display_rgb_preview: bool,
    downscale_method: FilterType,
    frame_sleep: (u8, u8),
    red_shift_fix: bool,
    dark_mode: bool,
    check_updates: bool,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            toasts: Toasts::default(),
            init: true,
            device_name: wooting::get_device_name(),
            device_creation: wooting::get_device_creation(),
            brightness: 100,
            reduce_bright_effects: false,
            current_frame_reduce: false,
            screen: 0,
            display_rgb_preview: true,
            downscale_method: FilterType::Triangle,
            frame_sleep: (15, 10), // (UI, Capture)
            red_shift_fix: false,
            dark_mode: true,
            check_updates: true,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.init {
            if !cfg!(windows) {
                self.toasts
                    .error("This application is not supported on your operating system")
                    .set_duration(Some(std::time::Duration::from_secs(120)));
            }
            logf!(Info, "Connected to device Name: {}", self.device_name);
            match self.device_name.as_str() {
                "N/A" => {
                    self.toasts
                        .error("No Wooting Device Found")
                        .set_duration(Some(std::time::Duration::from_secs(5)));
                }
                _ => {
                    self.toasts
                        .success(format!("Connected to {}", self.device_name))
                        .set_duration(Some(std::time::Duration::from_secs(3)));
                }
            };

            let config = match read_config() {
                Some(config) => config,
                None => {
                    reset_config();

                    self.toasts
                        .warning("Config file has been reset due to a config format error")
                        .set_duration(Some(std::time::Duration::from_secs(5)));

                    read_config().unwrap()
                }
            };

            self.brightness = config.brightness;
            self.reduce_bright_effects = config.reduce_bright_effects;
            self.screen = config.screen;
            self.display_rgb_preview = config.display_rgb_preview;
            self.downscale_method = downscale_index_to_filter(config.downscale_method_index);
            self.frame_sleep = config.frame_sleep;
            self.red_shift_fix = config.red_shift_fix;
            self.dark_mode = config.dark_mode;
            self.check_updates = config.check_updates;

            if self.dark_mode {
                ctx.set_visuals(egui::Visuals::dark());
            } else {
                ctx.set_visuals(egui::Visuals::light());
            }

            self.init = false;
        }

        let frame_rgb_size = *RGB_SIZE.read().unwrap();
        let mut resized_capture = image::DynamicImage::new_rgba8(1, 1);

        if frame_rgb_size.0 != 0 && frame_rgb_size.1 != 0 {
            resized_capture = SCREEN.read().unwrap().clone();

            if self.reduce_bright_effects {
                let avg_screen =
                    resized_capture
                        .clone()
                        .resize(1, 1, image::imageops::FilterType::Gaussian);

                let image::Rgba([r, g, b, _]) = avg_screen.get_pixel(0, 0);

                if r > 220 || g > 220 || b > 220 {
                    self.current_frame_reduce = true;
                    self.brightness -= 50;
                }
            }

            wooting::draw_rgb(
                resized_capture.clone(),
                self.brightness,
                self.red_shift_fix,
                self.device_name.clone(),
            );

            if self.current_frame_reduce {
                self.brightness += 50;
                self.current_frame_reduce = false;
            }
        } else {
            wooting::reconnect_device();
            self.device_name = wooting::get_device_name();
            self.device_creation = wooting::get_device_creation();
            RGB_SIZE
                .write()
                .unwrap()
                .clone_from(&wooting::get_rgb_size());
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Settings");
            ui.separator();

            ui.heading("Visual");
            if ui.add(egui::Slider::new(&mut self.brightness, 50..=150).text("Brightness")).on_hover_text("Adjusts the brightness of the lighting").changed() {
                save_config_option(ConfigChange::Brightness(self.brightness), &mut self.toasts);
            }
            if ui.add(egui::Slider::new(&mut self.screen, 0..=Screen::all().unwrap().len() - 1).text("Screen")).on_hover_text("Select the screen to capture").changed() {
                save_config_option(ConfigChange::Screen(self.screen), &mut self.toasts);
                *SCREEN_INDEX.write().unwrap() = self.screen;
            }
            if ui.checkbox(&mut self.reduce_bright_effects, "Reduce Bright Effects").on_hover_text("Reduces brightness when the screen is very bright").changed() {
                save_config_option(ConfigChange::ReduceBrightEffects(self.reduce_bright_effects), &mut self.toasts);
            }
            if ui.checkbox(&mut self.red_shift_fix, "Red Shift Fix").on_hover_text("Fixes the red shift/hue issue on some Wooting keyboards due to the stock keycaps or from custom switches like the Geon Raptor HE").changed() {
                save_config_option(ConfigChange::RedShiftFix(self.red_shift_fix), &mut self.toasts);
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
            if ui.add(egui::Slider::new(&mut self.frame_sleep.0, 0..=100).text("UI Frame Sleep (ms)")).on_hover_text("Waits the specified amount of time before updating the ui").changed() {
                save_config_option(ConfigChange::FrameSleep(self.frame_sleep), &mut self.toasts);
            }
            if ui.add(egui::Slider::new(&mut self.frame_sleep.1, 0..=100).text("Capture Frame Sleep (ms)")).on_hover_text("Waits the specified amount of time before capturing the screen").changed() {
                save_config_option(ConfigChange::FrameSleep(self.frame_sleep), &mut self.toasts);
                *FRAME_SLEEP.write().unwrap() = self.frame_sleep.1.into();
            }

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
                    .set_duration(Some(std::time::Duration::from_secs(1)));

                let new_config = read_config().unwrap();

                self.brightness = new_config.brightness;
                self.reduce_bright_effects = new_config.reduce_bright_effects;
                self.screen = new_config.screen;
                self.display_rgb_preview = new_config.display_rgb_preview;
                self.downscale_method = downscale_index_to_filter(new_config.downscale_method_index);
                self.frame_sleep = new_config.frame_sleep;
                self.red_shift_fix = new_config.red_shift_fix;
                self.dark_mode = new_config.dark_mode;
                self.check_updates = new_config.check_updates;

                if self.dark_mode {
                    ctx.set_visuals(egui::Visuals::dark());
                } else {
                    ctx.set_visuals(egui::Visuals::light());
                }
            }

            egui::TopBottomPanel::bottom("footer").show(ctx, |ui| {
                version_footer(ui, self.check_updates);
            });

            egui::SidePanel::right("lighting_preview_panel").show(ctx, |ui| {
                if self.display_rgb_preview {
                    rgb_preview(ui, frame_rgb_size, resized_capture);
                }
                display_device_info(ui, &mut self.toasts, &mut self.device_name, &mut self.device_creation, &mut self.init);
                display_lighting_dimensions(ui, frame_rgb_size);
            });
        });

        self.toasts.show(ctx);

        std::thread::sleep(std::time::Duration::from_millis(self.frame_sleep.0.into()));
        ctx.request_repaint()
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        save_config_option(ConfigChange::Brightness(self.brightness), &mut self.toasts);
        save_config_option(
            ConfigChange::ReduceBrightEffects(self.reduce_bright_effects),
            &mut self.toasts,
        );
        save_config_option(ConfigChange::Screen(self.screen), &mut self.toasts);
        save_config_option(
            ConfigChange::DisplayRgbPreview(self.display_rgb_preview),
            &mut self.toasts,
        );
        save_config_option(
            ConfigChange::DownscaleMethod(self.downscale_method),
            &mut self.toasts,
        );
        save_config_option(ConfigChange::FrameSleep(self.frame_sleep), &mut self.toasts);
        save_config_option(
            ConfigChange::RedShiftFix(self.red_shift_fix),
            &mut self.toasts,
        );
        save_config_option(ConfigChange::Darkmode(self.dark_mode), &mut self.toasts);
        save_config_option(
            ConfigChange::CheckUpdates(self.check_updates),
            &mut self.toasts,
        );
        wooting::exit_rgb();
    }
}
