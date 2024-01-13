use image::{imageops::FilterType, GenericImageView};
use screenshots::Screen;
use std::ffi::CStr;
use wooting_rgb_sys as wooting;

fn main() {
    unsafe {
        wooting::wooting_usb_disconnect(false);
        wooting::wooting_usb_find_keyboard();

        let wooting_usb_meta = *wooting::wooting_usb_get_meta();
        let model = CStr::from_ptr(wooting_usb_meta.model);

        let rgb_size = match model.to_str().unwrap() {
            //TODO: Verify these sizes for the one two and uwu
            "Wooting One" => (17, 6),
            "Wooting Two" | "Wooting Two LE" | "Wooting Two HE" | "Wooting Two HE (ARM)" => (21, 6),
            "Wooting 60HE" | "Wooting 60HE (ARM)" => (14, 6),
            "Wooting UwU" | "Wooting UwU RGB" => (3, 1),
            _ => {
                println!("Unsupported keyboard model: {}", model.to_str().unwrap());
                return;
            }
        };

        wooting::wooting_rgb_array_update_keyboard();

        loop {
            let screens = Screen::all().unwrap();
            let capture = screens[0].capture().unwrap();
            capture.save("temp.png").unwrap();

            let img = image::open("temp.png").unwrap();
            let resized_capture = img.resize_exact(rgb_size.0, rgb_size.1, FilterType::Nearest);

            for (x, y, pixel) in resized_capture.pixels() {
                let image::Rgba([r, g, b, _]) = pixel;
                wooting::wooting_rgb_array_set_single(y as u8, x as u8, r, g, b);
            }

            wooting::wooting_rgb_array_update_keyboard();

            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        wooting::wooting_rgb_close();
    }
}
