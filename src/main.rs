#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use egui_notify::Toasts;
use image::{imageops::FilterType, DynamicImage, GenericImageView};
use once_cell::sync::Lazy;
use screenshots::Screen;
use std::{ffi::CStr, sync::Mutex};
use wooting_rgb_sys as wooting;

// Statics for screen thread
static SCREEN: Mutex<Lazy<DynamicImage>> = Mutex::new(Lazy::new(|| {
    let img = image::ImageBuffer::new(1, 1);
    image::DynamicImage::ImageRgba8(img)
}));
static SCREEN_INDEX: Mutex<usize> = Mutex::new(0);
static DOWNSCALE_METHOD: Mutex<FilterType> = Mutex::new(FilterType::Triangle);
static FRAME_SLEEP: Mutex<u64> = Mutex::new(10);

fn main() -> Result<(), eframe::Error> {
    // Run to reset rgb
    unsafe {
        wooting::wooting_rgb_array_update_keyboard();
    }

    // Screen thread, captures the screen and stores it in the static SCREEN
    std::thread::spawn(|| loop {
        let screens = Screen::all().unwrap();
        let capture = screens[*SCREEN_INDEX.lock().unwrap()].capture().unwrap();

        let img = image::ImageBuffer::from_raw(capture.width(), capture.height(), capture.to_vec())
            .unwrap();
        let img = image::DynamicImage::ImageRgba8(img);
        let resized_capture = img.resize_exact(21, 6, *DOWNSCALE_METHOD.lock().unwrap());

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
    device_name: String,
    rgb_size: (u32, u32),
    brightness: u8,
    reduce_bright_effects: bool,
    current_frame_reduce: bool,
    screen: usize,
    display_rgb_preview: bool,
    downscale_method: FilterType,
    frame_sleep: u64,
    toasts: Toasts,
    init: bool,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            device_name: unsafe {
                wooting::wooting_usb_disconnect(false);
                wooting::wooting_usb_find_keyboard();

                let wooting_usb_meta = *wooting::wooting_usb_get_meta();
                let model = CStr::from_ptr(wooting_usb_meta.model);

                model.to_str().unwrap().to_string()
            },
            rgb_size: unsafe {
                wooting::wooting_usb_disconnect(false);
                wooting::wooting_usb_find_keyboard();

                let wooting_usb_meta = *wooting::wooting_usb_get_meta();
                let model = CStr::from_ptr(wooting_usb_meta.model);

                match model.to_str().unwrap() {
                    //TODO: Verify these sizes for the one two and uwu
                    "Wooting One" => (17, 6),
                    "Wooting Two"
                    | "Wooting Two LE"
                    | "Wooting Two HE"
                    | "Wooting Two HE (ARM)" => (21, 6),
                    "Wooting 60HE" | "Wooting 60HE (ARM)" => (14, 5),
                    "Wooting UwU" | "Wooting UwU RGB" => (3, 1),
                    _ => {
                        println!("Unsupported keyboard model: {}", model.to_str().unwrap());
                        (0, 0)
                    }
                }
            },
            brightness: 100,
            reduce_bright_effects: false,
            current_frame_reduce: false,
            screen: 0,
            display_rgb_preview: true,
            downscale_method: FilterType::Triangle,
            frame_sleep: 10,
            toasts: Toasts::default(),
            init: true,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.init {
            self.toasts
                .success(format!("Connected to {}", self.device_name))
                .set_duration(Some(std::time::Duration::from_secs(2)));
            self.init = false;
        }

        let resized_capture = SCREEN.lock().unwrap().clone();

        // Runs lighting operations
        unsafe {
            for (x, y, pixel) in resized_capture.pixels() {
                let image::Rgba([r, g, b, _]) = pixel;
                wooting::wooting_rgb_array_set_single(
                    y as u8 + 1,
                    x as u8,
                    (r as f32 * (self.brightness as f32 * 0.01)).round() as u8,
                    (g as f32 * (self.brightness as f32 * 0.01)).round() as u8,
                    (b as f32 * (self.brightness as f32 * 0.01)).round() as u8,
                );
            }

            wooting::wooting_rgb_array_update_keyboard();
        }

        if self.current_frame_reduce {
            self.brightness += 50;
            self.current_frame_reduce = false;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Settings");
            ui.separator();

            ui.heading("Visual");
            ui.add(egui::Slider::new(&mut self.brightness, 50..=150).text("Brightness")).on_disabled_hover_text("Adjust the brightness of the lighting");
            if ui.add(egui::Slider::new(&mut self.screen, 0..=Screen::all().unwrap().len() - 1).text("Screen")).on_disabled_hover_text("Select the screen to capture").changed() {
                *SCREEN_INDEX.lock().unwrap() = self.screen;
            }
            ui.checkbox(&mut self.reduce_bright_effects, "Reduce Bright Effects").on_disabled_hover_text("Reduces brightness when the screen is very bright");
            ui.menu_button("Downscale Method", |ui| {
                if ui.add(egui::SelectableLabel::new(
                    self.downscale_method == FilterType::Nearest,
                    "Nearest",
                )).on_disabled_hover_text("Fast and picks on up on small details but is inconsistent").clicked() {
                    DOWNSCALE_METHOD.lock().unwrap().clone_from(&FilterType::Nearest);
                    self.downscale_method = FilterType::Nearest;
                }
                if ui.add(egui::SelectableLabel::new(
                    self.downscale_method == FilterType::Triangle,
                    "Triangle",
                )).on_disabled_hover_text("Overall good results and is fast, best speed to quality ratio").clicked() {
                    DOWNSCALE_METHOD.lock().unwrap().clone_from(&FilterType::Triangle);
                    self.downscale_method = FilterType::Triangle;
                }
                if ui.add(egui::SelectableLabel::new(
                    self.downscale_method == FilterType::Gaussian,
                    "Gaussian",
                )).on_disabled_hover_text("Has a softer look with its blur effect, looks nice").clicked() {
                    DOWNSCALE_METHOD.lock().unwrap().clone_from(&FilterType::Gaussian);
                    self.downscale_method = FilterType::Gaussian;
                }
                if ui.add(egui::SelectableLabel::new(
                    self.downscale_method == FilterType::CatmullRom,
                    "CatmullRom",
                )).on_disabled_hover_text("Good results but is slow, similar results to Lanczos3").clicked() {
                    DOWNSCALE_METHOD.lock().unwrap().clone_from(&FilterType::CatmullRom);
                    self.downscale_method = FilterType::CatmullRom;
                }
                if ui.add(egui::SelectableLabel::new(
                    self.downscale_method == FilterType::Lanczos3,
                    "Lanczos3",
                )).on_disabled_hover_text("Gives the best results but is very slow").clicked() {
                    DOWNSCALE_METHOD.lock().unwrap().clone_from(&FilterType::Lanczos3);
                    self.downscale_method = FilterType::Lanczos3;
                }
            });
            ui.separator();

            ui.heading("Performance");
            if ui.add(egui::Slider::new(&mut self.frame_sleep, 0..=100).text("Frame Sleep (ms)")).changed() {
                *FRAME_SLEEP.lock().unwrap() = self.frame_sleep;
            }
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.display_rgb_preview, "Display RGB Preview").on_disabled_hover_text("Displays a preview of the lighting, this can be disabled to improve performance");
            });

            egui::TopBottomPanel::bottom("footer").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // Add your footer content here
                    ui.label(format!(
                        "Wootili-View {} by Mr.Ender",
                        env!("CARGO_PKG_VERSION")
                    ));
                });
            });

            egui::SidePanel::right("lighting_preview_panel").show(ctx, |ui| {
                if self.display_rgb_preview {
                    ui.heading("Preview Lighting");
                    for y in 0..self.rgb_size.1 {
                        ui.horizontal(|ui| {
                            for x in 0..self.rgb_size.0 {
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
                ui.add(egui::Label::new(format!("Device: {}", self.device_name,)));
                ui.add(egui::Label::new(format!(
                    "Lighting Dimentions: {}x{}",
                    self.rgb_size.0, self.rgb_size.1
                )));
            });
        });

        self.toasts.show(ctx);

        std::thread::sleep(std::time::Duration::from_millis(10));
        ctx.request_repaint()
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        // Runs to set lighting back to normal
        unsafe {
            wooting::wooting_rgb_close();
        }
    }
}
