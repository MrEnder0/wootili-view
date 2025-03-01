use eframe::egui;
use libloading::Library;
use std::sync::{Arc, OnceLock};

#[derive(Clone)]
pub struct Plugin {
    pub name: String,
    pub lib: Arc<Library>,
}

static AVAILABLE_PLUGINS: OnceLock<Vec<Plugin>> = OnceLock::new();

pub fn get_available_plugins() -> Vec<Plugin> {
    unsafe {
        AVAILABLE_PLUGINS
            .get_or_init(|| {
                let mut detected_plugins = Vec::new();

                // Update Check Plugin
                if let Ok(lib) = Library::new("update_check") {
                    if let Ok(_) = {
                        lib.get::<extern "C" fn(String) -> Option<String>>(
                            "update_check_ui".as_bytes(),
                        )
                    } {
                        detected_plugins.push(Plugin {
                            name: "update_check".to_string(),
                            lib: Arc::new(lib),
                        });
                    }
                }
                detected_plugins
            })
            .to_vec()
    }
}

pub fn update_check_ui(loaded_lib: Arc<Library>, ui: &mut egui::Ui, log_path: String) {
    unsafe {
        let update_check_ui: libloading::Symbol<extern "C" fn(&mut egui::Ui, String)> =
            match loaded_lib.get("update_check_ui".as_bytes()) {
                Ok(update_check_ui) => update_check_ui,
                Err(_) => return,
            };
        update_check_ui(ui, log_path);
    }
}
