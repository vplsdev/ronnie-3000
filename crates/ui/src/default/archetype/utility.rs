//! Archetype utility bar: tools and global switches along the bottom edge.

use eframe::egui::{self, Align2, Rect, Stroke};

use crate::backend::ChainViewState;
use crate::theme::Look;

use super::super::{
    chain::GallerySelection, header::ChromeController, overlays::Overlay, popover::Trigger,
};

const TEXT_SIZE: f32 = 11.0;
const PADDING: f32 = 12.0;
/// OPTIMIZE text while the low-latency mode is on.
const OPTIMIZE_ON: egui::Color32 = egui::Color32::from_rgb(0x3d, 0xd6, 0x8c);

/// Draws the utility bar into the current panel.
pub(super) fn show_utility(
    ui: &mut egui::Ui,
    chrome: &mut ChromeController,
    state: &ChainViewState,
) {
    let look = Look::get(ui.ctx());
    let bounds = ui.max_rect();
    let font = egui::FontId::new(TEXT_SIZE, egui::FontFamily::Name("arimo-bold".into()));
    let mut x = bounds.left() + 10.0;
    let tuner = chrome.tuner_visible();
    let optimize = chrome.optimize();

    let items: [(&str, &'static str, bool); 5] = [
        ("TUNER", "Tuner", tuner),
        ("MIDI", "MIDI", false),
        ("OPTIMIZE", "Optimize", optimize),
        ("TONE3000", "TONE3000", false),
        ("SETTINGS", "Settings", false),
    ];
    let mut clicked = None;
    for (index, (text, label, active)) in items.into_iter().enumerate() {
        if index > 0 {
            divider(ui, x, bounds);
        }
        let (response, width) = item(ui, x, bounds, text, label, active, &font, look);
        if response.clicked() {
            clicked = Some(label);
        }
        x += width;
    }

    // Right side: the account, then CPU.
    let account_text = chrome
        .account_name()
        .map_or_else(|| "SIGN IN".to_owned(), |name| name.to_uppercase());
    let account_width = text_width(ui, &account_text, &font) + 2.0 * PADDING;
    let account_x = bounds.right() - 96.0 - account_width;
    divider(ui, account_x, bounds);
    let (account, _) = item(
        ui,
        account_x,
        bounds,
        &account_text,
        "Account",
        chrome.overlay() == Overlay::Account,
        &font,
        look,
    );
    if account.clicked() {
        clicked = Some("Account");
    }
    if chrome.overlay() == Overlay::Account {
        chrome.set_overlay_trigger(Some(Trigger::from(&account)));
    }

    ui.painter().text(
        egui::pos2(bounds.right() - 16.0, bounds.center().y),
        Align2::RIGHT_CENTER,
        format!("CPU {:.1}%", chrome.cpu_percent()),
        egui::FontId::monospace(TEXT_SIZE),
        egui::Color32::from_rgb(0xbd, 0xbd, 0xbd),
    );

    match clicked {
        Some("Tuner") => chrome.set_tuner(!tuner),
        Some("MIDI" | "Settings") => chrome.open_settings(),
        Some("Optimize") => {
            let _ = chrome.set_optimize(!optimize);
        }
        Some("TONE3000") => {
            let selected = match chrome.gallery().selection() {
                GallerySelection::BlockDetail(id) => Some(id.clone()),
                _ => None,
            };
            if let Some(id) = super::header::lane_insert(state, selected.as_ref()) {
                chrome.gallery_mut().activate(&id);
            }
        }
        Some("Account") => chrome.open_account(),
        _ => {}
    }
}

fn divider(ui: &egui::Ui, x: f32, bounds: Rect) {
    ui.painter().vline(
        x,
        (bounds.center().y - 6.0)..=(bounds.center().y + 6.0),
        Stroke::new(1.0, egui::Color32::from_rgb(0x4a, 0x4a, 0x4a)),
    );
}

fn text_width(ui: &egui::Ui, text: &str, font: &egui::FontId) -> f32 {
    ui.fonts_mut(|fonts| {
        fonts
            .layout_no_wrap(text.to_owned(), font.clone(), egui::Color32::WHITE)
            .size()
            .x
    })
}

/// One text button of the bar; returns its response and width.
#[allow(clippy::too_many_arguments)]
fn item(
    ui: &mut egui::Ui,
    x: f32,
    bounds: Rect,
    text: &str,
    label: &'static str,
    active: bool,
    font: &egui::FontId,
    look: Look,
) -> (egui::Response, f32) {
    let width = text_width(ui, text, font) + 2.0 * PADDING;
    let rect = Rect::from_min_size(
        egui::pos2(x, bounds.top()),
        egui::vec2(width, bounds.height()),
    );
    let response = ui.interact(
        rect,
        ui.scope_id().with(("archetype-utility", label)),
        egui::Sense::click(),
    );
    response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, label));
    if response.hovered() {
        ui.painter()
            .rect_filled(rect, 0.0, egui::Color32::from_rgb(0x1d, 0x1d, 0x1d));
    }
    let color = match (active, label) {
        (true, "Optimize") => OPTIMIZE_ON,
        (true, _) => look.accent,
        (false, _) => look.utility_text,
    };
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        text,
        font.clone(),
        color,
    );
    (response, width)
}
