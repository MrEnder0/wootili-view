use eframe::egui::{SelectableLabel, Ui};
use image::imageops::FilterType;

use crate::DOWNSCALE_METHOD;

pub fn downscale_label(
    ui: &mut Ui,
    current: &mut FilterType,
    new: FilterType,
    label: &str,
    hover_text: &str,
) {
    if ui
        .add(SelectableLabel::new(*current == new, label))
        .on_hover_text(hover_text)
        .clicked()
    {
        DOWNSCALE_METHOD.lock().unwrap().clone_from(&new);
        *current = new;
    }
}
