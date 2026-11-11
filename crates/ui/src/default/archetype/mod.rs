//! The Archetype layout: a light header with the chain as gear icons, a stage for the selected
//! block, a light global panel, and a black utility bar.
//!
//! It reuses the state owners of the reference layout (`ChromeController`, `ChainGallery`,
//! `BlockDetail`, `Faceplate`) and only places them differently.

use crate::theme::Look;
use eframe::egui;

use super::header::{ChromeController, show_faceplate, show_overlay, show_stage};

/// Header height in design points.
pub(super) const HEADER_HEIGHT: f32 = 64.0;
/// Utility bar height in design points; it takes the slot of the hint strip.
pub(super) const UTILITY_HEIGHT: f32 = 36.0;

/// Draws the whole Archetype window below any banner.
pub(super) fn show(
    ui: &mut egui::Ui,
    chrome: &mut ChromeController,
    _state: &crate::backend::ChainViewState,
    now_ms: i64,
) {
    let look = Look::archetype();
    Look::install(ui.ctx(), look);
    ui.painter()
        .rect_filled(ui.max_rect(), 0.0, look.background);

    egui::Panel::top("archetype-header")
        .exact_size(HEADER_HEIGHT)
        .frame(egui::Frame::NONE.fill(look.background))
        .show(ui, |ui| {
            let rect = ui.max_rect();
            ui.painter().hline(
                rect.x_range(),
                rect.bottom() - 0.5,
                egui::Stroke::new(1.0, look.edge),
            );
        });
    show_overlay(ui, chrome);
    egui::Panel::bottom("archetype-utility")
        .exact_size(UTILITY_HEIGHT)
        .frame(egui::Frame::NONE.fill(look.utility))
        .show(ui, |_ui| {});
    show_faceplate(ui, chrome, look.panel);
    show_stage(ui, chrome, now_ms);
}
