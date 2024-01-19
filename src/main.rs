#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ui;
mod wooting;

use eframe::egui;
use egui_notify::Toasts;
use image::{imageops::FilterType, DynamicImage, GenericImageView};
use once_cell::sync::Lazy;
use screenshots::Screen;
use std::sync::Mutex;
use ui::downscale_label;

// Statics for screen thread
static SCREEN: Mutex<Lazy<DynamicImage>> = Mutex::new(Lazy::new(|| {
    let img = image::ImageBuffer::new(1, 1);
    image::DynamicImage::ImageRgba8(img)
}));
static SCREEN_INDEX: Mutex<usize> = Mutex::new(0);
static DOWNSCALE_METHOD: Mutex<FilterType> = Mutex::new(FilterType::Triangle);
static FRAME_SLEEP: Mutex<u64> = Mutex::new(10);
static RGB_SIZE: Lazy<(u32, u32)> = Lazy::new(wooting::get_rgb_size);

fn main() -> Result<(), eframe::Error> {
    wooting::update_rgb();

    // Screen thread, captures the screen and stores it in the static SCREEN
    std::thread::spawn(|| loop {
        let screens = Screen::all().unwrap();
        let capture = screens[*SCREEN_INDEX.lock().unwrap()].capture().unwrap();

        let img = image::ImageBuffer::from_raw(capture.width(), capture.height(), capture.to_vec())
            .unwrap();
        let img = image::DynamicImage::ImageRgba8(img);
        let resized_capture =
            img.resize_exact(RGB_SIZE.0, RGB_SIZE.1, *DOWNSCALE_METHOD.lock().unwrap());

        SCREEN.lock().unwrap().clone_from(&resized_capture);

        std::thread::sleep(std::time::Duration::from_millis(
            *FRAME_SLEEP.lock().unwrap(),
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
    frame_sleep: u64,
    red_shift_fix: bool,
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
            frame_sleep: 10,
            red_shift_fix: false,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.init {
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

            self.init = false;
        }

        let resized_capture = SCREEN.lock().unwrap().clone();

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

        wooting::draw_rgb(resized_capture.clone(), self.brightness, self.red_shift_fix);

        if self.current_frame_reduce {
            self.brightness += 50;
            self.current_frame_reduce = false;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Settings");
            ui.separator();

            ui.heading("Visual");
            ui.add(egui::Slider::new(&mut self.brightness, 50..=150).text("Brightness")).on_hover_text("Adjusts the brightness of the lighting");
            if ui.add(egui::Slider::new(&mut self.screen, 0..=Screen::all().unwrap().len() - 1).text("Screen")).on_hover_text("Select the screen to capture").changed() {
                *SCREEN_INDEX.lock().unwrap() = self.screen;
            }
            ui.checkbox(&mut self.reduce_bright_effects, "Reduce Bright Effects").on_hover_text("Reduces brightness when the screen is very bright");
            ui.checkbox(&mut self.red_shift_fix, "Red Shift Fix").on_hover_text("Fixes the red shift/hue issue on some Wooting keyboards due to the stock keycaps or from custom switches like the Geon Raptor HE");
            ui.menu_button("Downscale Method", |ui| {
                downscale_label(ui, &mut self.downscale_method, FilterType::Nearest, "Nearest", "Fast and picks on up on small details but is inconsistent");
                downscale_label(ui, &mut self.downscale_method, FilterType::Triangle, "Triangle", "Overall good results and is fast, best speed to quality ratio");
                downscale_label(ui, &mut self.downscale_method, FilterType::Gaussian, "Gaussian", "Fast but gives poor results");
                downscale_label(ui, &mut self.downscale_method, FilterType::CatmullRom, "CatmullRom", "Good results but is slow, similar results to Lanczos3");
                downscale_label(ui, &mut self.downscale_method, FilterType::Lanczos3, "Lanczos3", "Gives the best results but is very slow");
            });
            ui.separator();

            ui.heading("Performance");
            if ui.add(egui::Slider::new(&mut self.frame_sleep, 0..=100).text("Frame Sleep (ms)")).on_hover_text("Waits the specified amount of time before recapturing a new frame").changed() {
                *FRAME_SLEEP.lock().unwrap() = self.frame_sleep;
            }
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.display_rgb_preview, "Display RGB Preview").on_hover_text("Displays a preview of the lighting, this can be disabled to improve performance");
            });

            egui::TopBottomPanel::bottom("footer").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.hyperlink_to(format!(
                        "Wootili-View {} by Mr.Ender",
                        env!("CARGO_PKG_VERSION")
                    ), "https://github.com/MrEnder0/Wootili-View");
                });
            });

            egui::SidePanel::right("lighting_preview_panel").show(ctx, |ui| {
                if self.display_rgb_preview {
                    ui.heading("Preview Lighting");
                    for y in 0..RGB_SIZE.1 {
                        ui.horizontal(|ui| {
                            for x in 0..RGB_SIZE.0 {
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

                ui.heading("Device Info");
                ui.add(egui::Label::new(format!("Name: {}", self.device_name,)));
                ui.label(format!("Creation: {}", self.device_creation));

                let lighting_dimensions = if RGB_SIZE.0 == 0 && RGB_SIZE.1 == 0 {
                    "Unknown".to_string()
                } else {
                    format!("{}x{}", RGB_SIZE.0, RGB_SIZE.1)
                };
                ui.add(egui::Label::new(format!(
                    "Lighting Dimensions: {}",
                    lighting_dimensions
                )));
            });
        });

        self.toasts.show(ctx);

        std::thread::sleep(std::time::Duration::from_millis(10));
        ctx.request_repaint()
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        wooting::exit_rgb();
    }
}
