use image::{imageops::FilterType, GenericImageView};
use screenshots::Screen;
use std::ffi::CStr;
use wooting_rgb_sys as wooting;
use eframe::egui;
use tray_icon::TrayIconBuilder;

#[cfg(not(target_os = "linux"))]
use std::{cell::RefCell, rc::Rc};

fn main() -> Result<(), eframe::Error> {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/media/icon.png");
    let icon = load_icon(std::path::Path::new(path));

    #[cfg(target_os = "linux")]
    std::thread::spawn(|| {
        use tray_icon::menu::Menu;

        gtk::init().unwrap();
        let _tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(Menu::new()))
            .with_icon(icon)
            .build()
            .unwrap();

        gtk::main();
    });

    #[cfg(not(target_os = "linux"))]
    let mut _tray_icon = Rc::new(RefCell::new(None));
    #[cfg(not(target_os = "linux"))]
    let tray_c = _tray_icon.clone();

    // Run to reset rgb
    unsafe {
        wooting::wooting_rgb_array_update_keyboard();
    }

    eframe::run_native(
        "Wootili-View",
        eframe::NativeOptions::default(),
        Box::new(move |_cc| {
            #[cfg(not(target_os = "linux"))]
            {
                tray_c
                    .borrow_mut()
                    .replace(TrayIconBuilder::new().with_icon(icon).build().unwrap());
            }
            Box::<MyApp>::default()
        }),
    )
}

struct MyApp {
    rgb_size: (u32, u32),
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
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
                    "Wooting 60HE" | "Wooting 60HE (ARM)" => (14, 6),
                    "Wooting UwU" | "Wooting UwU RGB" => (3, 1),
                    _ => {
                        println!("Unsupported keyboard model: {}", model.to_str().unwrap());
                        return Self { rgb_size: (0, 0) };
                    }
                }
            },
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        use tray_icon::TrayIconEvent;

        if let Ok(event) = TrayIconEvent::receiver().try_recv() {
            println!("tray event: {event:?}");
        }

        let screens = Screen::all().unwrap();
        let capture = screens[0].capture().unwrap();
        
        let img = image::ImageBuffer::from_raw(capture.width(), capture.height(), capture.to_vec()).unwrap();
        let img = image::DynamicImage::ImageRgba8(img);
        let resized_capture = img.resize_exact(self.rgb_size.0, self.rgb_size.1, FilterType::Nearest);

        resized_capture.save("preview.png").unwrap();

        // Runs lighting operations
        unsafe {
            for (x, y, pixel) in resized_capture.pixels() {
                let image::Rgba([r, g, b, _]) = pixel;
                wooting::wooting_rgb_array_set_single(y as u8, x as u8, r, g, b);
            }

            wooting::wooting_rgb_array_update_keyboard();
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("TODO: Add settings here");
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

fn load_icon(path: &std::path::Path) -> tray_icon::Icon {
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::open(path)
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    tray_icon::Icon::from_rgba(icon_rgba, icon_width, icon_height).expect("Failed to open icon")
}
