//! The black menu that the + button opens: the gear categories, the lane that receives the
//! block, and the ways to load a file or open the tone browser.

use eframe::egui::{self, Align, Color32, FontFamily, FontId, Layout, Rect, RichText, Stroke};
use ronie_core::{ChainSide, Tone, ToneId, gear::GearCategory};

use crate::services::{catalog::CatalogStore, recent_tones::RecentTones};

use super::super::tile::{GearGlyph, draw_glyph};

const MENU_WIDTH: f32 = 380.0;
const PADDING: f32 = 8.0;
const HEAD: f32 = 34.0;
const ROW: f32 = 44.0;
const LINK: f32 = 36.0;
const GAP_BELOW_ANCHOR: f32 = 6.0;

const BACKGROUND: Color32 = Color32::from_rgb(0x0b, 0x0b, 0x0b);
const TEXT: Color32 = Color32::from_rgb(0xf2, 0xf2, 0xf2);
const MUTED: Color32 = Color32::from_rgb(0x8a, 0x8a, 0x87);
const SOFT: Color32 = Color32::from_rgb(0xbd, 0xbd, 0xbd);
const HOVER: Color32 = Color32::from_rgb(0x1d, 0x1d, 0x1d);
const HAIRLINE: Color32 = Color32::from_rgb(0x26, 0x26, 0x26);

/// What the menu shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum MenuView {
    /// The gear categories and the two links.
    Categories,
    /// The tones of one category.
    Results(GearCategory),
}

/// What the player did in the menu.
#[allow(dead_code)] // The results view builds Add, SignIn and Search.
#[derive(Debug, Clone, PartialEq)]
pub(super) enum AddMenuAction {
    /// Open the tone browser, narrowed to a category when there is one.
    OpenBrowser(Option<GearCategory>),
    /// Pick a local model file.
    LoadFile,
    /// Add a tone to the chosen lane.
    Add(Tone),
    /// Start the sign-in.
    SignIn,
    /// The search text changed.
    Search,
}

/// The menu's own state; it lives in egui temp data between frames.
#[derive(Debug, Clone)]
pub(super) struct AddMenu {
    pub(super) open: bool,
    pub(super) view: MenuView,
    pub(super) lane: ChainSide,
    pub(super) search: String,
}

impl Default for AddMenu {
    fn default() -> Self {
        Self {
            open: false,
            view: MenuView::Categories,
            lane: ChainSide::Left,
            search: String::new(),
        }
    }
}

impl AddMenu {
    fn id() -> egui::Id {
        egui::Id::unique("archetype-add-menu")
    }

    pub(super) fn load(ctx: &egui::Context) -> Self {
        ctx.data(|data| data.get_temp::<Self>(Self::id()))
            .unwrap_or_default()
    }

    pub(super) fn store(&self, ctx: &egui::Context) {
        ctx.data_mut(|data| data.insert_temp(Self::id(), self.clone()));
    }

    /// Opens the menu on its categories, for one lane.
    pub(super) fn open_for(&mut self, lane: ChainSide) {
        self.open = true;
        self.view = MenuView::Categories;
        self.lane = lane;
        self.search.clear();
    }
}

/// What the menu needs to know about the rest of the app.
#[allow(dead_code)] // The results view reads the fields that the categories do not.
pub(super) struct MenuData<'a> {
    pub(super) stereo: bool,
    pub(super) library: &'a RecentTones,
    pub(super) in_chain: Vec<ToneId>,
    pub(super) signed_in: bool,
    pub(super) results: Option<&'a CatalogStore>,
    pub(super) loading: Option<ToneId>,
}

const CATEGORIES: [(GearCategory, GearGlyph, &str, &str); 6] = [
    (
        GearCategory::Pedal,
        GearGlyph::Pedal,
        "PEDAL",
        "Overdrive, distortion, fuzz, boost, compressor",
    ),
    (
        GearCategory::Amp,
        GearGlyph::Amp,
        "AMP",
        "Heads and combos, no cab",
    ),
    (
        GearCategory::FullRig,
        GearGlyph::FullRig,
        "AMP + CAB",
        "A whole rig in one capture",
    ),
    (
        GearCategory::Cab,
        GearGlyph::Cabinet,
        "CAB",
        "Impulse responses, mic positions",
    ),
    (
        GearCategory::Outboard,
        GearGlyph::Outboard,
        "OUTBOARD",
        "Preamps, rack EQ and compressors",
    ),
    (
        GearCategory::Space,
        GearGlyph::Space,
        "SPACE",
        "Rooms and reverbs",
    ),
];

fn category_name(category: GearCategory) -> &'static str {
    CATEGORIES
        .iter()
        .find(|row| row.0 == category)
        .map_or("", |row| row.2)
}

fn sans(size: f32) -> FontId {
    FontId::new(size, FontFamily::Proportional)
}

fn bold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("arimo-bold".into()))
}

/// Puts one line of text in `rect`, at its left or right edge and centred in height.
fn text_in(ui: &mut egui::Ui, rect: Rect, text: RichText, right: bool) {
    let layout = if right {
        Layout::right_to_left(Align::Center)
    } else {
        Layout::left_to_right(Align::Center)
    };
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(rect).layout(layout));
    child.add(egui::Label::new(text).selectable(false).truncate());
}

fn tracked(text: &str, font: FontId, color: Color32, em: f32) -> RichText {
    let size = font.size;
    RichText::new(text)
        .font(font)
        .color(color)
        .extra_letter_spacing(size * em)
}

fn menu_item(ui: &egui::Ui, rect: Rect, id: &str, label: &str) -> egui::Response {
    let response = ui.interact(rect, ui.scope_id().with(id), egui::Sense::click());
    response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::MenuItem, true, label));
    response
}

fn chevron(painter: &egui::Painter, center: egui::Pos2, left: bool, color: Color32) {
    let (a, b) = if left { (2.0, -2.0) } else { (-2.0, 2.0) };
    painter.line(
        vec![
            center + egui::vec2(a, -4.0),
            center + egui::vec2(b, 0.0),
            center + egui::vec2(a, 4.0),
        ],
        Stroke::new(1.5, color),
    );
}

/// Draws the menu under `anchor` and returns what the player chose. The menu closes itself on
/// Escape and on a click outside it and outside `anchor`.
pub(super) fn show(
    ui: &mut egui::Ui,
    anchor: Rect,
    menu: &mut AddMenu,
    data: &MenuData<'_>,
) -> Option<AddMenuAction> {
    let height = match menu.view {
        MenuView::Categories => PADDING * 2.0 + HEAD + 4.0 + ROW * 6.0 + 13.0 + LINK * 2.0,
        MenuView::Results(_) => PADDING * 2.0 + HEAD,
    };
    let screen = ui.ctx().content_rect();
    let left = (anchor.center().x - MENU_WIDTH / 2.0)
        .clamp(screen.left() + 4.0, screen.right() - MENU_WIDTH - 4.0);
    let rect = Rect::from_min_size(
        egui::pos2(left, anchor.bottom() + GAP_BELOW_ANCHOR),
        egui::vec2(MENU_WIDTH, height),
    );
    let mut action = None;
    egui::Area::new(egui::Id::unique("archetype-add-menu-area"))
        .order(egui::Order::Foreground)
        .fixed_pos(rect.min)
        .show(ui.ctx(), |ui| {
            ui.set_min_size(rect.size());
            ui.set_max_size(rect.size());
            action = draw(ui, rect, anchor, menu, data);
        });

    let (escape, clicked_at) = ui.input(|input| {
        (
            input.key_pressed(egui::Key::Escape),
            input
                .pointer
                .primary_clicked()
                .then(|| input.pointer.latest_pos())
                .flatten(),
        )
    });
    if escape || clicked_at.is_some_and(|pos| !rect.contains(pos) && !anchor.contains(pos)) {
        menu.open = false;
    }
    action
}

fn draw(
    ui: &mut egui::Ui,
    rect: Rect,
    anchor: Rect,
    menu: &mut AddMenu,
    data: &MenuData<'_>,
) -> Option<AddMenuAction> {
    let painter = ui.painter().clone();
    painter.rect_filled(
        rect.translate(egui::vec2(0.0, 6.0)).expand(1.0),
        8.0,
        Color32::from_black_alpha(40),
    );
    painter.rect_filled(rect, 8.0, BACKGROUND);
    let tip = anchor
        .center()
        .x
        .clamp(rect.left() + 20.0, rect.right() - 20.0);
    painter.add(egui::Shape::convex_polygon(
        vec![
            egui::pos2(tip - 6.0, rect.top() + 0.5),
            egui::pos2(tip, rect.top() - 6.0),
            egui::pos2(tip + 6.0, rect.top() + 0.5),
        ],
        BACKGROUND,
        Stroke::NONE,
    ));

    let inner = rect.shrink(PADDING);
    let head = Rect::from_min_size(inner.min, egui::vec2(inner.width(), HEAD));
    let mut action = None;

    match menu.view {
        MenuView::Categories => {
            text_in(
                ui,
                Rect::from_min_max(head.min + egui::vec2(14.0, 0.0), head.max),
                tracked("ADD BLOCK", bold(10.0), TEXT, 0.14),
                false,
            );
        }
        MenuView::Results(category) => {
            let back = Rect::from_min_size(head.min, egui::vec2(120.0, HEAD));
            let response = ui.interact(back, ui.scope_id().with("back"), egui::Sense::click());
            response.widget_info(|| {
                egui::WidgetInfo::labeled(egui::Role::Button, true, "Back to categories")
            });
            chevron(
                &painter,
                back.left_center() + egui::vec2(18.0, 0.0),
                true,
                TEXT,
            );
            text_in(
                ui,
                Rect::from_min_max(back.min + egui::vec2(32.0, 0.0), back.max),
                tracked(category_name(category), bold(10.0), TEXT, 0.14),
                false,
            );
            if response.clicked() {
                menu.view = MenuView::Categories;
            }
        }
    }
    lane_picker(ui, head, menu, data.stereo);

    if menu.view == MenuView::Categories {
        let mut top = head.bottom() + 4.0;
        for (category, glyph, name, hint) in CATEGORIES {
            let row = Rect::from_min_size(
                egui::pos2(inner.left(), top),
                egui::vec2(inner.width(), ROW),
            );
            top += ROW;
            let response = menu_item(ui, row, name, name);
            if response.hovered() {
                painter.rect_filled(row, 6.0, HOVER);
            }
            let icon = Rect::from_center_size(
                egui::pos2(row.left() + 14.0 + 13.0, row.center().y),
                egui::Vec2::splat(26.0),
            );
            draw_glyph(&painter, icon, glyph, TEXT);
            let text_left = row.left() + 52.0;
            let text_right = row.right() - 76.0;
            text_in(
                ui,
                Rect::from_min_max(
                    egui::pos2(text_left, row.center().y - 14.5),
                    egui::pos2(text_right, row.center().y),
                ),
                tracked(name, bold(11.0), TEXT, 0.08),
                false,
            );
            text_in(
                ui,
                Rect::from_min_max(
                    egui::pos2(text_left, row.center().y),
                    egui::pos2(text_right, row.center().y + 14.5),
                ),
                RichText::new(hint).font(sans(11.0)).color(MUTED),
                false,
            );
            let count = data.library.count(category);
            text_in(
                ui,
                Rect::from_min_max(
                    egui::pos2(text_right, row.top()),
                    egui::pos2(row.right() - 38.0, row.bottom()),
                ),
                RichText::new(if count == 0 {
                    "\u{2014}".to_owned()
                } else {
                    count.to_string()
                })
                .font(FontId::monospace(10.0))
                .color(MUTED),
                true,
            );
            chevron(
                &painter,
                egui::pos2(row.right() - 20.0, row.center().y),
                false,
                MUTED,
            );
            if response.clicked() {
                menu.view = MenuView::Results(category);
            }
        }
        painter.hline(
            (inner.left() + 6.0)..=(inner.right() - 6.0),
            top + 6.5,
            Stroke::new(1.0, HAIRLINE),
        );
        top += 13.0;
        for (index, (label, hint, mono)) in [
            ("LOAD A FILE\u{2026}", ".nam \u{b7} .wav", true),
            ("OPEN THE TONE BROWSER", "filters, sort, pages", false),
        ]
        .into_iter()
        .enumerate()
        {
            let row = Rect::from_min_size(
                egui::pos2(inner.left(), top),
                egui::vec2(inner.width(), LINK),
            );
            top += LINK;
            let response = menu_item(ui, row, label, label);
            if response.hovered() {
                painter.rect_filled(row, 6.0, HOVER);
            }
            let icon = Rect::from_center_size(
                egui::pos2(row.left() + 14.0 + 7.0, row.center().y),
                egui::Vec2::splat(14.0),
            );
            link_icon(&painter, icon, index == 0);
            text_in(
                ui,
                Rect::from_min_max(
                    egui::pos2(row.left() + 14.0 + 14.0 + 10.0, row.top()),
                    egui::pos2(row.right() - 130.0, row.bottom()),
                ),
                tracked(label, bold(11.0), TEXT, 0.06),
                false,
            );
            text_in(
                ui,
                Rect::from_min_max(
                    egui::pos2(row.right() - 130.0, row.top()),
                    egui::pos2(row.right() - 14.0, row.bottom()),
                ),
                RichText::new(hint)
                    .font(if mono {
                        FontId::monospace(10.0)
                    } else {
                        sans(10.0)
                    })
                    .color(MUTED),
                true,
            );
            if response.clicked() {
                action = Some(if index == 0 {
                    AddMenuAction::LoadFile
                } else {
                    AddMenuAction::OpenBrowser(None)
                });
            }
        }
    }
    action
}

/// The lane choice in the head row, or the line that says where a block goes in mono.
fn lane_picker(ui: &mut egui::Ui, head: Rect, menu: &mut AddMenu, stereo: bool) {
    let right = head.right() - 6.0;
    if !stereo {
        text_in(
            ui,
            Rect::from_min_max(
                egui::pos2(right - 200.0, head.top()),
                egui::pos2(right, head.bottom()),
            ),
            RichText::new("to the end of the chain")
                .font(sans(10.0))
                .color(MUTED),
            true,
        );
        return;
    }
    const SEGMENT: f32 = 64.0;
    let track = Rect::from_center_size(
        egui::pos2(right - (SEGMENT + 2.0), head.center().y),
        egui::vec2(SEGMENT * 2.0 + 4.0, 22.0),
    );
    let painter = ui.painter().clone();
    painter.rect_filled(track, 4.0, HOVER);
    text_in(
        ui,
        Rect::from_min_max(
            egui::pos2(track.left() - 28.0, head.top()),
            egui::pos2(track.left() - 8.0, head.bottom()),
        ),
        RichText::new("to").font(sans(10.0)).color(MUTED),
        true,
    );
    for (index, (side, label)) in [(ChainSide::Left, "LANE A"), (ChainSide::Right, "LANE B")]
        .into_iter()
        .enumerate()
    {
        let segment = Rect::from_min_size(
            track.min + egui::vec2(2.0 + SEGMENT * index as f32, 2.0),
            egui::vec2(SEGMENT, 18.0),
        );
        let chosen = menu.lane == side;
        let response = ui.interact(segment, ui.scope_id().with(label), egui::Sense::click());
        response.widget_info(|| {
            egui::WidgetInfo::selected(egui::Role::RadioButton, true, chosen, label)
        });
        if chosen {
            painter.rect_filled(segment, 3.0, TEXT);
        }
        painter.text(
            segment.center(),
            egui::Align2::CENTER_CENTER,
            label,
            bold(10.0),
            if chosen { BACKGROUND } else { SOFT },
        );
        if response.clicked() {
            menu.lane = side;
        }
    }
}

/// The small outline icon of a link row: a page, or a magnifier.
fn link_icon(painter: &egui::Painter, rect: Rect, page: bool) {
    let stroke = Stroke::new(1.5, TEXT);
    if page {
        painter.rect_stroke(
            rect.shrink2(egui::vec2(2.0, 0.5)),
            2.0,
            stroke,
            egui::StrokeKind::Inside,
        );
        painter.hline(
            (rect.left() + 5.0)..=(rect.right() - 5.0),
            rect.center().y,
            stroke,
        );
    } else {
        let lens = rect.center() - egui::vec2(1.5, 1.5);
        painter.circle_stroke(lens, 4.5, stroke);
        painter.line_segment(
            [
                lens + egui::vec2(3.2, 3.2),
                rect.right_bottom() - egui::vec2(1.0, 1.0),
            ],
            stroke,
        );
    }
}
