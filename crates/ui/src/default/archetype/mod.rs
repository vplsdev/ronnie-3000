//! The Archetype layout: a light header with the chain as gear icons, a stage for the selected
//! block, a light global panel, and a black utility bar.
//!
//! It reuses the state owners of the reference layout (`ChromeController`, `ChainGallery`,
//! `BlockDetail`, `Faceplate`) and only places them differently.

use crate::theme::Look;
mod header;
mod utility;

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
    // Fill only what is left below any banner, so that the banner stays visible.
    ui.painter()
        .rect_filled(ui.available_rect_before_wrap(), 0.0, look.background);

    egui::Panel::top("archetype-header")
        .exact_size(HEADER_HEIGHT)
        .frame(egui::Frame::NONE.fill(look.background))
        .show(ui, |ui| header::show_header(ui, chrome, state));
    show_overlay(ui, chrome);
    egui::Panel::bottom("archetype-utility")
        .exact_size(UTILITY_HEIGHT)
        .frame(egui::Frame::NONE.fill(look.utility))
        .show(ui, |ui| utility::show_utility(ui, chrome, state));
    chrome.faceplate_mut().set_show_lanes(true);
    show_faceplate(ui, chrome, look.panel);
    if let Some(stereo) = chrome.faceplate_mut().take_lanes_request() {
        let _ = chrome.set_stereo(stereo);
    }

    // Settle selection first, so that the stage decides on this frame's state and never falls
    // through to the reference chain gallery.
    chrome.sync_detail();
    chrome.sync_browser();
    let mut focus = Focus::load(ui.ctx());
    focus.observe(chrome, state);
    let stage_free = !chrome.tuner_visible()
        && !chrome.auth_active()
        && chrome.browser_mut().is_none()
        && chrome.detail_mut().is_none()
        && matches!(
            chrome.gallery().selection(),
            super::chain::GallerySelection::None
        );
    if stage_free {
        match focus.next_block(state) {
            Some(id) => {
                chrome.gallery_mut().activate(&id);
                chrome.sync_detail();
            }
            None => {
                focus.store(ui.ctx());
                show_empty(ui, chrome, state, look);
                return;
            }
        }
    }
    focus.store(ui.ctx());
    if let Some(detail) = chrome.detail_mut() {
        detail.set_back_link(false);
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

/// Which block the stage showed, so that it can choose the right block after a change.
#[derive(Clone, Default)]
struct Focus {
    /// The block on the stage, and its lane and position among the tone blocks of that lane.
    shown: Option<(String, bool, usize)>,
    /// The slot that the tone browser fills, so that the loaded block opens.
    pending: Option<String>,
    /// The preset the stage last saw, so that a preset switch opens its first block.
    preset: Option<String>,
}

impl Focus {
    fn id() -> egui::Id {
        egui::Id::unique("archetype-focus")
    }

    fn load(ctx: &egui::Context) -> Self {
        ctx.data(|data| data.get_temp::<Self>(Self::id()))
            .unwrap_or_default()
    }

    fn store(&self, ctx: &egui::Context) {
        ctx.data_mut(|data| data.insert_temp(Self::id(), self.clone()));
    }

    /// Records the current selection and notices a preset switch.
    fn observe(&mut self, chrome: &ChromeController, state: &crate::backend::ChainViewState) {
        let preset = chrome.active_preset_name().map(str::to_owned);
        if preset != self.preset {
            if self.preset.is_some() || self.shown.is_some() {
                self.shown = None;
                self.pending = None;
            }
            self.preset = preset;
        }
        match chrome.gallery().selection() {
            super::chain::GallerySelection::BlockDetail(id) => {
                if let Some(place) = place_of(state, &id.0) {
                    self.shown = Some((id.0.clone(), place.0, place.1));
                }
            }
            super::chain::GallerySelection::SelectTone {
                target_block_id, ..
            } => {
                self.pending = Some(target_block_id.0.clone());
            }
            super::chain::GallerySelection::None => {}
        }
    }

    /// The block to open on a free stage: the block just loaded, else the neighbour of the block
    /// that went away, else the first block (lane A, then lane B).
    fn next_block(&mut self, state: &crate::backend::ChainViewState) -> Option<String> {
        if let Some(id) = self.pending.take()
            && place_of(state, &id).is_some()
        {
            return Some(id);
        }
        if let Some((id, right, index)) = &self.shown {
            if place_of(state, id).is_some() {
                return Some(id.clone());
            }
            let lane = tones(state, *right);
            if !lane.is_empty() {
                return Some(lane[(*index).min(lane.len() - 1)].clone());
            }
        }
        tones(state, false)
            .into_iter()
            .next()
            .or_else(|| tones(state, true).into_iter().next())
    }
}

/// The ids of the tone blocks of one lane, in order.
fn tones(state: &crate::backend::ChainViewState, right: bool) -> Vec<String> {
    let lane = if right {
        &state.chain.right
    } else {
        &state.chain.left
    };
    lane.iter()
        .filter(|block| !block.is_insert)
        .map(|block| block.id.0.clone())
        .collect()
}

/// The lane (right or not) and the position among that lane's tone blocks of block `id`.
fn place_of(state: &crate::backend::ChainViewState, id: &str) -> Option<(bool, usize)> {
    for right in [false, true] {
        if let Some(index) = tones(state, right).iter().position(|other| other == id) {
            return Some((right, index));
        }
    }
    None
}
