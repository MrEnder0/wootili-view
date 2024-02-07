use std::{sync::{atomic::AtomicBool, RwLock}, time::Duration};

use crate::wooting;
use image::{imageops::FilterType, DynamicImage, GenericImageView};
use lazy_static::lazy_static;
use scorched::{LogExpect, LogImportance};
use xcap::Monitor;
use std::sync::atomic::Ordering;

#[derive(Clone)]
pub struct CaptureSettings {
    pub screen_index: usize,
    pub downscale_method: FilterType,
    pub capture_frame_limit: u32,
    pub reduce_bright_effects: bool,
    pub red_shift_fix: bool,
    pub brightness: u8,
    pub device_name: String,
    pub rgb_size: (u32, u32),
    pub display_rgb_preview: bool,
}

pub static CAPTURE_SETTINGS_RELOAD: AtomicBool = AtomicBool::new(false);
pub static CAPTURE_SETTINGS: RwLock<CaptureSettings> = RwLock::new(CaptureSettings {
    screen_index: 0,
    downscale_method: FilterType::Triangle,
    reduce_bright_effects: false,
    red_shift_fix: false,
    brightness: 100,
    capture_frame_limit: 10,
    device_name: String::new(),
    rgb_size: (0, 0),
    display_rgb_preview: false,
});
pub static CAPTURE_LOCK: AtomicBool = AtomicBool::new(false);
lazy_static! {
    pub static ref CAPTURE_PREVIEW: RwLock<DynamicImage> = RwLock::new({
        let img = image::ImageBuffer::new(1, 1);
        image::DynamicImage::ImageRgba8(img)
    });
}

pub fn capture() {
    let mut current_settings = CaptureSettings {
        screen_index: 0,
        downscale_method: FilterType::Triangle,
        reduce_bright_effects: false,
        red_shift_fix: false,
        brightness: 100,
        capture_frame_limit: 10,
        device_name: wooting::get_device_name(),
        rgb_size: (0, 0),
        display_rgb_preview: false,
    };
    let mut last_frame = DynamicImage::new_rgba8(1, 1);
    let mut next_frame: Duration;

    loop {
        if CAPTURE_LOCK.load(Ordering::Relaxed) {
            std::thread::sleep(Duration::from_millis(10));
            continue;
        }

        if CAPTURE_SETTINGS_RELOAD.load(Ordering::Relaxed) {
            current_settings = CAPTURE_SETTINGS.read().unwrap().clone();
            CAPTURE_SETTINGS_RELOAD.store(false, Ordering::Relaxed);
        }

        let mut current_frame_reduce = false;

        let monitors = Monitor::all().unwrap();
        let capture = monitors[current_settings.screen_index]
            .capture_image()
            .unwrap();

        let img = image::DynamicImage::ImageRgba8(
            image::ImageBuffer::from_raw(capture.width(), capture.height(), capture.to_vec())
                .log_expect(
                    LogImportance::Error,
                    "Failed to convert capture to image buffer",
                ),
        );

        // If the image is the same as the last frame, we don't need to process it
        if img == last_frame {
            next_frame = Duration::from_millis(
                ((1.0 / current_settings.capture_frame_limit as f32) * 1000.0).round() as u64,
            );
            std::thread::sleep(next_frame - Duration::from_millis(1));
            continue;
        }

        last_frame = img.clone();

        let rgb_screen = img.resize_exact(
            current_settings.rgb_size.0,
            current_settings.rgb_size.1,
            current_settings.downscale_method,
        );

        if current_settings.display_rgb_preview {
            *CAPTURE_PREVIEW.write().unwrap() = rgb_screen.clone();
        }

        let frame_rgb_size: (u32, u32) = current_settings.rgb_size;
        if frame_rgb_size.0 != 0 && frame_rgb_size.1 != 0 {
            let resized_capture = rgb_screen.clone();

            if current_settings.reduce_bright_effects {
                let avg_screen =
                    resized_capture
                        .clone()
                        .resize(1, 1, image::imageops::FilterType::Gaussian);

                let image::Rgba([r, g, b, _]) = avg_screen.get_pixel(0, 0);

                if r > 220 || g > 220 || b > 220 {
                    current_frame_reduce = true;
                    current_settings.brightness -= 50;
                }
            }

            wooting::draw_rgb(
                resized_capture.clone(),
                current_settings.brightness,
                current_settings.red_shift_fix,
                current_settings.device_name.clone(),
            );

            if current_frame_reduce {
                current_settings.brightness += 50;
            }
        } else {
            wooting::reconnect_device();
            current_settings.device_name = wooting::get_device_name();

            current_settings.rgb_size = wooting::get_rgb_size()
                .log_expect(scorched::LogImportance::Error, "Failed to get rgb size");
        }

        next_frame = Duration::from_millis(
            ((1.0 / current_settings.capture_frame_limit as f32) * 1000.0).round() as u64,
        );
        std::thread::sleep(next_frame - Duration::from_millis(1));
    }
}
