//! The Archetype layout: a light header with the chain as gear icons, a stage for the selected
//! block, a light global panel, and a black utility bar.
//!
//! It reuses the state owners of the reference layout (`ChromeController`, `ChainGallery`,
//! `BlockDetail`, `Faceplate`) and only places them differently.

use crate::theme::Look;
mod header;

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
    state: &crate::backend::ChainViewState,
    now_ms: i64,
) {
    let look = Look::archetype();
    Look::install(ui.ctx(), look);
    ui.painter()
        .rect_filled(ui.max_rect(), 0.0, look.background);

    egui::Panel::top("archetype-header")
        .exact_size(HEADER_HEIGHT)
        .frame(egui::Frame::NONE.fill(look.background))
        .show(ui, |ui| header::show_header(ui, chrome, state));
    show_overlay(ui, chrome);
    egui::Panel::bottom("archetype-utility")
        .exact_size(UTILITY_HEIGHT)
        .frame(egui::Frame::NONE.fill(look.utility))
        .show(ui, |_ui| {});
    show_faceplate(ui, chrome, look.panel);
    let first_tone = state
        .chain
        .left
        .iter()
        .find(|block| !block.is_insert)
        .map(|block| block.id.0.clone());
    let stage_free = !chrome.tuner_visible()
        && !chrome.auth_active()
        && chrome.browser_mut().is_none()
        && chrome.detail_mut().is_none()
        && matches!(
            chrome.gallery().selection(),
            super::chain::GallerySelection::None
        );
    if stage_free {
        match first_tone {
            // A block always shows: with nothing selected, open the first block of lane A.
            Some(id) => chrome.gallery_mut().activate(&id),
            None => {
                show_empty(ui, chrome, state, look);
                return;
            }
        }
    }
    show_stage(ui, chrome, now_ms);
}

/// The stage of a chain with no tone blocks: a prompt and a button that opens the browser.
fn show_empty(
    ui: &mut egui::Ui,
    chrome: &mut ChromeController,
    state: &crate::backend::ChainViewState,
    look: Look,
) {
    let area = ui.available_rect_before_wrap();
    let painter = ui.painter().clone();
    ui.put(
        egui::Rect::from_center_size(
            area.center() - egui::vec2(0.0, 28.0),
            egui::vec2(400.0, 28.0),
        ),
        egui::Label::new(
            egui::RichText::new("Your chain is empty")
                .font(egui::FontId::new(
                    18.0,
                    egui::FontFamily::Name("arimo-bold".into()),
                ))
                .color(look.text),
        )
        .selectable(false),
    );
    let button = egui::Rect::from_center_size(
        area.center() + egui::vec2(0.0, 16.0),
        egui::vec2(180.0, 36.0),
    );
    let response = ui.interact(
        button,
        ui.scope_id().with("archetype-first-block"),
        egui::Sense::click(),
    );
    response.widget_info(|| {
        egui::WidgetInfo::labeled(egui::Role::Button, true, "Add your first block")
    });
    painter.rect_filled(
        button,
        6.0,
        if response.hovered() {
            look.accent
        } else {
            look.text
        },
    );
    painter.text(
        button.center(),
        egui::Align2::CENTER_CENTER,
        "Add a block",
        egui::FontId::new(13.0, egui::FontFamily::Name("arimo-bold".into())),
        look.background,
    );
    if response.clicked() {
        let insert = state
            .chain
            .left
            .iter()
            .find(|block| block.is_insert)
            .map(|block| block.id.0.clone());
        if let Some(id) = insert {
            chrome.gallery_mut().activate(&id);
        }
    }
}
