#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use image::{imageops::FilterType, GenericImageView};
use screenshots::Screen;
use std::ffi::CStr;
use wooting_rgb_sys as wooting;

fn main() -> Result<(), eframe::Error> {
    // Run to reset rgb
    unsafe {
        wooting::wooting_rgb_array_update_keyboard();
    }

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
    screen: usize,
    display_rgb_preview: bool,
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
            screen: 0,
            display_rgb_preview: true,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let screens = Screen::all().unwrap();
        let capture = screens[self.screen].capture().unwrap();

        let img = image::ImageBuffer::from_raw(capture.width(), capture.height(), capture.to_vec())
            .unwrap();
        let img = image::DynamicImage::ImageRgba8(img);
        let resized_capture =
            img.resize_exact(self.rgb_size.0, self.rgb_size.1, FilterType::Nearest);

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

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Settings");
            ui.separator();

            ui.heading("Visual");
            ui.add(egui::Slider::new(&mut self.brightness, 50..=150).text("Brightness"));
            ui.add(egui::Slider::new(&mut self.screen, 0..=screens.len() - 1).text("Screen"));
            ui.separator();

            ui.heading("Performance");
            ui.horizontal(|ui| {
                ui.label("Display RGB Preview");
                ui.checkbox(&mut self.display_rgb_preview, "");
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
                }

                ui.heading("Keyboard Info");
                ui.add(egui::Label::new(format!("Device: {}", self.device_name,)));
                ui.add(egui::Label::new(format!(
                    "Lighting Dimentions: {}x{}",
                    self.rgb_size.0, self.rgb_size.1
                )));
            });
        });

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
