//! The Archetype layout: light header with the chain as gear icons, stage, light global panel
//! and black utility bar.

use std::sync::Arc;

use ronie_ui::{
    backend::{MockBackend, UiBackend},
    default::ChromeController,
    driver::UiDriver,
    shell::ViewMode,
    testbed::{UiScenario, load_scenarios},
    theme::Look,
};

fn scenario(id: &str) -> UiScenario {
    load_scenarios(include_str!("../../../fixtures/ui-scenarios.json"))
        .expect("scenarios")
        .into_iter()
        .find(|scenario| scenario.id == id)
        .expect("scenario exists")
}

fn archetype_driver(id: &str) -> (UiDriver, Arc<MockBackend>) {
    let backend = Arc::new(MockBackend::from_scenario(&scenario(id)));
    let mut chrome = ChromeController::new(backend.clone(), Vec::new(), false, true);
    chrome.set_view_mode(ViewMode::RonieFlow);
    let mut driver = UiDriver::new(chrome, (*backend).clone());
    driver.settle();
    (driver, backend)
}

#[test]
fn reference_is_the_default_view_mode() {
    let backend = Arc::new(MockBackend::from_scenario(&scenario("main-mono")));
    let chrome = ChromeController::new(backend, Vec::new(), false, true);
    assert_eq!(chrome.view_mode(), ViewMode::ReferenceDefault);
}

#[test]
fn archetype_mode_installs_the_light_look() {
    let (driver, _) = archetype_driver("main-mono");
    assert_eq!(Look::get(driver.context()), Look::archetype());
}

#[test]
fn settings_still_open_in_archetype_mode() {
    let (mut driver, _) = archetype_driver("main-mono");
    driver.chrome_mut().open_settings();
    driver.settle();
    assert!(driver.chrome_mut().settings_mut().is_some());
    assert!(
        !driver.is_visible("Browse presets"),
        "settings replace the whole window"
    );
}

#[test]
fn the_global_panel_hides_while_the_browser_is_open() {
    let (mut driver, _) = archetype_driver("main-mono");
    assert!(
        driver.label_starting_with("Gate").is_some(),
        "the global panel shows"
    );
    let _ = driver.click("Add a block").expect("the + button");
    driver.settle();
    let _ = driver.click("OPEN THE TONE BROWSER").expect("browser link");
    driver.settle();
    assert!(
        driver.chrome_mut().browser_mut().is_some(),
        "the tone browser opened"
    );
    assert!(
        driver.label_starting_with("Gate").is_none(),
        "the global panel hides under the browser"
    );
}

// ---------------------------------------------------------------- Task 3: header

use ronie_core::BlockId;
use ronie_ui::default::GallerySelection;

/// The scenarios document with one scenario edited, loaded as a driver in Archetype mode.
fn edited_driver(
    id: &str,
    edit: impl FnOnce(&mut serde_json::Value),
) -> (UiDriver, Arc<MockBackend>) {
    let mut doc: serde_json::Value =
        serde_json::from_str(include_str!("../../../fixtures/ui-scenarios.json")).expect("json");
    let scenario = doc["scenarios"]
        .as_array_mut()
        .expect("scenarios")
        .iter_mut()
        .find(|scenario| scenario["id"] == id)
        .expect("scenario exists");
    edit(&mut scenario["state"]["chain"]);
    let scenario = load_scenarios(&doc.to_string())
        .expect("scenarios")
        .into_iter()
        .find(|scenario| scenario.id == id)
        .expect("scenario exists");
    let backend = Arc::new(MockBackend::from_scenario(&scenario));
    let mut chrome = ChromeController::new(backend.clone(), Vec::new(), false, true);
    chrome.set_view_mode(ViewMode::RonieFlow);
    let mut driver = UiDriver::new(chrome, (*backend).clone());
    driver.settle();
    (driver, backend)
}

fn rect_of(driver: &UiDriver, label: &str) -> Option<eframe::egui::Rect> {
    driver
        .controls()
        .iter()
        .find(|control| control.label == label)
        .map(|control| control.rect)
}

const VOX: &str = "Chain: '02 Vox AC30/6 Top Boost";

#[test]
fn strip_shows_one_button_per_block() {
    let (driver, _) = archetype_driver("main-mono");
    for title in [
        "Chain: Green Russian Big Muff + BD-2",
        VOX,
        "Chain: UK Greenback - Marshall 1960TV 4x12 M201",
        "Add a block",
    ] {
        assert!(driver.is_visible(title), "{title} is missing");
    }
    for label in [
        "Previous preset",
        "Next preset",
        "Browse presets",
        "Save preset",
        "New preset",
    ] {
        assert!(driver.is_visible(label), "{label} is missing");
    }
}

#[test]
fn clicking_a_block_opens_it() {
    let (mut driver, _) = archetype_driver("main-mono");
    let _ = driver.click(VOX).expect("block button");
    driver.settle();
    assert!(
        matches!(
            driver.chrome().gallery().selection(),
            GallerySelection::BlockDetail(id) if *id == BlockId("blk-2".to_owned())
        ) || driver.chrome_mut().detail_mut().is_some(),
        "the block opened"
    );
}

#[test]
fn right_clicking_a_block_bypasses_it() {
    let (mut driver, backend) = archetype_driver("main-mono");
    let _ = driver.secondary_click(VOX).expect("block button");
    driver.settle();
    let enabled = backend.chain_state(0).chain.left[1].params.enabled;
    assert!(!enabled, "the block is bypassed");
}

const CATEGORY_ROWS: [&str; 6] = ["PEDAL", "AMP", "AMP + CAB", "CAB", "OUTBOARD", "SPACE"];

fn open_menu(driver: &mut UiDriver) {
    let _ = driver.click("Add a block").expect("plus");
    driver.settle();
}

#[test]
fn plus_opens_the_add_menu() {
    let (mut driver, _) = archetype_driver("main-mono");
    open_menu(&mut driver);
    for label in CATEGORY_ROWS
        .into_iter()
        .chain(["LOAD A FILE\u{2026}", "OPEN THE TONE BROWSER"])
    {
        assert!(driver.is_visible(label), "{label} is missing");
    }
    assert!(driver.chrome_mut().browser_mut().is_none());
    assert!(
        driver.chrome_mut().detail_mut().is_some(),
        "the stage still shows the amp page"
    );
}

#[test]
fn a_second_click_or_escape_closes_the_menu() {
    let (mut driver, _) = archetype_driver("main-mono");
    open_menu(&mut driver);
    let _ = driver.click("Add a block").expect("plus");
    driver.settle();
    assert!(!driver.is_visible("PEDAL"), "the second click closes it");
    open_menu(&mut driver);
    assert!(driver.is_visible("PEDAL"));
    let _ = driver.press_key(eframe::egui::Key::Escape);
    driver.settle();
    assert!(!driver.is_visible("PEDAL"), "escape closes it");
}

#[test]
fn clicking_outside_closes_the_menu() {
    let (mut driver, _) = archetype_driver("main-mono");
    open_menu(&mut driver);
    assert!(driver.is_visible("PEDAL"));
    let _ = driver.click_empty_space();
    driver.settle();
    assert!(!driver.is_visible("PEDAL"));
}

fn lane_radio(driver: &UiDriver, label: &str) -> bool {
    driver
        .controls()
        .iter()
        .find(|control| control.label == label)
        .unwrap_or_else(|| panic!("{label} is missing"))
        .checked
}

#[test]
fn stereo_menu_offers_lane_a_and_b() {
    let (mut driver, _) = archetype_driver("main-stereo");
    open_menu(&mut driver);
    assert!(lane_radio(&driver, "LANE A"), "lane A holds the selection");
    assert!(!lane_radio(&driver, "LANE B"));
    let _ = driver.click("Chain: Roland JC-40").expect("lane B block");
    driver.settle();
    let _ = driver.click("Add a block").expect("plus");
    driver.settle();
    if !driver.is_visible("LANE B") {
        open_menu(&mut driver);
    }
    assert!(lane_radio(&driver, "LANE B"), "lane B holds the selection");
    assert!(!lane_radio(&driver, "LANE A"));
}

#[test]
fn mono_menu_adds_to_the_end_of_the_chain() {
    let (mut driver, _) = archetype_driver("main-mono");
    open_menu(&mut driver);
    assert!(driver.shows_text("to the end of the chain"));
    assert!(!driver.is_visible("LANE A") && !driver.is_visible("LANE B"));
}

fn remembered_driver(tones: &[(i64, &str)]) -> UiDriver {
    use ronie_core::gear::GearCategory;
    use ronie_ui::{backend::UiAction, services::recent_tones::RecentTones};
    let mut recent = RecentTones::default();
    for (id, gear) in tones {
        recent.remember(
            ronie_core::Tone::from_value(serde_json::json!({
                "id": id, "title": format!("Tone {id}"), "format": "nam", "gear": gear,
                "models": [{"id": id * 10, "name": "m",
                    "model_url": format!("https://fixtures.invalid/m/{id}.nam")}]
            }))
            .expect("tone"),
        );
    }
    assert_eq!(recent.count(GearCategory::Amp), 2);
    let backend = Arc::new(MockBackend::from_scenario(&scenario("main-mono")));
    backend
        .dispatch(UiAction::SetTextPreference {
            key: RecentTones::KEY.to_owned(),
            value: recent.to_text(),
        })
        .expect("seed");
    let mut chrome = ChromeController::new(backend.clone(), Vec::new(), false, true);
    chrome.set_view_mode(ViewMode::RonieFlow);
    let mut driver = UiDriver::new(chrome, (*backend).clone());
    driver.settle();
    driver
}

#[test]
fn category_rows_count_the_library() {
    let mut driver = remembered_driver(&[(1, "amp"), (2, "amp")]);
    open_menu(&mut driver);
    assert!(driver.shows_text("2"), "the AMP row counts two tones");
    assert!(
        driver.shows_text("\u{2014}"),
        "an empty category shows a dash"
    );
    let (mut empty, _) = archetype_driver("main-mono");
    open_menu(&mut empty);
    assert!(!empty.shows_text("2"));
}

#[test]
fn mono_falls_back_to_lane_a() {
    let (mut driver, _) = archetype_driver("main-stereo");
    let _ = driver.click("Chain: Roland JC-40").expect("lane B block");
    driver.settle();
    open_menu(&mut driver);
    assert!(lane_radio(&driver, "LANE B"));
    // A click on the switch would close the menu, so the chain changes behind it.
    let _ = driver.chrome_mut().set_stereo(false);
    driver.settle();
    assert!(driver.shows_text("to the end of the chain"));
    let _ = driver.click("OPEN THE TONE BROWSER").expect("link");
    driver.settle();
    assert!(driver.chrome_mut().browser_mut().is_some());
    let target = driver
        .chrome_mut()
        .load_flow_mut()
        .pending_target()
        .cloned()
        .expect("a target");
    assert_eq!(target.side, ronie_core::ChainSide::Left);
}

#[test]
fn open_the_tone_browser_opens_it_for_the_lane() {
    let (mut driver, _) = archetype_driver("main-stereo");
    let _ = driver.click("Chain: Roland JC-40").expect("lane B block");
    driver.settle();
    open_menu(&mut driver);
    let _ = driver.click("OPEN THE TONE BROWSER").expect("link");
    driver.settle();
    assert!(driver.chrome_mut().browser_mut().is_some());
    let target = driver
        .chrome_mut()
        .load_flow_mut()
        .pending_target()
        .cloned()
        .expect("a target");
    assert_eq!(target.side, ronie_core::ChainSide::Right);
    assert!(!driver.is_visible("PEDAL"), "the menu closed");
}

#[test]
fn a_long_chain_scrolls_instead_of_overlapping() {
    let (driver, _) = edited_driver("main-mono", |chain| {
        let template = chain["chain"][1].clone();
        let mut blocks = Vec::new();
        for index in 0..14 {
            let mut block = template.clone();
            block["blockId"] = format!("long-{index}").into();
            block["tone"]["title"] = format!("Block {index}").into();
            blocks.push(block);
        }
        blocks.push(chain["chain"][3].clone());
        chain["chain"] = blocks.into();
    });
    let strip = eframe::egui::Rect::from_min_max(
        eframe::egui::pos2(260.0, 0.0),
        eframe::egui::pos2(644.0, 64.0),
    );
    let shown: Vec<_> = (0..14)
        .filter_map(|index| rect_of(&driver, &format!("Chain: Block {index}")))
        .collect();
    assert!(!shown.is_empty(), "some blocks show");
    assert!(shown.len() < 14, "not all fourteen fit");
    for rect in &shown {
        assert!(strip.contains_rect(*rect), "{rect:?} leaves the strip");
    }
    assert!(driver.is_visible("Scroll blocks right"));
}

#[test]
fn stereo_lists_lane_a_then_lane_b() {
    let (driver, _) = archetype_driver("main-stereo");
    let left = rect_of(&driver, "Chain: UK Greenback - Marshall 1960TV 4x12 M201").expect("lane A");
    let right = rect_of(&driver, "Chain: Roland JC-40").expect("lane B");
    assert!(left.max.x <= right.min.x, "lane A comes before lane B");
}

#[test]
fn a_long_preset_name_does_not_move_the_arrows() {
    let (short, _) = archetype_driver("main-mono");
    let (long, _) = edited_driver("main-mono", |chain| {
        chain["preset"]["name"] =
            "A very long preset name that goes on and on and on and on".into();
    });
    assert!(rect_of(&short, "Next preset").is_some(), "the arrows show");
    assert_eq!(
        rect_of(&short, "Next preset"),
        rect_of(&long, "Next preset")
    );
    assert_eq!(
        rect_of(&short, "Previous preset"),
        rect_of(&long, "Previous preset")
    );
}

// ---------------------------------------------------------------- Task 4: stage

#[test]
fn the_first_block_opens_by_itself() {
    let (mut driver, _) = archetype_driver("main-mono");
    assert!(
        matches!(
            driver.chrome().gallery().selection(),
            GallerySelection::BlockDetail(id) if *id == BlockId("blk-1".to_owned())
        ),
        "the first block of lane A is selected"
    );
    assert!(
        driver.chrome_mut().detail_mut().is_some(),
        "its detail shows"
    );
}

#[test]
fn an_empty_chain_prompts_to_add_a_block() {
    let (mut driver, _) = archetype_driver("main-empty");
    assert!(driver.shows_text("Your chain is empty"));
    let _ = driver.click("Add your first block").expect("prompt button");
    driver.settle();
    assert!(
        driver.chrome_mut().browser_mut().is_some(),
        "the tone browser opened"
    );
}

#[test]
fn the_tuner_takes_the_stage() {
    let (mut driver, _) = archetype_driver("main-mono");
    driver.chrome_mut().set_tuner(true);
    driver.settle();
    assert!(driver.chrome().tuner_visible());
    assert!(driver.chrome_mut().detail_mut().is_none() || !driver.is_visible("Back"));
    assert!(!driver.shows_text("Your chain is empty"));
}

// ---------------------------------------------------------------- Task 5: panel and utility bar

fn faceplate_shapes(look: Look) -> String {
    use eframe::egui;
    let backend = Arc::new(MockBackend::from_scenario(&scenario("main-mono")));
    let mut faceplate = ronie_ui::default::faceplate::Faceplate::new(backend);
    let context = egui::Context::default();
    ronie_ui::fonts::Fonts::install(&context);
    Look::install(&context, look);
    let input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(1024.0, 578.0),
        )),
        ..Default::default()
    };
    let mut output = context.run_ui(input, |ui| {
        egui::Panel::bottom("faceplate")
            .exact_size(108.0)
            .show(ui, |ui| faceplate.show(ui));
    });
    output.textures_delta.clear();
    output
        .shapes
        .iter()
        .map(|clipped| format!("{:?}", clipped.shape))
        .collect::<Vec<_>>()
        .join("\n")
}

fn color_text(color: eframe::egui::Color32) -> String {
    format!("{color:?}")
}

#[test]
fn faceplate_uses_the_light_panel_in_archetype_mode() {
    let look = Look::archetype();
    let shapes = faceplate_shapes(look);
    assert!(
        shapes.contains(&color_text(look.panel)),
        "the panel fill is light"
    );
    assert!(
        shapes.contains(&color_text(look.text_muted)),
        "captions use the muted text colour"
    );
    let reference = faceplate_shapes(Look::reference());
    assert!(
        !reference.contains(&color_text(look.panel)),
        "the reference stays dark"
    );
}

#[test]
fn utility_tuner_toggles_the_tuner() {
    let (mut driver, _) = archetype_driver("main-mono");
    let _ = driver.click("Tuner").expect("tuner");
    driver.settle();
    assert!(driver.chrome().tuner_visible());
}

#[test]
fn utility_settings_opens_settings() {
    let (mut driver, _) = archetype_driver("main-mono");
    let _ = driver.click("Settings").expect("settings");
    driver.settle();
    assert!(driver.chrome_mut().settings_mut().is_some());
}

#[test]
fn utility_mono_stereo_switch_modes() {
    let (mut driver, backend) = archetype_driver("main-mono");
    let _ = driver.click("Stereo mode").expect("stereo");
    driver.settle();
    assert!(backend.chain_state(0).chain.stereo_enabled);
    let _ = driver.click("Mono mode").expect("mono");
    driver.settle();
    assert!(!backend.chain_state(0).chain.stereo_enabled);
}

#[test]
fn utility_undo_redo_follow_history() {
    let (mut driver, backend) = archetype_driver("main-mono");
    let _ = driver.secondary_click(VOX).expect("bypass a block");
    driver.settle();
    assert!(!backend.chain_state(0).chain.left[1].params.enabled);
    let _ = driver.click("Undo").expect("undo");
    driver.settle();
    assert!(
        backend.chain_state(0).chain.left[1].params.enabled,
        "undo restored it"
    );
    let _ = driver.click("Redo").expect("redo");
    driver.settle();
    assert!(
        !backend.chain_state(0).chain.left[1].params.enabled,
        "redo bypassed it again"
    );
}

#[test]
fn disabled_controls_fade_into_the_light_panel() {
    let veil = color_text(ronie_ui::theme::palette::KNOB_VEIL);
    assert!(
        faceplate_shapes(Look::reference()).contains(&veil),
        "the reference dims with the dark veil"
    );
    assert!(
        !faceplate_shapes(Look::archetype()).contains(&veil),
        "no dark veil on the light panel"
    );
}

// ---------------------------------------------------------------- Task 7: review fixes

/// Runs `frames` frames of the whole window for `chrome` and returns the last frame's shapes.
fn window_shapes(chrome: &mut ChromeController, frames: usize) -> Vec<eframe::egui::Shape> {
    use eframe::egui;
    let context = egui::Context::default();
    ronie_ui::fonts::Fonts::install(&context);
    let mut shapes = Vec::new();
    for _ in 0..frames {
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1024.0, 658.0),
            )),
            ..Default::default()
        };
        let mut output = context.run_ui(input, |ui| {
            egui::CentralPanel::default()
                .frame(egui::Frame::NONE)
                .show(ui, |ui| ronie_ui::default::show(ui, chrome));
        });
        output.textures_delta.clear();
        shapes = output
            .shapes
            .into_iter()
            .map(|clipped| clipped.shape)
            .collect();
    }
    shapes
}

fn flat(shapes: Vec<eframe::egui::Shape>) -> Vec<eframe::egui::Shape> {
    let mut out = Vec::new();
    for shape in shapes {
        match shape {
            eframe::egui::Shape::Vec(inner) => out.extend(flat(inner)),
            other => out.push(other),
        }
    }
    out
}

#[test]
fn the_background_leaves_the_banner_visible() {
    use eframe::egui::Shape;
    let fixture = scenario("banner-no-input");
    let backend = Arc::new(MockBackend::from_scenario(&fixture));
    let mut chrome = ChromeController::new(backend, Vec::new(), false, true);
    chrome.set_view_mode(ViewMode::RonieFlow);
    let device = ronie_ui::default::SystemDeviceState::from_ui_value(&fixture.state["device"])
        .expect("device settings")
        .banner;
    chrome.set_device_state(Some(device), 0);
    let shapes = flat(window_shapes(&mut chrome, 3));
    let background = Look::archetype().background;
    let banner_drawn = shapes.iter().any(|shape| {
        matches!(shape, Shape::Rect(rect) if rect.rect.min.y < 1.0 && (40.0..=46.0).contains(&rect.rect.height()) && rect.fill == eframe::egui::Color32::BLACK)
    });
    assert!(banner_drawn, "the scenario shows a banner");
    let covered = shapes.iter().any(|shape| {
        matches!(shape, Shape::Rect(rect) if rect.fill == background && rect.rect.min.y < 40.0)
    });
    assert!(!covered, "no light fill is painted over the banner");
}

#[test]
fn removing_a_block_opens_its_neighbour_without_a_flash() {
    let (mut driver, _) = archetype_driver("main-mono");
    let _ = driver.click("REMOVE").expect("remove");
    driver.frame();
    assert!(
        !driver.is_visible("Add Tone"),
        "the chain gallery never shows on the stage"
    );
    driver.settle();
    assert!(
        matches!(
            driver.chrome().gallery().selection(),
            GallerySelection::BlockDetail(id) if *id == BlockId("blk-2".to_owned())
        ),
        "the next block opens: {:?}",
        driver.chrome().gallery().selection()
    );
}

#[test]
fn removing_the_last_block_opens_the_previous_one() {
    let (mut driver, _) = archetype_driver("main-mono");
    let _ = driver
        .click("Chain: UK Greenback - Marshall 1960TV 4x12 M201")
        .expect("third block");
    driver.settle();
    let _ = driver.click("REMOVE").expect("remove");
    for _ in 0..3 {
        assert!(
            !driver.is_visible("Add Tone"),
            "the chain gallery never shows on the stage"
        );
        driver.frame();
    }
    driver.settle();
    assert!(
        matches!(
            driver.chrome().gallery().selection(),
            GallerySelection::BlockDetail(id) if *id == BlockId("blk-2".to_owned())
        ),
        "the previous block opens: {:?}",
        driver.chrome().gallery().selection()
    );
}

#[test]
fn a_second_click_on_the_preset_pill_closes_the_browser() {
    use ronie_ui::default::Overlay;
    let (mut driver, _) = archetype_driver("main-mono");
    let _ = driver.click("Browse presets").expect("pill");
    driver.settle();
    assert_eq!(driver.chrome().overlay(), Overlay::PresetBrowser);
    let _ = driver.click("Browse presets").expect("pill again");
    driver.settle();
    assert_eq!(
        driver.chrome().overlay(),
        Overlay::None,
        "the second click closes it"
    );
}

#[test]
fn deck_captions_stay_light_on_the_dark_deck() {
    use eframe::egui::{self, Shape};
    let backend = Arc::new(MockBackend::from_scenario(&scenario("main-mono")));
    let mut faceplate = ronie_ui::default::faceplate::Faceplate::new(backend);
    faceplate.open_deck(ronie_ui::default::DeckKind::Gate);
    let context = egui::Context::default();
    ronie_ui::fonts::Fonts::install(&context);
    Look::install(&context, Look::archetype());
    let mut texts = Vec::new();
    for _ in 0..2 {
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1024.0, 578.0),
            )),
            ..Default::default()
        };
        let mut output = context.run_ui(input, |ui| {
            egui::Panel::bottom("faceplate")
                .exact_size(108.0)
                .show(ui, |ui| faceplate.show(ui));
        });
        output.textures_delta.clear();
        texts = flat(
            output
                .shapes
                .into_iter()
                .map(|clipped| clipped.shape)
                .collect(),
        )
        .into_iter()
        .filter_map(|shape| match shape {
            Shape::Text(text) if text.pos.y < 470.0 => {
                Some((text.galley.text().to_owned(), text.fallback_color))
            }
            _ => None,
        })
        .collect();
    }
    assert!(!texts.is_empty(), "the deck shows above the panel");
    for (text, color) in &texts {
        // Unmultiplied, so that a dimmed caption is judged by its own colour.
        let [r, g, b, _] = color.to_srgba_unmultiplied();
        let brightest = r.max(g).max(b);
        assert!(
            brightest >= 0x80,
            "{text} is dark ({color:?}) on the dark deck"
        );
    }
}

#[test]
fn the_block_you_add_opens_on_the_stage() {
    let (mut driver, backend) = archetype_driver("main-mono");
    let _ = driver.click("Add a block").expect("plus");
    driver.settle();
    let _ = driver.click("OPEN THE TONE BROWSER").expect("browser link");
    driver.settle();
    let tone = ronie_core::Tone::from_value(serde_json::json!({
        "id": 4242, "title": "Freshly added amp", "format": "nam", "gear": "amp",
        "models": [{"id": 424201, "name": "One", "model_url": "https://fixtures.invalid/m/424201.nam"}]
    }))
    .expect("tone");
    driver.chrome_mut().pick_tone(tone);
    for _ in 0..20 {
        driver.settle();
    }
    let chain = backend.chain_state(0).chain;
    let added = chain
        .left
        .iter()
        .find(|block| {
            block
                .tone
                .as_ref()
                .is_some_and(|tone| tone.title == "Freshly added amp")
        })
        .map(|block| block.id.clone())
        .expect("the tone was added");
    assert!(
        matches!(driver.chrome().gallery().selection(), GallerySelection::BlockDetail(id) if *id == added),
        "the added block opens: {:?}",
        driver.chrome().gallery().selection()
    );
}

#[test]
fn a_loaded_tone_joins_the_library() {
    use ronie_ui::services::RecentTones;
    let (mut driver, backend) = archetype_driver("main-mono");
    let _ = driver.click("Add a block").expect("plus");
    driver.settle();
    let _ = driver.click("OPEN THE TONE BROWSER").expect("browser link");
    driver.settle();
    let tone = ronie_core::Tone::from_value(serde_json::json!({
        "id": 4343, "title": "Library amp", "format": "nam", "gear": "amp",
        "models": [{"id": 434301, "name": "One", "model_url": "https://fixtures.invalid/m/434301.nam"}]
    }))
    .expect("tone");
    driver.chrome_mut().pick_tone(tone);
    for _ in 0..20 {
        driver.settle();
    }
    assert_eq!(
        driver.chrome().recent_tones().tones()[0].title,
        "Library amp"
    );
    let stored = backend
        .text_preference(RecentTones::KEY)
        .expect("the library is stored");
    assert!(stored.contains("Library amp"), "{stored}");
}

// ---------------------------------------------------------------- Task 8: designs

#[test]
fn the_utility_bar_holds_the_designed_items() {
    let (driver, _) = archetype_driver("main-mono");
    for label in [
        "Tuner", "MIDI", "Optimize", "TONE3000", "Settings", "Account",
    ] {
        let rect = rect_of(&driver, label).unwrap_or_else(|| panic!("{label} is missing"));
        assert!(
            rect.min.y >= 578.0 - 1.0,
            "{label} sits in the utility bar: {rect:?}"
        );
    }
}

#[test]
fn optimize_toggles_from_the_utility_bar() {
    let (mut driver, backend) = archetype_driver("main-mono");
    let before = ronie_ui::default::tuning::optimize(backend.as_ref());
    let _ = driver.click("Optimize").expect("optimize");
    driver.settle();
    assert_ne!(
        ronie_ui::default::tuning::optimize(backend.as_ref()),
        before
    );
}

#[test]
fn tone3000_opens_the_tone_browser() {
    let (mut driver, _) = archetype_driver("main-mono");
    let _ = driver.click("TONE3000").expect("tone3000");
    driver.settle();
    assert!(driver.chrome_mut().browser_mut().is_some());
}

#[test]
fn undo_and_redo_sit_in_the_preset_manager() {
    let (driver, _) = archetype_driver("main-mono");
    for label in ["Undo", "Redo"] {
        let rect = rect_of(&driver, label).unwrap_or_else(|| panic!("{label} is missing"));
        assert!(
            rect.max.y <= 64.0 && rect.min.x >= 652.0,
            "{label} sits in the header: {rect:?}"
        );
    }
}

#[test]
fn the_lanes_switch_sits_in_the_global_panel() {
    let (driver, _) = archetype_driver("main-mono");
    for label in ["Mono mode", "Stereo mode"] {
        let rect = rect_of(&driver, label).unwrap_or_else(|| panic!("{label} is missing"));
        assert!(
            (470.0..578.0).contains(&rect.center().y),
            "{label} sits in the panel: {rect:?}"
        );
    }
}

#[test]
fn the_block_page_has_no_back_link() {
    let (driver, _) = archetype_driver("main-mono");
    assert!(driver.chrome().gallery().selection() != &GallerySelection::None);
    assert!(!driver.is_visible("Back to chain"));
}

#[test]
fn plus_adds_to_the_lane_of_the_selected_block() {
    let (mut driver, _) = archetype_driver("main-stereo");
    let _ = driver.click("Chain: Roland JC-40").expect("lane B block");
    driver.settle();
    let _ = driver.click("Add a block").expect("plus");
    driver.settle();
    let _ = driver.click("OPEN THE TONE BROWSER").expect("browser link");
    driver.settle();
    assert!(
        matches!(
            driver.chrome().gallery().selection(),
            GallerySelection::SelectTone {
                side: ronie_core::ChainSide::Right,
                ..
            }
        ) || driver.chrome_mut().browser_mut().is_some()
            && !matches!(
                driver.chrome().gallery().selection(),
                GallerySelection::SelectTone {
                    side: ronie_core::ChainSide::Left,
                    ..
                }
            ),
        "the browser fills lane B: {:?}",
        driver.chrome().gallery().selection()
    );
}

// ---------------------------------------------------------------- Rendered skins

#[test]
fn the_amp_page_shows_the_render_knobs_instead_of_the_trims() {
    let (mut driver, _) = archetype_driver("main-mono");
    let _ = driver
        .click("Chain: '02 Vox AC30/6 Top Boost")
        .expect("amp");
    driver.settle();
    assert!(driver.is_visible("Amp render"));
    for label in ["GAIN IN:", "OUTPUT:", "MIX:"] {
        assert!(
            driver.label_starting_with(label).is_some(),
            "{label} is on the render"
        );
    }
    assert!(
        driver.label_starting_with("In:").is_none(),
        "the In trim gives way to GAIN IN"
    );
}

#[test]
fn dragging_gain_in_on_the_render_raises_the_input_gain() {
    let (mut driver, backend) = archetype_driver("main-mono");
    let _ = driver
        .click("Chain: '02 Vox AC30/6 Top Boost")
        .expect("amp");
    driver.settle();
    let amp = |backend: &MockBackend| {
        backend
            .chain_state(0)
            .chain
            .left
            .iter()
            .find(|block| {
                block
                    .tone
                    .as_ref()
                    .is_some_and(|tone| tone.title.contains("AC30"))
            })
            .expect("amp block")
            .params
            .input_gain
    };
    let before = amp(&backend);
    let up: Vec<_> = (1..=6)
        .map(|step| eframe::egui::vec2(0.0, -10.0 * step as f32))
        .collect();
    let label = driver
        .label_starting_with("GAIN IN:")
        .expect("GAIN IN knob");
    let _ = driver.drag_control(&label, &up).expect("drag");
    assert!(amp(&backend) > before, "input gain rose from {before}");
}

#[test]
fn a_cab_page_shows_its_render_without_knobs() {
    let (mut driver, _) = archetype_driver("main-mono");
    let _ = driver
        .click("Chain: UK Greenback - Marshall 1960TV 4x12 M201")
        .expect("cab");
    driver.settle();
    assert!(driver.is_visible("Cab render"));
    assert!(driver.label_starting_with("GAIN IN:").is_none());
}

#[test]
fn the_reference_layout_keeps_its_trims() {
    let backend = Arc::new(MockBackend::from_scenario(&scenario("main-mono")));
    let chrome = ChromeController::new(backend.clone(), Vec::new(), false, true);
    let mut driver = UiDriver::new(chrome, (*backend).clone());
    driver.settle();
    driver
        .chrome_mut()
        .gallery_mut()
        .activate(&first_tone_id(&backend));
    driver.settle();
    assert!(driver.label_starting_with("In:").is_some());
    assert!(driver.label_starting_with("GAIN IN:").is_none());
}

fn first_tone_id(backend: &MockBackend) -> String {
    backend
        .chain_state(0)
        .chain
        .left
        .iter()
        .find(|block| block.tone.is_some())
        .expect("a tone block")
        .id
        .0
        .clone()
}

// ---------------------------------------------------------------- Light block page

use ronie_ui::default::block::BlockView;

fn amp_page() -> (UiDriver, Arc<MockBackend>) {
    let (mut driver, backend) = archetype_driver("main-mono");
    let _ = driver.click(VOX).expect("amp");
    driver.settle();
    (driver, backend)
}

fn block_titled(backend: &MockBackend, needle: &str) -> Option<ronie_core::ChainBlock> {
    let state = backend.chain_state(0);
    state
        .chain
        .left
        .iter()
        .chain(state.chain.right.iter())
        .find(|block| {
            block
                .tone
                .as_ref()
                .is_some_and(|tone| tone.title.contains(needle))
        })
        .cloned()
}

#[test]
fn the_block_page_has_a_title_row_and_tabs() {
    let (driver, _) = amp_page();
    for label in ["GEAR", "EQ", "INFO", "BYPASS", "REPLACE", "REMOVE"] {
        assert!(driver.is_visible(label), "{label} is on the page");
    }
    assert!(
        driver.shows_text("AMP · '02 VOX AC30/6 TOP BOOST"),
        "the title row names the block"
    );
    assert!(driver.shows_text("NAM capture from TONE3000 · 6 captures in this tone"));
}

#[test]
fn the_eq_tab_opens_the_eq() {
    let (mut driver, _) = amp_page();
    let _ = driver.click("EQ").expect("EQ tab");
    driver.settle();
    assert_eq!(
        driver.chrome().detail().map(|detail| detail.view()),
        Some(BlockView::Eq)
    );
    let _ = driver.click("GEAR").expect("GEAR tab");
    driver.settle();
    assert_eq!(
        driver.chrome().detail().map(|detail| detail.view()),
        Some(BlockView::Tone)
    );
}

#[test]
fn bypass_toggles_the_block() {
    let (mut driver, backend) = amp_page();
    let _ = driver.click("BYPASS").expect("BYPASS");
    driver.settle();
    assert!(!block_titled(&backend, "AC30").expect("amp").params.enabled);
}

#[test]
fn remove_removes_the_block() {
    let (mut driver, backend) = amp_page();
    let _ = driver.click("REMOVE").expect("REMOVE");
    driver.settle();
    assert!(block_titled(&backend, "AC30").is_none());
}

#[test]
fn replace_opens_the_browser_for_the_block() {
    let (mut driver, _) = amp_page();
    let _ = driver.click("REPLACE").expect("REPLACE");
    driver.settle();
    assert!(
        matches!(
            driver.chrome().gallery().selection(),
            GallerySelection::SelectTone { .. }
        ) || driver.chrome_mut().browser_mut().is_some(),
        "the browser opens to replace the block"
    );
}

#[test]
fn the_amp_render_fills_the_designed_stage_height() {
    let (driver, _) = amp_page();
    let render = rect_of(&driver, "Amp render").expect("render");
    assert!(
        (240.0..=300.0).contains(&render.height()),
        "the render is {render:?}"
    );
}

#[test]
fn render_knobs_have_a_comfortable_hit_target() {
    let (driver, _) = amp_page();
    for prefix in ["GAIN IN:", "OUTPUT:", "MIX:"] {
        let label = driver.label_starting_with(prefix).expect("knob");
        let rect = rect_of(&driver, &label).expect("rect");
        assert!(
            rect.width() >= 28.0 && rect.height() >= 28.0,
            "{prefix} is {rect:?}"
        );
    }
}

#[test]
fn a_focused_render_knob_shows_its_value() {
    let (mut driver, _) = amp_page();
    for _ in 0..60 {
        if driver
            .focused()
            .is_some_and(|label| label.starts_with("GAIN IN"))
        {
            break;
        }
        let _ = driver.press_key(eframe::egui::Key::Tab);
    }
    assert!(
        driver
            .focused()
            .is_some_and(|label| label.starts_with("GAIN IN")),
        "Tab reaches GAIN IN"
    );
    driver.unhover();
    driver.settle();
    assert!(
        driver.shows_text("GAIN IN  0.0 dB"),
        "the value hint shows for the focused knob"
    );
}

#[test]
fn a_cab_page_keeps_its_level_and_mix_trims() {
    let (mut driver, _) = archetype_driver("main-mono");
    let _ = driver
        .click("Chain: UK Greenback - Marshall 1960TV 4x12 M201")
        .expect("cab");
    driver.settle();
    for prefix in ["In:", "Mix:", "Out:"] {
        assert!(
            driver.label_starting_with(prefix).is_some(),
            "{prefix} trim"
        );
    }
}

#[test]
fn normalize_toggles_from_the_gear_page() {
    let (mut driver, backend) = amp_page();
    backend
        .dispatch(ronie_ui::backend::UiAction::SetPreference {
            key: "t3k.showBlockNormalizeControl".to_owned(),
            value: true,
        })
        .expect("show normalize");
    driver.settle();
    assert!(
        block_titled(&backend, "AC30")
            .expect("amp")
            .params
            .normalize
    );
    let _ = driver.click("NORMALIZE").expect("NORMALIZE");
    driver.settle();
    assert!(
        !block_titled(&backend, "AC30")
            .expect("amp")
            .params
            .normalize
    );
}

// ---------------------------------------------------------------- Pedals share the stage

const OD: &str = "TS808 Tube Screamer Overdrive";

/// main-mono with a green overdrive right after the fuzz.
fn pedalboard_driver() -> (UiDriver, Arc<MockBackend>) {
    let mut scenario = scenario("main-mono");
    let chain = scenario.state["chain"]["chain"]
        .as_array_mut()
        .expect("chain");
    let mut od = chain[0].clone();
    od["blockId"] = "blk-od".into();
    od["activeModelId"] = 1199.into();
    od["tone"]["id"] = 199.into();
    od["tone"]["title"] = OD.into();
    od["tone"]["models"] = serde_json::json!([{ "id": 1199, "name": "Drive 5" }]);
    chain.insert(1, od);
    let backend = Arc::new(MockBackend::from_scenario(&scenario));
    let mut chrome = ChromeController::new(backend.clone(), Vec::new(), false, true);
    chrome.set_view_mode(ViewMode::RonieFlow);
    let mut driver = UiDriver::new(chrome, (*backend).clone());
    driver.settle();
    (driver, backend)
}

#[test]
fn consecutive_pedals_share_the_stage() {
    let (driver, _) = pedalboard_driver();
    assert!(driver.is_visible("Pedal render"), "the open pedal");
    assert!(
        driver.is_visible(&format!("Pedal render: {OD}")),
        "its neighbour shares the stage"
    );
    assert!(
        driver.shows_text("PEDALS · GREEN RUSSIAN BIG MUFF + BD-2 → TS808 TUBE SCREAMER OVERDRIVE")
    );
}

#[test]
fn clicking_a_neighbour_pedal_selects_it() {
    let (mut driver, _) = pedalboard_driver();
    let _ = driver
        .click(&format!("Pedal render: {OD}"))
        .expect("neighbour");
    driver.settle();
    assert_eq!(
        driver.chrome().gallery().selection(),
        &GallerySelection::BlockDetail(BlockId("blk-od".to_owned()))
    );
    assert!(
        driver.is_visible("Pedal render: Green Russian Big Muff + BD-2"),
        "the fuzz is now the neighbour"
    );
}

#[test]
fn an_amp_shares_the_stage_with_no_one() {
    let (mut driver, _) = pedalboard_driver();
    let _ = driver.click(VOX).expect("amp");
    driver.settle();
    assert!(driver.label_starting_with("Pedal render").is_none());
}

// ---------------------------------------------------------------- Light EQ and INFO

fn eq_block(backend: &MockBackend) -> ronie_core::EqParams {
    block_titled(backend, "AC30").expect("amp").params.eq
}

#[test]
fn the_eq_page_has_band_chips_power_and_pre_post() {
    let (mut driver, backend) = amp_page();
    let _ = driver.click("EQ").expect("EQ tab");
    driver.settle();
    for band in 1..=6 {
        assert!(
            driver.is_visible(&format!("Select band {band}")),
            "band {band}"
        );
    }
    let _ = driver.click("Select band 3").expect("band 3");
    driver.settle();
    assert_eq!(
        driver
            .chrome()
            .detail()
            .map(|detail| detail.eq().selected()),
        Some(2)
    );
    assert!(!eq_block(&backend).enabled);
    let _ = driver.click("EQ power").expect("power");
    driver.settle();
    assert!(eq_block(&backend).enabled, "the EQ turns on");
    assert!(!eq_block(&backend).pre);
    let _ = driver.click("PRE / POST").expect("pre/post");
    driver.settle();
    assert!(eq_block(&backend).pre, "the EQ moves before the block");
}

#[test]
fn the_eq_page_shows_the_curve() {
    let (mut driver, _) = amp_page();
    let _ = driver.click("EQ").expect("EQ tab");
    driver.settle();
    assert!(driver.is_visible("Band 1 frequency and gain"));
}

#[test]
fn the_info_page_shows_the_tone_and_its_page_link() {
    let (mut driver, _) = amp_page();
    let _ = driver.click("INFO").expect("INFO tab");
    driver.settle();
    assert!(driver.shows_text("'02 Vox AC30/6 Top Boost"));
    assert!(driver.is_visible("Open on TONE3000"));
}

#[test]
fn the_size_toggle_keeps_one_name() {
    let (mut driver, backend) = amp_page();
    backend
        .dispatch(ronie_ui::backend::UiAction::SetPreference {
            key: "t3k.showBlockSizeControl".to_owned(),
            value: true,
        })
        .expect("show size");
    driver.settle();
    let _ = driver.click("Full size").expect("size toggle");
    driver.settle();
    assert!(driver.is_visible("Full size"), "the name survives the flip");
}
