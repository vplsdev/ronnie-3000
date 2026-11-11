//! Archetype utility bar: tools and global switches along the bottom edge.

use eframe::egui::{self, Align2, Rect, Stroke};

use crate::backend::ChainViewState;
use crate::theme::Look;

use super::super::header::ChromeController;

const TEXT_SIZE: f32 = 11.0;
const PADDING: f32 = 12.0;

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
    let stereo = state.chain.stereo_enabled;
    let tuner = chrome.tuner_visible();

    let items: [(&str, &'static str, bool, bool); 7] = [
        ("TUNER", "Tuner", true, tuner),
        ("MONO", "Mono mode", true, !stereo),
        ("STEREO", "Stereo mode", true, stereo),
        ("UNDO", "Undo", state.can_undo, false),
        ("REDO", "Redo", state.can_redo, false),
        ("ACCOUNT", "Account", true, false),
        ("SETTINGS", "Settings", true, false),
    ];
    let mut clicked = None;
    for (index, (text, label, enabled, active)) in items.into_iter().enumerate() {
        if index > 0 {
            ui.painter().vline(
                x,
                (bounds.center().y - 6.0)..=(bounds.center().y + 6.0),
                Stroke::new(1.0, egui::Color32::from_rgb(0x4a, 0x4a, 0x4a)),
            );
        }
        let width = ui.fonts_mut(|fonts| {
            fonts
                .layout_no_wrap(text.to_owned(), font.clone(), look.utility_text)
                .size()
                .x
        }) + 2.0 * PADDING;
        let rect = Rect::from_min_size(
            egui::pos2(x, bounds.top()),
            egui::vec2(width, bounds.height()),
        );
        let response = ui.interact(
            rect,
            ui.scope_id().with(("archetype-utility", label)),
            if enabled {
                egui::Sense::click()
            } else {
                egui::Sense::hover()
            },
        );
        response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, enabled, label));
        if response.hovered() && enabled {
            ui.painter()
                .rect_filled(rect, 0.0, egui::Color32::from_rgb(0x1d, 0x1d, 0x1d));
        }
        let color = if !enabled {
            egui::Color32::from_rgb(0x6a, 0x6a, 0x6a)
        } else if active {
            look.accent
        } else {
            look.utility_text
        };
        ui.painter().text(
            rect.center(),
            Align2::CENTER_CENTER,
            text,
            font.clone(),
            color,
        );
        if response.clicked() {
            clicked = Some(label);
        }
        x += width;
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
        Some("Mono mode") => {
            let _ = chrome.set_stereo(false);
        }
        Some("Stereo mode") => {
            let _ = chrome.set_stereo(true);
        }
        Some("Undo") => {
            let _ = chrome.undo();
        }
        Some("Redo") => {
            let _ = chrome.redo();
        }
        Some("Account") => chrome.open_account(),
        Some("Settings") => chrome.open_settings(),
        _ => {}
    }
}
