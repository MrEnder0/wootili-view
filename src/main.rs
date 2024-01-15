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
    reduce_bright_effects: bool,
    current_frame_reduce: bool,
    screen: usize,
    display_rgb_preview: bool,
    downscale_method: FilterType,
    frame_sleep: u64,
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
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        //TODO: Find way to run this seprately from the main loop due to this being the bulk of the cpu usage
        let screens = Screen::all().unwrap();
        let capture = screens[self.screen].capture().unwrap();

        let img = image::ImageBuffer::from_raw(capture.width(), capture.height(), capture.to_vec())
            .unwrap();
        let img = image::DynamicImage::ImageRgba8(img);
        let resized_capture =
            img.resize_exact(self.rgb_size.0, self.rgb_size.1, self.downscale_method);

        if self.reduce_bright_effects {
            let avg_screen = img.resize(
                1,
                1,
                image::imageops::FilterType::Triangle,
            );

            let image::Rgba([r, g, b, _]) = avg_screen.get_pixel(0, 0);

            if r > 220 || g > 220 || b > 220 {
                self.current_frame_reduce = true;
                self.brightness -= 50;
            }
        }

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
            ui.add(egui::Slider::new(&mut self.screen, 0..=screens.len() - 1).text("Screen")).on_disabled_hover_text("Select the screen to capture");
            ui.checkbox(&mut self.reduce_bright_effects, "Reduce Bright Effects").on_disabled_hover_text("Reduces brightness when the screen is very bright");
            ui.menu_button("Downscale Method", |ui| {
                ui.selectable_value(&mut self.downscale_method, FilterType::Nearest, "Nearest")
                    .on_hover_text("Fast and picks on up on small details but is inconsistent");
                ui.selectable_value(&mut self.downscale_method, FilterType::Triangle, "Triangle")
                    .on_hover_text("Overall good results and is fast, best speed to quality ratio");
                ui.selectable_value(&mut self.downscale_method, FilterType::Gaussian, "Gaussian")
                    .on_hover_text("Has a softer look with its blur effect, looks nice");
                ui.selectable_value(
                    &mut self.downscale_method,
                    FilterType::CatmullRom,
                    "CatmullRom",
                )
                .on_hover_text("Good results but is slow, similar results to Lanczos3");
                ui.selectable_value(&mut self.downscale_method, FilterType::Lanczos3, "Lanczos3")
                    .on_hover_text("Gives the best results but is very slow");
            });
            ui.separator();

            ui.heading("Performance");
            ui.add(egui::Slider::new(&mut self.frame_sleep, 0..=100).text("Frame Sleep (ms)"));
            ui.horizontal(|ui| {
                ui.label("Display RGB Preview");
                ui.checkbox(&mut self.display_rgb_preview, "");
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

        std::thread::sleep(std::time::Duration::from_millis(self.frame_sleep));
        ctx.request_repaint()
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        // Runs to set lighting back to normal
        unsafe {
            wooting::wooting_rgb_close();
        }
    }
}
