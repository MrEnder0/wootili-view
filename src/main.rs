#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use image::{imageops::FilterType, GenericImageView};
use screenshots::Screen;
use wooting_rgb_sys as wooting;

fn main() {
    unsafe {
        wooting::wooting_rgb_array_update_keyboard();

        loop {
            let screens = Screen::all().unwrap();
            let capture = screens[0].capture().unwrap();
            capture.save("temp.png").unwrap();

            let img = image::open("temp.png").unwrap();
            let resized_capture = img.resize_exact(14, 6, FilterType::Nearest);

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
