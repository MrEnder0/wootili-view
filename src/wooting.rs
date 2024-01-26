use std::ffi::CStr;

use image::GenericImageView;
use scorched::{logf, LogData, LogImportance};
use wooting_rgb_sys as wooting;

pub fn get_rgb_size() -> (u32, u32) {
    unsafe {
        wooting::wooting_usb_disconnect(false);
        wooting::wooting_usb_find_keyboard();

        let wooting_usb_meta = *wooting::wooting_usb_get_meta();
        let model = CStr::from_ptr(wooting_usb_meta.model);

        match model.to_str().unwrap() {
            //TODO: Verify these sizes for the one two and uwu
            "Wooting One" => (17, 6),
            "Wooting Two" | "Wooting Two LE" | "Wooting Two HE" | "Wooting Two HE (ARM)" => (21, 6),
            "Wooting 60HE" | "Wooting 60HE (ARM)" => (14, 5),
            "Wooting UwU" | "Wooting UwU RGB" => (3, 1),
            _ => {
                logf!(
                    Error,
                    "Unsupported keyboard model: {}",
                    model.to_str().unwrap()
                );
                (0, 0)
            }
        }
    }
}

pub fn get_device_name() -> String {
    unsafe {
        wooting::wooting_usb_disconnect(false);
        wooting::wooting_usb_find_keyboard();

        let wooting_usb_meta = *wooting::wooting_usb_get_meta();
        let model = CStr::from_ptr(wooting_usb_meta.model);

        model.to_str().unwrap().to_string()
    }
}

pub fn get_device_creation() -> String {
    unsafe {
        wooting::wooting_usb_disconnect(false);
        wooting::wooting_usb_find_keyboard();

        let len = u8::MAX as usize + 3;
        let mut buff = vec![0u8; len];
        wooting::wooting_usb_send_feature_with_response(buff.as_mut_ptr(), len, 3, 0, 0, 0, 0);

        let year: u16 = 2000 + buff[7] as u16;
        let week = buff[8];

        if year == 2000 && week == 0 {
            logf!(Warning, "Failed to get device creation date");
            "N/A".to_string()
        } else {
            format!("Week {} of {}", week, year)
        }
    }
}

pub fn draw_rgb(resized_capture: image::DynamicImage, brightness: u8, red_shift_fix: bool) {
    unsafe {
        for (x, y, pixel) in resized_capture.pixels() {
            let image::Rgba([r, g, b, _]) = pixel;
            let adjusted_r = if red_shift_fix {
                r.saturating_sub(40)
            } else {
                r
            };
            wooting::wooting_rgb_array_set_single(
                y as u8 + 1,
                x as u8,
                (adjusted_r as f32 * (brightness as f32 * 0.01)).round() as u8,
                (g as f32 * (brightness as f32 * 0.01)).round() as u8,
                (b as f32 * (brightness as f32 * 0.01)).round() as u8,
            );
        }

        wooting::wooting_rgb_array_update_keyboard();
    }
}

pub fn reconnect_device() {
    logf!(Info, "Reconnecting Device");
    unsafe {
        wooting::wooting_rgb_close();
        update_rgb();
    }
}

pub fn exit_rgb() {
    logf!(Info, "Exiting RGB");
    unsafe {
        wooting::wooting_rgb_close();
    }
}

pub fn update_rgb() {
    unsafe {
        wooting::wooting_rgb_array_update_keyboard();
    }
}
