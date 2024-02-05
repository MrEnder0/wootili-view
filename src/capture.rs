use std::{sync::RwLock, time::Duration};

use crate::wooting::RGB_SIZE;
use image::imageops::FilterType;
use scorched::{LogExpect, LogImportance};
use xcap::Monitor;

pub struct CaptureSettings {
    pub screen_index: usize,
    pub downscale_method: FilterType,
    pub capture_frame_limit: u32,
}

pub static CAPTURE_SETTINGS_RELOAD: RwLock<bool> = RwLock::new(false);
pub static CAPTURE_SETTINGS: RwLock<CaptureSettings> = RwLock::new(CaptureSettings {
    screen_index: 0,
    downscale_method: FilterType::Triangle,
    capture_frame_limit: 10,
});

pub fn capture() {
    let mut next_frame: Duration;
    let mut screen_index = 0;
    let mut downscale_method = FilterType::Triangle;
    let mut capture_frame_limit = 10;

    loop {
        if *CAPTURE_SETTINGS_RELOAD.read().unwrap() {
            let new_settings = CAPTURE_SETTINGS.read().unwrap();
            screen_index = new_settings.screen_index;
            downscale_method = new_settings.downscale_method;
            capture_frame_limit = new_settings.capture_frame_limit;
            *CAPTURE_SETTINGS_RELOAD.write().unwrap() = false;
        }

        let frame_rgb_size = *RGB_SIZE.read().unwrap();

        let monitors = Monitor::all().unwrap();
        let capture = monitors[screen_index].capture_image().unwrap();

        let img = image::DynamicImage::ImageRgba8(
            image::ImageBuffer::from_raw(capture.width(), capture.height(), capture.to_vec())
                .log_expect(
                    LogImportance::Error,
                    "Failed to convert capture to image buffer",
                ),
        );
        let resized_capture =
            img.resize_exact(frame_rgb_size.0, frame_rgb_size.1, downscale_method);

        crate::wooting::SCREEN
            .write()
            .unwrap()
            .clone_from(&resized_capture);

        next_frame =
            Duration::from_millis(((1.0 / capture_frame_limit as f32) * 1000.0).round() as u64);
        std::thread::sleep(next_frame - Duration::from_millis(1));
    }
}
