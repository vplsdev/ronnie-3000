//! The Archetype layout: light header with the chain as gear icons, stage, light global panel
//! and black utility bar.

use std::sync::Arc;

use ronie_ui::{
    backend::{MockBackend, UiBackend},
    default::{ChromeController, Placement},
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
    assert!(
        driver.is_visible("OPEN THE TONE BROWSER"),
        "the menu is open"
    );
    let _ = driver.click("Add a block").expect("plus");
    driver.settle();
    assert!(!driver.is_visible("PEDAL"), "the second click closes it");
    assert!(
        !driver.is_visible("OPEN THE TONE BROWSER"),
        "the second click closes it"
    );
    open_menu(&mut driver);
    assert!(
        driver.is_visible("OPEN THE TONE BROWSER"),
        "the menu is open"
    );
    let _ = driver.press_key(eframe::egui::Key::Escape);
    driver.settle();
    assert!(!driver.is_visible("PEDAL"), "escape closes it");
    assert!(
        !driver.is_visible("OPEN THE TONE BROWSER"),
        "escape closes it"
    );
}

#[test]
fn clicking_outside_closes_the_menu() {
    let (mut driver, _) = archetype_driver("main-mono");
    open_menu(&mut driver);
    assert!(
        driver.is_visible("OPEN THE TONE BROWSER"),
        "the menu is open"
    );
    let _ = driver.click_empty_space();
    driver.settle();
    assert!(!driver.is_visible("PEDAL"));
    assert!(
        !driver.is_visible("OPEN THE TONE BROWSER"),
        "outside click closes it"
    );
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

fn add_beside(
    placement: impl FnOnce(&ronie_core::ChainSnapshot) -> Placement,
) -> (Vec<String>, String) {
    let (mut driver, backend) = pedalboard_driver();
    let chain = backend.chain_state(0).chain;
    let insert = chain
        .left
        .iter()
        .find(|block| block.is_insert)
        .map(|block| block.id.0.clone())
        .expect("insert");
    let tone = ronie_core::Tone::from_value(serde_json::json!({
        "id": 4243, "title": "Placed amp", "format": "nam", "gear": "amp",
        "models": [{"id": 424301, "name": "One", "model_url": "https://fixtures.invalid/m/424301.nam"}]
    }))
    .expect("tone");
    let placement = placement(&chain);
    driver
        .chrome_mut()
        .add_tone(ronie_core::ChainSide::Left, &insert, tone, placement);
    for _ in 0..20 {
        driver.settle();
    }
    let titles = backend
        .chain_state(0)
        .chain
        .left
        .iter()
        .filter_map(|block| block.tone.as_ref().map(|tone| tone.title.clone()))
        .collect();
    (titles, "Placed amp".to_owned())
}

#[test]
fn a_tone_added_after_a_block_lands_after_it() {
    let (titles, added) = add_beside(|chain| Placement::After(chain.left[0].id.clone()));
    assert_eq!(titles.len(), 5, "{titles:?}");
    assert_eq!(titles[1], added, "{titles:?}");
}

#[test]
fn a_vanished_anchor_falls_back_to_the_end() {
    let (titles, added) = add_beside(|_| Placement::After(ronie_core::BlockId("gone".to_owned())));
    assert_eq!(titles.len(), 5, "{titles:?}");
    assert_eq!(titles[4], added, "{titles:?}");
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

// ---------------------------------------------------------------- Slots on the stage

/// main-mono with `extra` more overdrives after the fuzz, so the pedals stage is crowded.
fn crowded_pedalboard(extra: usize) -> UiDriver {
    let mut scenario = scenario("main-mono");
    let chain = scenario.state["chain"]["chain"]
        .as_array_mut()
        .expect("chain");
    for index in 0..extra {
        let mut od = chain[0].clone();
        od["blockId"] = format!("blk-od{index}").into();
        od["tone"]["id"] = (200 + index).into();
        od["tone"]["title"] = format!("Overdrive {index}").into();
        chain.insert(1 + index, od);
    }
    let backend = Arc::new(MockBackend::from_scenario(&scenario));
    let mut chrome = ChromeController::new(backend.clone(), Vec::new(), false, true);
    chrome.set_view_mode(ViewMode::RonieFlow);
    let mut driver = UiDriver::new(chrome, (*backend).clone());
    driver.settle();
    driver
}

#[test]
fn a_pedal_run_ends_with_a_pedal_slot() {
    let (driver, _) = pedalboard_driver();
    let slot = rect_of(&driver, "Add a pedal after these").expect("the slot");
    let last = rect_of(&driver, &format!("Pedal render: {OD}")).expect("last pedal");
    assert!(slot.left() >= last.right() + 17.9, "{slot:?} vs {last:?}");
    assert!((slot.center().y - last.center().y).abs() < 1.0);
    // 110 x 176 against a 300 pt render, scaled with the render.
    let scale = last.height() / 300.0;
    assert!((slot.width() - 110.0 * scale).abs() < 0.1, "{slot:?}");
    assert!((slot.height() - 176.0 * scale).abs() < 0.1, "{slot:?}");
}

#[test]
fn an_amp_has_a_pedal_slot_before_it() {
    let (mut driver, _) = archetype_driver("main-mono");
    let _ = driver.click(VOX).expect("amp");
    driver.settle();
    let slot = rect_of(&driver, "Add a pedal before the amp").expect("the slot");
    let amp = rect_of(&driver, "Amp render").expect("amp");
    assert!(slot.right() <= amp.left() - 17.9, "{slot:?} vs {amp:?}");
    let scale = amp.height() / 300.0;
    assert!((slot.width() - 96.0 * scale).abs() < 0.1, "{slot:?}");
    assert!((slot.height() - 160.0 * scale).abs() < 0.1, "{slot:?}");
    assert!(rect_of(&driver, "Add a pedal after these").is_none());
}

#[test]
fn an_amp_followed_by_a_cab_has_no_cab_slot() {
    let (mut driver, _) = archetype_driver("main-mono");
    let _ = driver.click(VOX).expect("amp");
    driver.settle();
    assert!(rect_of(&driver, "Add a cab after the amp").is_none());
}

#[test]
fn an_amp_with_no_cab_after_it_has_a_cab_slot() {
    let (mut driver, _) = edited_driver("main-mono", |chain| {
        chain["chain"].as_array_mut().expect("chain").remove(2);
    });
    let _ = driver.click(VOX).expect("amp");
    driver.settle();
    let slot = rect_of(&driver, "Add a cab after the amp").expect("the slot");
    let amp = rect_of(&driver, "Amp render").expect("amp");
    assert!(slot.left() >= amp.right() + 17.9, "{slot:?} vs {amp:?}");
    let scale = amp.height() / 300.0;
    assert!((slot.width() - 96.0 * scale).abs() < 0.1, "{slot:?}");
    assert!((slot.height() - 160.0 * scale).abs() < 0.1, "{slot:?}");
}

#[test]
fn a_cab_page_has_no_slots() {
    let (mut driver, _) = archetype_driver("main-mono");
    let _ = driver
        .click("Chain: UK Greenback - Marshall 1960TV 4x12 M201")
        .expect("cab");
    driver.settle();
    for name in [
        "Add a pedal after these",
        "Add a pedal before the amp",
        "Add a cab after the amp",
    ] {
        assert!(rect_of(&driver, name).is_none(), "{name}");
    }
}

#[test]
fn slots_stay_visible_in_a_narrow_window() {
    let roomy = rect_of(&pedalboard_driver().0, "Pedal render").expect("roomy pedal");
    let mut driver = crowded_pedalboard(4);
    let slot = rect_of(&driver, "Add a pedal after these").expect("the slot");
    let first = rect_of(&driver, "Pedal render").expect("open pedal");
    assert!(
        first.height() < roomy.height(),
        "the renders shrank: {first:?} vs {roomy:?}"
    );
    assert!(
        slot.right() <= 1024.0 - 56.0,
        "{slot:?} is inside the stage"
    );
    let scale = first.height() / 300.0;
    assert!((slot.height() - 176.0 * scale).abs() < 0.1, "{slot:?}");
    let _ = driver.click("Add a pedal after these").expect("clickable");
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

// ---------------------------------------------------------------- Add menu results

const AMP_SEARCH: &str = "Search amps in your library and on TONE3000";

fn amp_tone(id: i64, title: &str) -> ronie_core::Tone {
    ronie_core::Tone::from_value(serde_json::json!({
        "id": id, "title": title, "format": "nam", "gear": "amp",
        "downloads_count": 3200, "user": {"username": "lowend_lab"},
        "models": [{"id": id * 10, "name": "m",
            "model_url": format!("https://fixtures.invalid/m/{id}.nam")}]
    }))
    .expect("tone")
}

/// A driver whose library holds `library`, newest first, and whose user is signed in or not.
fn menu_driver(
    id: &str,
    signed_in: bool,
    library: &[ronie_core::Tone],
) -> (UiDriver, Arc<MockBackend>) {
    use ronie_ui::{backend::UiAction, services::recent_tones::RecentTones};
    let mut recent = RecentTones::default();
    for tone in library.iter().rev() {
        recent.remember(tone.clone());
    }
    let backend = Arc::new(MockBackend::from_scenario(&scenario(id)));
    backend
        .dispatch(UiAction::SetTextPreference {
            key: RecentTones::KEY.to_owned(),
            value: recent.to_text(),
        })
        .expect("seed");
    let mut chrome = ChromeController::new(backend.clone(), Vec::new(), signed_in, true);
    chrome.set_view_mode(ViewMode::RonieFlow);
    let mut driver = UiDriver::new(chrome, (*backend).clone());
    driver.settle();
    (driver, backend)
}

fn open_amp(driver: &mut UiDriver) {
    open_menu(driver);
    let _ = driver.click("AMP").expect("AMP row");
    driver.settle();
}

/// Answers the query of the menu store with `tones`, out of `total` tones.
fn answer_menu(driver: &mut UiDriver, tones: Vec<ronie_core::Tone>, total: u64) {
    let store = driver.chrome_mut().add_store_mut().expect("the menu store");
    let generation = store.begin_query();
    let completed = store.complete_query(
        generation,
        ronie_core::Page {
            data: tones,
            page: 1,
            page_size: 12,
            total,
            total_pages: total.div_ceil(12),
        },
        false,
    );
    assert!(completed);
    driver.settle();
}

fn lane_titles(backend: &MockBackend, right: bool) -> Vec<String> {
    let chain = backend.chain_state(0).chain;
    let lane = if right { &chain.right } else { &chain.left };
    lane.iter()
        .filter(|block| !block.is_insert)
        .filter_map(|block| block.tone.as_ref().map(|tone| tone.title.clone()))
        .collect()
}

#[test]
fn a_category_shows_library_and_tone3000() {
    let (mut driver, _) = menu_driver("main-mono", true, &[]);
    open_amp(&mut driver);
    for text in ["YOUR LIBRARY", "TONE3000", "TRENDING"] {
        assert!(driver.shows_text(text), "{text} is missing");
    }
    assert!(driver.is_visible("Back to categories"));
    assert!(driver.is_visible(AMP_SEARCH));
    assert!(
        driver.shows_text("Nothing yet. Tones you load show up here."),
        "an empty library says so"
    );
    assert!(!driver.is_visible("PEDAL"), "the categories gave way");
}

#[test]
fn tone3000_rows_come_from_the_menu_store() {
    let (mut driver, _) = menu_driver("main-mono", true, &[]);
    open_amp(&mut driver);
    let store = driver.chrome_mut().add_store_mut().expect("the menu store");
    assert_eq!(store.query().gear, "amp");
    answer_menu(
        &mut driver,
        vec![
            amp_tone(501, "5150 III Red"),
            amp_tone(502, "Friedman BE-100"),
            amp_tone(503, "Twin Reverb Clean"),
        ],
        1284,
    );
    for title in ["5150 III Red", "Friedman BE-100", "Twin Reverb Clean"] {
        assert!(driver.is_visible(title), "{title} is missing");
    }
    assert!(driver.shows_text("lowend_lab \u{b7} 3.2k \u{2193}"));
    assert!(driver.is_visible("SHOW ALL 1,284 AMPS IN THE TONE BROWSER"));
}

#[test]
fn adding_a_library_tone_puts_it_at_the_end_of_the_lane() {
    let (mut driver, backend) = menu_driver("main-mono", false, &[amp_tone(601, "Plexi Lead")]);
    open_amp(&mut driver);
    assert!(driver.shows_text("Downloaded \u{b7} 1 capture"));
    let _ = driver.click("Plexi Lead").expect("library row");
    driver.settle();
    let titles = lane_titles(&backend, false);
    assert_eq!(
        titles.last().map(String::as_str),
        Some("Plexi Lead"),
        "{titles:?}"
    );
    assert!(!driver.is_visible("Back to categories"), "the menu closed");
    let added = backend
        .chain_state(0)
        .chain
        .left
        .iter()
        .find(|block| {
            block
                .tone
                .as_ref()
                .is_some_and(|tone| tone.title == "Plexi Lead")
        })
        .map(|block| block.id.clone())
        .expect("the block");
    assert!(
        matches!(driver.chrome().gallery().selection(), GallerySelection::BlockDetail(id) if *id == added),
        "the new block is on the stage: {:?}",
        driver.chrome().gallery().selection()
    );
    assert!(driver.chrome_mut().browser_mut().is_none());
}

#[test]
fn stereo_adds_to_the_chosen_lane() {
    let (mut driver, backend) = menu_driver("main-stereo", false, &[amp_tone(602, "Lane B Amp")]);
    open_amp(&mut driver);
    let _ = driver.click("LANE B").expect("lane B");
    driver.settle();
    let _ = driver.click("Lane B Amp").expect("library row");
    driver.settle();
    assert!(lane_titles(&backend, true).contains(&"Lane B Amp".to_owned()));
    assert!(!lane_titles(&backend, false).contains(&"Lane B Amp".to_owned()));
}

#[test]
fn a_tone_already_in_the_chain_reads_in_chain() {
    let backend = MockBackend::from_scenario(&scenario("main-mono"));
    let vox = backend
        .chain_state(0)
        .chain
        .left
        .iter()
        .find_map(|block| block.tone.clone().filter(|tone| tone.title.contains("Vox")))
        .expect("the Vox tone");
    let (mut driver, _) = menu_driver("main-mono", false, &[vox, amp_tone(603, "Not Loaded")]);
    open_amp(&mut driver);
    assert!(driver.shows_text("IN CHAIN"));
    assert!(driver.shows_text("ADD"), "the other tone can be added");
}

#[test]
fn searching_filters_the_library() {
    let (mut driver, _) = menu_driver(
        "main-mono",
        true,
        &[amp_tone(604, "JCM800 Crunch"), amp_tone(605, "Twin Clean")],
    );
    open_amp(&mut driver);
    assert!(driver.is_visible("Twin Clean"));
    let _ = driver.fill(AMP_SEARCH, "jcm").expect("search");
    assert!(driver.is_visible("JCM800 Crunch"));
    assert!(!driver.is_visible("Twin Clean"));
    let text = driver
        .chrome_mut()
        .add_store_mut()
        .map(|store| store.query().text.clone());
    assert_eq!(text.as_deref(), Some("jcm"));
}

/// A build with no key yet, whose sign-in opens the key page.
struct NoKey;

impl ronie_ui::services::key_setup::KeySetup for NoKey {
    fn origin(&self) -> String {
        "https://www.fixtures.invalid".to_owned()
    }

    fn has_saved_key(&self) -> bool {
        false
    }

    fn save(
        &mut self,
        _key: &str,
    ) -> Result<
        Option<ronie_ui::services::key_setup::CommunityParts>,
        ronie_ui::services::key_setup::KeySetupError,
    > {
        Ok(None)
    }

    fn remove(
        &mut self,
    ) -> Result<
        Option<ronie_ui::services::key_setup::CommunityParts>,
        ronie_ui::services::key_setup::KeySetupError,
    > {
        Ok(None)
    }
}

#[test]
fn signed_out_shows_the_sign_in_box() {
    let (mut driver, _) = menu_driver("main-mono", false, &[amp_tone(606, "Kept Amp")]);
    driver.chrome_mut().use_key_setup(Box::new(NoKey));
    open_amp(&mut driver);
    assert!(driver.is_visible("SIGN IN"));
    assert!(driver.is_visible("Search amps in your library"));
    assert!(
        driver.shows_text("Sign in to search and download from the TONE3000 community library.")
    );
    assert!(driver.is_visible("Kept Amp"), "the library stays");
    assert!(!driver.shows_text("TRENDING"));
    assert!(!driver.chrome().auth_active());
    let _ = driver.click("SIGN IN").expect("sign in");
    driver.settle();
    // A build with no key starts its sign-in on the key page, as the browser's button does.
    assert!(driver.chrome().key_page().is_some(), "the sign-in starts");
    assert!(!driver.shows_text("YOUR LIBRARY"), "the menu closed");
}

#[test]
fn signing_out_while_results_show_turns_tone3000_into_the_sign_in_box() {
    let (mut driver, _) = menu_driver("main-mono", true, &[]);
    open_amp(&mut driver);
    answer_menu(&mut driver, vec![amp_tone(607, "Remote Amp")], 1);
    assert!(driver.is_visible("Remote Amp"));
    driver.chrome_mut().sign_out().expect("sign out");
    driver.frame();
    assert!(driver.is_visible("SIGN IN"));
    assert!(!driver.is_visible("Remote Amp"));
}

#[test]
fn show_all_opens_the_browser_filtered() {
    let (mut driver, _) = menu_driver("main-mono", true, &[]);
    open_amp(&mut driver);
    let _ = driver.fill(AMP_SEARCH, "plexi").expect("search");
    answer_menu(&mut driver, vec![amp_tone(608, "Plexi Remote")], 3);
    let _ = driver
        .click("SHOW ALL 3 AMPS IN THE TONE BROWSER")
        .expect("show all");
    driver.settle();
    let query = driver
        .chrome_mut()
        .browser_mut()
        .map(|browser| browser.store().query().clone())
        .expect("the browser opened");
    assert_eq!(query.gear, "amp");
    assert_eq!(query.text, "plexi");
    assert!(!driver.is_visible("Back to categories"), "the menu closed");
}

#[test]
fn back_returns_to_the_categories() {
    let (mut driver, _) = menu_driver("main-mono", true, &[]);
    open_amp(&mut driver);
    let _ = driver.click("Back to categories").expect("back");
    driver.settle();
    assert!(
        driver.is_visible("OPEN THE TONE BROWSER"),
        "the categories show again"
    );
    assert!(!driver.shows_text("YOUR LIBRARY"));
    assert!(
        driver.chrome_mut().add_store_mut().is_none(),
        "the results are gone"
    );
}

#[test]
fn a_second_add_takes_over_a_download_in_progress() {
    let (mut driver, backend) = menu_driver("main-mono", true, &[]);
    open_amp(&mut driver);
    answer_menu(
        &mut driver,
        vec![amp_tone(701, "Slow Download"), amp_tone(702, "Second Pick")],
        2,
    );
    // The first tone is still downloading.
    let insert = backend
        .chain_state(0)
        .chain
        .left
        .iter()
        .rev()
        .find(|block| block.is_insert)
        .map(|block| block.id.0.clone())
        .expect("insert");
    let flow = driver.chrome_mut().load_flow_mut();
    flow.select(ronie_core::ChainSide::Left, &insert);
    let _ticket = flow
        .begin_remote_load(amp_tone(701, "Slow Download"))
        .expect("load");
    driver.settle();
    assert!(driver.shows_text("DOWNLOADING"));

    let _ = driver.click("Second Pick").expect("second row");
    driver.settle();
    let titles = lane_titles(&backend, false);
    assert!(titles.contains(&"Second Pick".to_owned()), "{titles:?}");
    assert!(!titles.contains(&"Slow Download".to_owned()), "{titles:?}");
    assert_eq!(driver.chrome().loading_tone(), None);
    assert!(!driver.shows_text("DOWNLOADING"));
}

#[test]
fn a_refused_library_load_shows_a_toast_and_keeps_the_menu_and_the_tone() {
    let local = ronie_core::Tone::from_value(serde_json::json!({
        "id": 0, "title": "Collected Stash", "format": "nam", "gear": "amp", "local": true,
        "models": [{"id": 1, "name": "m", "model_url": "file:///stash/collected.nam"}]
    }))
    .expect("tone");
    let (mut driver, backend) = menu_driver("main-mono", false, &[local]);
    backend.refuse_tone_loads(true);
    open_amp(&mut driver);
    assert!(driver.shows_text("Local .nam file"));
    let _ = driver.click("Collected Stash").expect("library row");
    driver.settle();
    assert!(
        !lane_titles(&backend, false).contains(&"Collected Stash".to_owned()),
        "the chain stays as it was"
    );
    assert!(
        driver
            .chrome()
            .visible_toast()
            .is_some_and(|text| text.contains("try again")),
        "the player reads why: {:?}",
        driver.chrome().visible_toast()
    );
    assert!(driver.is_visible("Collected Stash"), "the menu stays open");
    assert!(
        driver
            .chrome()
            .recent_tones()
            .tones()
            .iter()
            .any(|tone| tone.title == "Collected Stash"),
        "the tone stays in the library"
    );
}

// ---------------------------------------------------------------- Add menu: load a file

/// Writes a short mono 16-bit WAV impulse response, a file that the menu's file dialog takes.
fn write_ir(path: &std::path::Path) {
    let samples = [0_i16, 1000, -1000, 0];
    let mut bytes = Vec::from(b"RIFF".as_slice());
    bytes.extend_from_slice(&(36 + samples.len() as u32 * 2).to_le_bytes());
    bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&16_u32.to_le_bytes());
    bytes.extend_from_slice(&1_u16.to_le_bytes());
    bytes.extend_from_slice(&1_u16.to_le_bytes());
    bytes.extend_from_slice(&48_000_u32.to_le_bytes());
    bytes.extend_from_slice(&96_000_u32.to_le_bytes());
    bytes.extend_from_slice(&2_u16.to_le_bytes());
    bytes.extend_from_slice(&16_u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&(samples.len() as u32 * 2).to_le_bytes());
    for sample in samples {
        bytes.extend_from_slice(&sample.to_le_bytes());
    }
    std::fs::write(path, bytes).expect("WAV");
}

fn last_insert(backend: &MockBackend) -> String {
    backend
        .chain_state(0)
        .chain
        .left
        .iter()
        .rev()
        .find(|block| block.is_insert)
        .map(|block| block.id.0.clone())
        .expect("insert")
}

#[test]
fn a_loaded_file_lands_at_the_end_of_the_lane_joins_the_library_and_opens() {
    let (mut driver, backend) = archetype_driver("main-mono");
    let directory = tempfile::tempdir().expect("tempdir");
    let path = directory.path().join("closet-cab.wav");
    write_ir(&path);
    let insert = last_insert(&backend);

    driver
        .chrome_mut()
        .drop_file_into(&insert, &path, Placement::End)
        .expect("the file loads");
    driver.settle();

    let lane = backend.chain_state(0).chain.left.clone();
    let last_tone = lane
        .iter()
        .rev()
        .find(|block| !block.is_insert)
        .expect("a tone block");
    assert_eq!(last_tone.id.0, insert, "the file fills the last insert");
    let title = last_tone.tone.as_ref().map(|tone| tone.title.clone());
    assert!(
        driver
            .chrome()
            .recent_tones()
            .tones()
            .first()
            .is_some_and(|tone| Some(&tone.title) == title.as_ref() && tone.local),
        "the file joins the library"
    );
    assert!(
        matches!(driver.chrome().gallery().selection(), GallerySelection::BlockDetail(id) if id.0 == insert),
        "the new block is on the stage: {:?}",
        driver.chrome().gallery().selection()
    );
}

#[test]
fn a_missing_file_shows_a_toast() {
    let (mut driver, backend) = archetype_driver("main-mono");
    let directory = tempfile::tempdir().expect("tempdir");
    let insert = last_insert(&backend);
    let before = backend.chain_state(0).revision;

    let result = driver.chrome_mut().drop_file_into(
        &insert,
        &directory.path().join("gone.nam"),
        Placement::End,
    );
    driver.settle();

    assert!(result.is_err());
    assert_eq!(backend.chain_state(0).revision, before, "the chain stays");
    assert!(
        driver.chrome().visible_toast().is_some(),
        "the player reads that the file did not load"
    );
}

// ---------------------------------------------------------------- A slot opens the add menu

const PEDAL_SEARCH: &str = "Search pedals in your library";

fn gear_tone(id: i64, title: &str, gear: &str) -> ronie_core::Tone {
    ronie_core::Tone::from_value(serde_json::json!({
        "id": id, "title": title, "format": "nam", "gear": gear,
        "downloads_count": 1200, "user": {"username": "lowend_lab"},
        "models": [{"id": id * 10, "name": "m",
            "model_url": format!("https://fixtures.invalid/m/{id}.nam")}]
    }))
    .expect("tone")
}

/// A driver of scenario `id`, with its chain changed by `edit`, `library` in the library, and
/// its user signed in or not.
fn slot_driver(
    id: &str,
    signed_in: bool,
    library: &[ronie_core::Tone],
    edit: impl FnOnce(&mut Vec<serde_json::Value>),
) -> (UiDriver, Arc<MockBackend>) {
    use ronie_ui::{backend::UiAction, services::recent_tones::RecentTones};
    let mut scenario = scenario(id);
    edit(
        scenario.state["chain"]["chain"]
            .as_array_mut()
            .expect("chain"),
    );
    let mut recent = RecentTones::default();
    for tone in library.iter().rev() {
        recent.remember(tone.clone());
    }
    let backend = Arc::new(MockBackend::from_scenario(&scenario));
    backend
        .dispatch(UiAction::SetTextPreference {
            key: RecentTones::KEY.to_owned(),
            value: recent.to_text(),
        })
        .expect("seed");
    let mut chrome = ChromeController::new(backend.clone(), Vec::new(), signed_in, true);
    chrome.set_view_mode(ViewMode::RonieFlow);
    let mut driver = UiDriver::new(chrome, (*backend).clone());
    driver.settle();
    (driver, backend)
}

/// main-mono with a green overdrive right after the fuzz, as `pedalboard_driver`.
fn add_overdrive(chain: &mut Vec<serde_json::Value>) {
    let mut od = chain[0].clone();
    od["blockId"] = "blk-od".into();
    od["activeModelId"] = 1199.into();
    od["tone"]["id"] = 199.into();
    od["tone"]["title"] = OD.into();
    od["tone"]["models"] = serde_json::json!([{ "id": 1199, "name": "Drive 5" }]);
    chain.insert(1, od);
}

fn plus_is_solid(driver: &UiDriver) -> bool {
    driver
        .controls()
        .iter()
        .find(|control| control.label == "Add a block")
        .expect("the + button")
        .checked
}

/// The menu's outline: its back button sits in its top left corner inside the 8 pt padding, its
/// search field ends 8 + 6 + 8 pt inside its right edge, and its last link row ends 8 pt above
/// its bottom.
fn menu_rect(driver: &UiDriver, search: &str, last_row: &str) -> eframe::egui::Rect {
    let back = rect_of(driver, "Back to categories").expect("the back button");
    let field = rect_of(driver, search).expect("the search field");
    let link = rect_of(driver, last_row).expect("the last row");
    eframe::egui::Rect::from_min_max(
        back.min - eframe::egui::vec2(8.0, 8.0),
        eframe::egui::pos2(field.right() + 22.0, link.bottom() + 8.0),
    )
}

#[test]
fn the_pedal_slot_opens_the_menu_on_pedals() {
    let (mut driver, _) = slot_driver("main-mono", false, &[], add_overdrive);
    open_menu(&mut driver);
    assert!(plus_is_solid(&driver), "the + menu shows the + solid");
    let _ = driver.click("Add a block").expect("plus");
    driver.settle();

    let slot = rect_of(&driver, "Add a pedal after these").expect("the slot");
    let _ = driver.click("Add a pedal after these").expect("slot");
    driver.settle();
    assert!(driver.is_visible("Back to categories"));
    assert!(driver.shows_text("PEDAL"), "the header names the category");
    assert!(
        driver.shows_text("after TS808 Tube Scream\u{2026}"),
        "the header says where the pedal goes"
    );
    assert!(!driver.shows_text("to the end of the chain"));
    assert!(!driver.is_visible("LANE A") && !driver.is_visible("LANE B"));
    assert!(!plus_is_solid(&driver), "the + stays dashed");

    let menu = menu_rect(&driver, PEDAL_SEARCH, "OPEN THE TONE BROWSER");
    assert!(!menu.intersects(slot), "{menu:?} covers {slot:?}");
    assert!(
        menu.right() <= slot.left() - 9.0 || menu.left() >= slot.right() + 9.0,
        "{menu:?} sits beside {slot:?}"
    );
    assert!((menu.width() - 380.0).abs() < 2.0, "{menu:?}");
    assert!(
        (menu.top()..=menu.bottom()).contains(&slot.center().y),
        "{menu:?} is level with {slot:?}"
    );
}

#[test]
fn a_second_click_on_the_slot_closes_its_menu() {
    let (mut driver, _) = slot_driver("main-mono", false, &[], add_overdrive);
    let _ = driver.click("Add a pedal after these").expect("slot");
    driver.settle();
    assert!(driver.is_visible("Back to categories"));
    let _ = driver.click("Add a pedal after these").expect("slot");
    driver.settle();
    assert!(!driver.is_visible("Back to categories"));
}

#[test]
fn the_plus_takes_over_a_slot_menu() {
    let (mut driver, _) = slot_driver("main-mono", false, &[], add_overdrive);
    let _ = driver.click("Add a pedal after these").expect("slot");
    driver.settle();
    open_menu(&mut driver);
    assert!(driver.is_visible("OPEN THE TONE BROWSER"), "the + menu");
    assert!(driver.shows_text("to the end of the chain"));
    assert!(plus_is_solid(&driver));
}

fn library_pedal() -> ronie_core::Tone {
    gear_tone(801, "Library Drive", "pedal")
}

#[test]
fn adding_from_the_pedal_slot_puts_the_pedal_after_the_run() {
    let (mut driver, backend) = slot_driver("main-mono", false, &[library_pedal()], add_overdrive);
    let _ = driver.click("Add a pedal after these").expect("slot");
    driver.settle();
    let _ = driver.click("Library Drive").expect("library row");
    driver.settle();
    let titles = lane_titles(&backend, false);
    assert_eq!(titles.len(), 5, "{titles:?}");
    assert_eq!(titles[1], OD, "{titles:?}");
    assert_eq!(titles[2], "Library Drive", "{titles:?}");
    assert!(!driver.is_visible("Back to categories"), "the menu closed");
    let added = block_titled(&backend, "Library Drive")
        .expect("the block")
        .id;
    assert!(
        matches!(driver.chrome().gallery().selection(), GallerySelection::BlockDetail(id) if *id == added),
        "the new block is on the stage: {:?}",
        driver.chrome().gallery().selection()
    );
}

#[test]
fn adding_from_the_amp_slot_puts_the_pedal_before_the_amp() {
    let (mut driver, backend) = slot_driver("main-mono", false, &[library_pedal()], |_| {});
    let _ = driver.click(VOX).expect("amp");
    driver.settle();
    let _ = driver.click("Add a pedal before the amp").expect("slot");
    driver.settle();
    assert!(driver.shows_text("before '02 Vox AC30/6 To\u{2026}"));
    let _ = driver.click("Library Drive").expect("library row");
    driver.settle();
    let titles = lane_titles(&backend, false);
    assert_eq!(titles.len(), 4, "{titles:?}");
    assert_eq!(titles[1], "Library Drive", "{titles:?}");
    assert!(titles[2].contains("AC30"), "{titles:?}");
}

#[test]
fn the_cab_slot_adds_a_cab_after_the_amp() {
    // A pedal after the amp in place of the cab, so the slot's place differs from the end.
    let (mut driver, backend) = slot_driver(
        "main-mono",
        false,
        &[gear_tone(802, "Library Cab", "cab")],
        |chain| {
            let mut late = chain[0].clone();
            late["blockId"] = "blk-late".into();
            late["tone"]["title"] = "Late Fuzz".into();
            chain[2] = late;
        },
    );
    let _ = driver.click(VOX).expect("amp");
    driver.settle();
    let _ = driver.click("Add a cab after the amp").expect("slot");
    driver.settle();
    assert!(driver.shows_text("after '02 Vox AC30/6 To\u{2026}"));
    assert!(driver.shows_text("CAB"), "the header names the category");
    let _ = driver.click("Library Cab").expect("library row");
    driver.settle();
    let titles = lane_titles(&backend, false);
    assert_eq!(titles.len(), 4, "{titles:?}");
    assert_eq!(titles[2], "Library Cab", "{titles:?}");
    assert_eq!(titles[3], "Late Fuzz", "{titles:?}");
}

#[test]
fn one_undo_removes_a_slot_add() {
    let (mut driver, backend) = slot_driver("main-mono", false, &[library_pedal()], add_overdrive);
    let before = lane_titles(&backend, false);
    let _ = driver.click("Add a pedal after these").expect("slot");
    driver.settle();
    let _ = driver.click("Library Drive").expect("library row");
    driver.settle();
    assert_eq!(lane_titles(&backend, false).len(), before.len() + 1);
    let _ = driver.click("Undo").expect("undo");
    driver.settle();
    assert_eq!(lane_titles(&backend, false), before);
}

#[test]
fn back_from_a_slot_menu_shows_categories_and_keeps_the_placement() {
    let (mut driver, backend) = slot_driver("main-mono", false, &[library_pedal()], add_overdrive);
    let _ = driver.click("Add a pedal after these").expect("slot");
    driver.settle();
    let _ = driver.click("Back to categories").expect("back");
    driver.settle();
    assert!(driver.is_visible("OPEN THE TONE BROWSER"), "the categories");
    assert!(
        driver.shows_text("after TS808 Tube Scream\u{2026}"),
        "the place stays"
    );
    let _ = driver.click("PEDAL").expect("PEDAL row");
    driver.settle();
    let _ = driver.click("Library Drive").expect("library row");
    driver.settle();
    let titles = lane_titles(&backend, false);
    assert_eq!(titles[2], "Library Drive", "{titles:?}");
}

#[test]
fn a_slot_add_in_lane_b_stays_in_lane_b() {
    let (mut driver, backend) = slot_driver("main-stereo", false, &[library_pedal()], |_| {});
    let _ = driver.click("Chain: Roland JC-40").expect("lane B amp");
    driver.settle();
    let left = lane_titles(&backend, false);
    let _ = driver.click("Add a pedal before the amp").expect("slot");
    driver.settle();
    assert!(!driver.is_visible("LANE A") && !driver.is_visible("LANE B"));
    assert!(driver.shows_text("before Roland JC-40"));
    let _ = driver.click("Library Drive").expect("library row");
    driver.settle();
    let right = lane_titles(&backend, true);
    assert_eq!(
        right.first().map(String::as_str),
        Some("Library Drive"),
        "{right:?}"
    );
    assert_eq!(
        right.get(1).map(String::as_str),
        Some("Roland JC-40"),
        "{right:?}"
    );
    assert_eq!(lane_titles(&backend, false), left);
}

#[test]
fn a_slot_menu_takes_over_a_download_of_the_plus_menu() {
    let (mut driver, backend) = slot_driver("main-mono", true, &[library_pedal()], add_overdrive);
    open_menu(&mut driver);
    let _ = driver.click("PEDAL").expect("PEDAL row");
    driver.settle();
    answer_menu(
        &mut driver,
        vec![gear_tone(701, "Slow Download", "pedal")],
        1,
    );
    // The + menu's pick is still downloading to the end of the lane.
    let insert = last_insert(&backend);
    let flow = driver.chrome_mut().load_flow_mut();
    flow.select(ronie_core::ChainSide::Left, &insert);
    let _ticket = flow
        .begin_remote_load(gear_tone(701, "Slow Download", "pedal"))
        .expect("load");
    driver.settle();
    assert!(driver.shows_text("DOWNLOADING"));
    // The + menu covers the slot, so the player closes it; the download goes on.
    let _ = driver.press_key(eframe::egui::Key::Escape);
    driver.settle();
    assert_eq!(
        driver.chrome().loading_tone(),
        Some(ronie_core::ToneId(701))
    );

    let _ = driver.click("Add a pedal after these").expect("slot");
    driver.settle();
    assert!(driver.is_visible("Back to categories"), "the slot menu");
    let _ = driver.click("Library Drive").expect("library row");
    driver.settle();
    let titles = lane_titles(&backend, false);
    assert_eq!(titles[2], "Library Drive", "{titles:?}");
    assert!(!titles.contains(&"Slow Download".to_owned()), "{titles:?}");
    assert_eq!(driver.chrome().loading_tone(), None);
    assert!(!driver.shows_text("DOWNLOADING"));
}

/// The labels of the slots that read as selected, which the stage draws solid.
fn solid_slots(driver: &UiDriver) -> Vec<String> {
    driver
        .controls()
        .iter()
        .filter(|control| control.label.starts_with("Add a ") && control.label != "Add a block")
        .filter(|control| control.checked)
        .map(|control| control.label.clone())
        .collect()
}

#[test]
fn the_open_slot_reads_as_selected() {
    let (mut driver, _) = slot_driver("main-mono", false, &[], add_overdrive);
    assert!(solid_slots(&driver).is_empty());
    let _ = driver.click("Add a pedal after these").expect("slot");
    driver.settle();
    assert_eq!(solid_slots(&driver), ["Add a pedal after these"]);
}

#[test]
fn a_slot_menu_closes_when_its_slot_leaves_the_stage() {
    let (mut driver, backend) = slot_driver("main-mono", true, &[library_pedal()], add_overdrive);
    open_menu(&mut driver);
    let _ = driver.click("PEDAL").expect("PEDAL row");
    driver.settle();
    let slow = gear_tone(701, "Slow Download", "pedal");
    answer_menu(&mut driver, vec![slow.clone()], 1);
    // The + menu's pick is still downloading to the end of the lane.
    let insert = last_insert(&backend);
    let flow = driver.chrome_mut().load_flow_mut();
    flow.select(ronie_core::ChainSide::Left, &insert);
    let ticket = flow.begin_remote_load(slow.clone()).expect("load");
    driver.settle();
    let _ = driver.press_key(eframe::egui::Key::Escape);
    driver.settle();

    let _ = driver.click("Add a pedal after these").expect("slot");
    driver.settle();
    assert!(driver.is_visible("Back to categories"), "the slot menu");
    assert_eq!(solid_slots(&driver), ["Add a pedal after these"]);

    // The download lands and its pedal opens on the stage, whose own pedal slot sits after it.
    assert!(
        ticket
            .publish(slow, ronie_core::ModelId(7010), vec![1, 2, 3].into())
            .expect("publish")
    );
    assert!(
        driver
            .chrome_mut()
            .load_flow_mut()
            .finish_remote(ticket.generation())
    );
    driver.chrome_mut().gallery_mut().activate(&insert);
    driver.chrome_mut().sync_detail();
    driver.settle();
    assert!(
        driver.is_visible("Add a pedal after these"),
        "the new page has a pedal slot"
    );
    assert!(
        !driver.is_visible("Back to categories"),
        "the menu of the slot that left closes"
    );
    assert!(
        solid_slots(&driver).is_empty(),
        "{:?}",
        solid_slots(&driver)
    );
}

#[test]
fn a_browser_pick_from_a_slot_lands_at_the_slot() {
    let (mut driver, backend) = slot_driver("main-mono", false, &[], add_overdrive);
    let before = lane_titles(&backend, false);
    let _ = driver.click("Add a pedal after these").expect("slot");
    driver.settle();
    let _ = driver.click("OPEN THE TONE BROWSER").expect("browser link");
    driver.settle();
    assert!(
        driver.chrome_mut().browser_mut().is_some(),
        "the browser opened"
    );
    driver
        .chrome_mut()
        .pick_tone(gear_tone(803, "Browser Drive", "pedal"));
    for _ in 0..20 {
        driver.settle();
    }
    let titles = lane_titles(&backend, false);
    assert_eq!(titles.len(), before.len() + 1, "{titles:?}");
    assert_eq!(titles[2], "Browser Drive", "{titles:?}");
    let _ = driver.click("Undo").expect("undo");
    driver.settle();
    assert_eq!(lane_titles(&backend, false), before);
}

#[test]
fn a_file_from_a_slot_lands_at_the_slot() {
    let (mut driver, backend) = slot_driver("main-mono", false, &[], add_overdrive);
    let before = lane_titles(&backend, false);
    let directory = tempfile::tempdir().expect("tempdir");
    let path = directory.path().join("closet-cab.wav");
    write_ir(&path);
    let insert = last_insert(&backend);
    driver
        .chrome_mut()
        .drop_file_into(
            &insert,
            &path,
            Placement::After(BlockId("blk-od".to_owned())),
        )
        .expect("the file loads");
    driver.settle();
    let lane = backend.chain_state(0).chain.left.clone();
    let tones: Vec<_> = lane.iter().filter(|block| !block.is_insert).collect();
    assert_eq!(tones.len(), before.len() + 1);
    assert_eq!(tones[2].id.0, insert, "the file sits after the run");
    assert!(
        matches!(driver.chrome().gallery().selection(), GallerySelection::BlockDetail(id) if id.0 == insert),
        "the new block is on the stage"
    );
    let _ = driver.click("Undo").expect("undo");
    driver.settle();
    assert_eq!(lane_titles(&backend, false), before);
}

/// A local tone, which lands at once with or without a service.
fn local_pedal(id: i64, title: &str) -> ronie_core::Tone {
    ronie_core::Tone::from_value(serde_json::json!({
        "id": id, "title": title, "format": "nam", "gear": "pedal", "local": true,
        "models": [{"id": id * 10, "name": "m", "model_url": format!("file:///stash/{id}.nam")}]
    }))
    .expect("tone")
}

/// Opens the tone browser from the pedal slot after the run, for the lane's insert.
fn open_browser_from_the_slot(driver: &mut UiDriver) {
    let _ = driver.click("Add a pedal after these").expect("slot");
    driver.settle();
    let _ = driver.click("OPEN THE TONE BROWSER").expect("browser link");
    driver.settle();
    assert!(
        driver.chrome_mut().browser_mut().is_some(),
        "the browser opened"
    );
}

/// Loads `tone` into insert `insert` of lane A through the browser of the chain, which no slot
/// opened.
fn load_through_the_browser(driver: &mut UiDriver, insert: &str, tone: ronie_core::Tone) {
    let chrome = driver.chrome_mut();
    chrome
        .gallery_mut()
        .select_tone(ronie_core::ChainSide::Left, BlockId(insert.to_owned()));
    chrome.sync_browser();
    chrome.pick_tone(tone);
    chrome.sync_browser();
    driver.settle();
}

/// A key setup that starts a service over `transport` when a key is saved.
struct KeyOnSave(tempfile::TempDir, ronie_core::RecordedTransport);

/// Saves a key, which starts a service over `transport` in place of none.
fn start_a_service(driver: &mut UiDriver, transport: ronie_core::RecordedTransport) {
    let chrome = driver.chrome_mut();
    chrome.use_key_setup(Box::new(KeyOnSave(
        tempfile::tempdir().expect("temporary settings"),
        transport,
    )));
    chrome.open_key_page();
    chrome
        .key_page_mut()
        .expect("key page")
        .set_draft("t3k_pub_test_fake");
    chrome.save_key().expect("the key saves");
}

impl ronie_ui::services::key_setup::KeySetup for KeyOnSave {
    fn origin(&self) -> String {
        "https://www.fixtures.invalid".to_owned()
    }

    fn has_saved_key(&self) -> bool {
        false
    }

    fn save(
        &mut self,
        key: &str,
    ) -> Result<
        Option<ronie_ui::services::key_setup::CommunityParts>,
        ronie_ui::services::key_setup::KeySetupError,
    > {
        let origin = url::Url::parse("https://api.example").expect("origin");
        Ok(Some(
            ronie_ui::services::community::CommunityService::new(
                ronie_core::OAuthConfig::new(origin.clone(), key.to_owned()),
                ronie_core::OriginPolicy::new(origin, []),
                self.1.clone(),
                ronie_core::SettingsStore::standalone(
                    self.0.path(),
                    ronie_core::UI_PREFERENCES_SETTINGS,
                ),
            )
            .into(),
        ))
    }

    fn remove(
        &mut self,
    ) -> Result<
        Option<ronie_ui::services::key_setup::CommunityParts>,
        ronie_ui::services::key_setup::KeySetupError,
    > {
        Ok(None)
    }
}

#[test]
fn a_service_change_ends_the_place_of_a_browser_from_a_slot() {
    let (mut driver, backend) = slot_driver("main-mono", false, &[], add_overdrive);
    let insert = last_insert(&backend);
    open_browser_from_the_slot(&mut driver);
    // A saved key starts a service, whose views start again with no browser.
    start_a_service(&mut driver, ronie_core::RecordedTransport::new(Vec::new()));
    assert!(driver.chrome_mut().browser_mut().is_none());

    load_through_the_browser(&mut driver, &insert, local_pedal(901, "Later Drive"));
    let titles = lane_titles(&backend, false);
    assert_eq!(
        titles.last().map(String::as_str),
        Some("Later Drive"),
        "{titles:?}"
    );
}

#[test]
fn a_browser_from_a_slot_places_only_the_tone_of_its_insert() {
    let (mut driver, backend) = slot_driver("main-mono", false, &[], add_overdrive);
    let insert = last_insert(&backend);
    open_browser_from_the_slot(&mut driver);
    // A file fills the slot's insert at the end of the lane while the browser is open.
    let directory = tempfile::tempdir().expect("tempdir");
    let path = directory.path().join("closet-cab.wav");
    write_ir(&path);
    driver
        .chrome_mut()
        .drop_file_into(&insert, &path, Placement::End)
        .expect("the file loads");
    driver.settle();
    let file = lane_titles(&backend, false);

    // A later load into the next insert lands at the end, and the file stays where it is.
    let next = last_insert(&backend);
    assert_ne!(next, insert);
    load_through_the_browser(&mut driver, &next, local_pedal(902, "Next Drive"));
    let mut expected = file;
    expected.push("Next Drive".to_owned());
    assert_eq!(lane_titles(&backend, false), expected);
}

#[test]
fn a_file_load_keeps_the_place_of_a_slot_download() {
    let (mut driver, backend) = slot_driver("main-stereo", false, &[], |_| {});
    // A service whose first request, the download, waits for good.
    let held = Arc::new(tokio::sync::Notify::new());
    start_a_service(
        &mut driver,
        ronie_core::RecordedTransport::gated(Vec::new(), held.clone(), held),
    );
    driver.chrome_mut().close_auth();
    let chain = backend.chain_state(0).chain;
    let first = chain.left[0].id.clone();
    let insert = last_insert(&backend);
    let lane_b_insert = chain
        .right
        .iter()
        .rev()
        .find(|block| block.is_insert)
        .map(|block| block.id.0.clone())
        .expect("lane B insert");

    // A slot download before the first block of lane A starts and is held.
    let pedal = gear_tone(704, "Slot Download", "pedal");
    driver.chrome_mut().add_tone(
        ronie_core::ChainSide::Left,
        &insert,
        pedal.clone(),
        Placement::Before(first.clone()),
    );
    assert_eq!(
        driver.chrome().loading_tone(),
        Some(ronie_core::ToneId(704))
    );

    // A file lands at the end of lane B meanwhile.
    let directory = tempfile::tempdir().expect("tempdir");
    let path = directory.path().join("closet-cab.wav");
    write_ir(&path);
    driver
        .chrome_mut()
        .drop_file_into(&lane_b_insert, &path, Placement::End)
        .expect("the file loads");
    assert_eq!(
        lane_titles(&backend, true).last().map(String::as_str),
        Some("closet-cab")
    );

    // The download lands in its insert, as the worker publishes it.
    backend
        .dispatch(ronie_ui::backend::UiAction::LoadTone {
            target_insert_id: BlockId(insert.clone()),
            tone: pedal,
            model_id: ronie_core::ModelId(7040),
        })
        .expect("publish");
    let flow = driver.chrome_mut().load_flow_mut();
    let generation = flow.generation();
    assert!(flow.finish_remote(generation));
    driver.chrome_mut().sync_browser();
    let left = lane_titles(&backend, false);
    assert_eq!(
        left.first().map(String::as_str),
        Some("Slot Download"),
        "{left:?}"
    );
}

// ---------------------------------------------------------------- A cab's two mics

const CAB: &str = "Chain: UK Greenback - Marshall 1960TV 4x12 M201";

/// main-mono with its cab holding three mic positions, open on the cab.
fn cab_driver() -> (UiDriver, Arc<MockBackend>) {
    let (mut driver, backend) = edited_driver("main-mono", |chain| {
        let tone = &mut chain["chain"][2]["tone"];
        tone["models"] = serde_json::json!([
            {"id": 1103, "name": "M201 Cap Edge"},
            {"id": 1203, "name": "SM57 Cap Edge"},
            {"id": 1303, "name": "R121 Center"},
        ]);
        tone["models_count"] = 3.into();
        // A local pack switches its mics with no download and no sign-in.
        tone["local"] = true.into();
    });
    let _ = driver.click(CAB).expect("cab");
    driver.settle();
    (driver, backend)
}

fn cab(backend: &MockBackend) -> ronie_core::ChainBlock {
    block_titled(backend, "Greenback").expect("cab")
}

fn add_b(driver: &mut UiDriver) {
    let _ = driver.click("Add a second IR").expect("ADD A SECOND IR");
    driver.settle();
}

#[test]
fn a_cab_with_two_mics_offers_a_second_ir() {
    let (driver, backend) = cab_driver();
    let panel = rect_of(&driver, "Cab mics").expect("the MICS panel");
    assert!(driver.shows_text("MICS"));
    assert!(driver.shows_text("1 IR"));
    assert!(driver.is_visible("IR A capture"));
    assert!(driver.is_visible("Add a second IR"));
    assert!(driver.shows_text(
        "Blend a second mic: pick another mic position from this cab. Both play at once; BLEND \
         mixes them."
    ));
    assert!(!driver.is_visible("IR B capture"));
    assert!((panel.width() - 300.0).abs() < 0.5, "{panel:?}");
    let render = rect_of(&driver, "Cab render").expect("render");
    assert!(
        render.right() <= panel.left() - 20.0,
        "the render moves left of the panel: {render:?} vs {panel:?}"
    );
    assert_eq!(cab(&backend).ir_b, None);
}

#[test]
fn adding_b_picks_another_mic_and_shows_the_blend() {
    let (mut driver, backend) = cab_driver();
    add_b(&mut driver);
    let block = cab(&backend);
    let b = block.ir_b.expect("B is set").model_id;
    assert_ne!(Some(b), block.active_model_id, "another mic");
    assert!(driver.shows_text("2 IRs blended"));
    assert!(driver.is_visible("IR B capture"));
    assert!(!driver.is_visible("Add a second IR"));
    assert!(driver.is_visible("Blend: 50 / 50"));
    assert!(driver.is_visible("IR A level: 0.0 dB"));
    assert!(driver.is_visible("IR B level: 0.0 dB"));
    assert!(driver.is_visible("Invert IR B phase"));
    assert!(driver.is_visible("Remove IR B"));
}

#[test]
fn a_blend_drag_is_one_undo_step() {
    let (mut driver, backend) = cab_driver();
    add_b(&mut driver);
    let label = driver.label_starting_with("Blend:").expect("BLEND knob");
    let up: Vec<eframe::egui::Vec2> = (1..=6)
        .map(|step| eframe::egui::vec2(0.0, -6.0 * step as f32))
        .collect();
    let _ = driver.drag_control(&label, &up).expect("drag");
    let dragged = cab(&backend);
    assert!(dragged.params.ir_blend > 0.6, "{}", dragged.params.ir_blend);
    assert!(
        driver
            .label_starting_with("Blend:")
            .is_some_and(|label| label != "Blend: 50 / 50")
    );
    let _ = driver.click("Undo").expect("undo");
    driver.settle();
    let undone = cab(&backend);
    assert_eq!(
        undone.params.ir_blend, 0.5,
        "one step undoes the whole drag"
    );
    assert!(undone.ir_b.is_some(), "and leaves B");
}

#[test]
fn a_level_readout_drags_like_a_knob() {
    let (mut driver, backend) = cab_driver();
    add_b(&mut driver);
    let up: Vec<eframe::egui::Vec2> = (1..=4)
        .map(|step| eframe::egui::vec2(0.0, -5.0 * step as f32))
        .collect();
    let _ = driver
        .drag_control("IR B level: 0.0 dB", &up)
        .expect("drag");
    assert!(cab(&backend).params.ir_b_level > 0.5);
    assert_eq!(cab(&backend).params.ir_a_level, 0.5);
}

#[test]
fn removing_b_restores_a_alone() {
    let (mut driver, backend) = cab_driver();
    add_b(&mut driver);
    let _ = driver.click("Remove IR B").expect("REMOVE B");
    driver.settle();
    let block = cab(&backend);
    assert_eq!(block.ir_b, None);
    assert_eq!(block.active_model_id, Some(ronie_core::ModelId(1103)));
    assert!(driver.shows_text("1 IR"));
    assert!(driver.is_visible("Add a second IR"));
    assert!(!driver.is_visible("IR B capture"));
}

#[test]
fn invert_toggles_b_phase() {
    let (mut driver, backend) = cab_driver();
    add_b(&mut driver);
    assert!(!cab(&backend).params.ir_b_invert);
    let _ = driver.click("Invert IR B phase").expect("Ø B");
    driver.settle();
    assert!(cab(&backend).params.ir_b_invert);
    let _ = driver.click("Invert IR B phase").expect("Ø B");
    driver.settle();
    assert!(!cab(&backend).params.ir_b_invert);
}

#[test]
fn picking_b_from_its_capture_list_changes_only_b() {
    let (mut driver, backend) = cab_driver();
    add_b(&mut driver);
    let before = cab(&backend).ir_b.expect("B").model_id;
    let _ = driver.click("IR B capture").expect("B picker");
    driver.settle();
    assert!(
        !driver.is_visible("M201 Cap Edge"),
        "A's mic is not offered as B"
    );
    let other = if before.0 == 1203 {
        "R121 Center"
    } else {
        "SM57 Cap Edge"
    };
    let _ = driver.click(other).expect("other mic");
    driver.settle();
    let block = cab(&backend);
    assert_ne!(block.ir_b.expect("B").model_id, before);
    assert_eq!(block.active_model_id, Some(ronie_core::ModelId(1103)));
}

#[test]
fn a_b_with_no_bytes_shows_as_not_loaded() {
    let (mut driver, backend) = edited_driver("main-mono", |chain| {
        let block = &mut chain["chain"][2];
        block["tone"]["models"] = serde_json::json!([
            {"id": 1103, "name": "M201 Cap Edge"},
            {"id": 1203, "name": "SM57 Cap Edge"},
        ]);
        block["irBModelId"] = 1203.into();
        block["irBMissing"] = true.into();
    });
    assert!(cab(&backend).ir_b_missing);
    let _ = driver.click(CAB).expect("cab");
    driver.settle();
    assert!(driver.is_visible("IR B capture (not loaded)"));
    assert!(!driver.is_visible("IR B capture"));
}

#[test]
fn a_single_model_cab_has_no_second_ir() {
    let (mut driver, _) = edited_driver("main-mono", |chain| {
        chain["chain"][2]["tone"]["models_count"] = 1.into();
    });
    let _ = driver.click(CAB).expect("cab");
    driver.settle();
    assert!(driver.is_visible("Cab render"));
    assert!(!driver.is_visible("Cab mics"));
    assert!(!driver.is_visible("Add a second IR"));
}

#[test]
fn the_reference_layout_has_no_mics_panel() {
    let mut doc: serde_json::Value =
        serde_json::from_str(include_str!("../../../fixtures/ui-scenarios.json")).expect("json");
    let scenario = doc["scenarios"]
        .as_array_mut()
        .expect("scenarios")
        .iter_mut()
        .find(|scenario| scenario["id"] == "main-mono")
        .expect("scenario");
    let block = &mut scenario["state"]["chain"]["chain"][2];
    block["tone"]["models"] = serde_json::json!([
        {"id": 1103, "name": "M201 Cap Edge"},
        {"id": 1203, "name": "SM57 Cap Edge"},
    ]);
    block["irBModelId"] = 1203.into();
    let scenario = load_scenarios(&doc.to_string())
        .expect("scenarios")
        .into_iter()
        .find(|scenario| scenario.id == "main-mono")
        .expect("scenario");
    let backend = Arc::new(MockBackend::from_scenario(&scenario));
    let chrome = ChromeController::new(backend.clone(), Vec::new(), false, true);
    let mut driver = UiDriver::new(chrome, (*backend).clone());
    driver.settle();
    driver.chrome_mut().gallery_mut().activate("blk-3");
    driver.settle();
    assert!(
        driver.label_starting_with("In:").is_some(),
        "the detail is open"
    );
    for name in [
        "Cab mics",
        "Add a second IR",
        "IR A capture",
        "IR B capture",
        "Remove IR B",
        "Invert IR B phase",
    ] {
        assert!(!driver.is_visible(name), "{name}");
    }
    assert!(driver.label_starting_with("Blend:").is_none());
}

// ---------------------------------------------------------------- The pedal LCD

/// The pedalboard where the fuzz is a local tone with three captures, followed by the
/// single-capture overdrive.
fn capture_pedalboard() -> (UiDriver, Arc<MockBackend>) {
    crowded_capture_pedalboard(0)
}

/// `capture_pedalboard` with `extra` more overdrives after it, so the renders shrink.
fn crowded_capture_pedalboard(extra: usize) -> (UiDriver, Arc<MockBackend>) {
    let mut scenario = scenario("main-mono");
    let chain = scenario.state["chain"]["chain"]
        .as_array_mut()
        .expect("chain");
    chain[0]["tone"]["local"] = true.into();
    chain[0]["tone"]["models"] = serde_json::json!([
        { "id": 1104, "name": "Muff 1" },
        { "id": 1105, "name": "Muff 2" },
        { "id": 1106, "name": "Muff 3" },
    ]);
    let mut od = chain[0].clone();
    od["blockId"] = "blk-od".into();
    od["activeModelId"] = 1199.into();
    od["tone"]["id"] = 199.into();
    od["tone"]["title"] = OD.into();
    od["tone"]["models"] = serde_json::json!([{ "id": 1199, "name": "Drive 5" }]);
    for index in 0..extra {
        let mut more = od.clone();
        more["blockId"] = format!("blk-od{index}").into();
        more["tone"]["id"] = (300 + index).into();
        more["tone"]["title"] = format!("Overdrive {index}").into();
        chain.insert(1, more);
    }
    chain.insert(1, od);
    let backend = Arc::new(MockBackend::from_scenario(&scenario));
    let mut chrome = ChromeController::new(backend.clone(), Vec::new(), false, true);
    chrome.set_view_mode(ViewMode::RonieFlow);
    let mut driver = UiDriver::new(chrome, (*backend).clone());
    driver.settle();
    (driver, backend)
}

fn active_model(backend: &MockBackend, block: &str) -> Option<i64> {
    backend
        .chain_state(0)
        .chain
        .left
        .iter()
        .find(|each| each.id.0 == block)
        .and_then(|each| each.active_model_id)
        .map(|id| id.0)
}

#[test]
fn a_pedal_lcd_shows_its_capture() {
    let (driver, _) = capture_pedalboard();
    assert!(
        driver.is_visible("Capture display: Muff 1"),
        "{:?}",
        driver.visible_labels()
    );
    assert!(driver.shows_text("MUFF 1"));
}

#[test]
fn hovering_the_lcd_shows_the_capture_hint() {
    let (mut driver, _) = capture_pedalboard();
    driver.hover("Capture display: Muff 1").expect("LCD");
    driver.settle();
    assert!(driver.shows_text("CAPTURE 1 / 3 · Muff 1 · click for next"));
}

#[test]
fn clicking_the_lcd_steps_to_the_next_capture() {
    let (mut driver, backend) = capture_pedalboard();
    let _ = driver.click("Capture display: Muff 1").expect("LCD");
    driver.settle();
    assert_eq!(active_model(&backend, "blk-1"), Some(1105));
    assert!(driver.shows_text("MUFF 2"));
    let _ = driver.click("Capture display: Muff 2").expect("LCD");
    driver.settle();
    assert_eq!(active_model(&backend, "blk-1"), Some(1106));
    let _ = driver.click("Capture display: Muff 3").expect("LCD");
    driver.settle();
    assert_eq!(active_model(&backend, "blk-1"), Some(1104), "wraps");
    assert!(driver.is_visible("Capture display: Muff 1"));
}

#[test]
fn a_single_capture_pedal_does_nothing_on_click() {
    let (mut driver, backend) = capture_pedalboard();
    let _ = driver
        .click(&format!("Pedal render: {OD}"))
        .expect("neighbour");
    driver.settle();
    let revision = backend.chain_state(0).revision;
    let _ = driver.click("Capture display: Drive 5").expect("LCD");
    driver.settle();
    assert_eq!(active_model(&backend, "blk-od"), Some(1199));
    assert_eq!(backend.chain_state(0).revision, revision);
    assert!(driver.shows_text("DRIVE 5"));
}

#[test]
fn a_neighbour_lcd_is_shown_but_not_interactive() {
    let (mut driver, _) = capture_pedalboard();
    assert!(driver.shows_text("DRIVE 5"), "the neighbour's LCD");
    assert!(
        driver
            .label_starting_with("Capture display: Drive")
            .is_none(),
        "no display control on a neighbour"
    );
    let _ = driver
        .click(&format!("Pedal render: {OD}"))
        .expect("neighbour");
    driver.settle();
    assert_eq!(
        driver.chrome().gallery().selection(),
        &GallerySelection::BlockDetail(BlockId("blk-od".to_owned()))
    );
}

/// The fuzz of main-detail on its own Archetype page, attached to a service that lists
/// captures 1104 to 1106. Clicks and frames go through `lcd_frame`.
fn catalog_pedal(backend: &Arc<MockBackend>) -> ronie_ui::default::BlockDetail {
    use ronie_ui::services::{ModelList, ModelSwitchFlow, session::SessionStatus::SignedIn};

    let shared: Arc<dyn UiBackend> = backend.clone();
    let mut detail =
        ronie_ui::default::BlockDetail::new(shared.clone(), "blk-1", false).expect("detail");
    detail.set_page(ronie_ui::default::block::BlockPage::Archetype);
    detail.attach_model_switching(ModelSwitchFlow::new(shared));
    let request = detail.take_models_request(SignedIn).expect("a list is due");
    let models = [1104, 1105, 1106]
        .into_iter()
        .map(|id| {
            ronie_core::ToneModel::from_value(
                &serde_json::json!({
                    "id": id,
                    "name": format!("Model {id}"),
                    "model_url": format!("https://models.example/{id}.nam"),
                }),
                false,
            )
            .expect("model")
        })
        .collect();
    assert!(detail.complete_models(
        request.generation,
        request.tone,
        ModelList { models, total: 3 }
    ));
    detail
}

/// One frame of a lone detail. Returns its controls by label and its static texts.
fn lcd_frame(
    context: &eframe::egui::Context,
    detail: &mut ronie_ui::default::BlockDetail,
    events: Vec<eframe::egui::Event>,
) -> (Vec<(String, eframe::egui::Rect)>, Vec<String>) {
    use eframe::egui;
    use ronie_ui::services::session::SessionStatus::SignedIn;

    let input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(1024.0, 614.0),
        )),
        events,
        ..Default::default()
    };
    let mut output = context.run_ui(input, |ui| detail.show(ui, SignedIn));
    output.textures_delta.clear();
    let Some(update) = output.platform_output.accesskit_update.take() else {
        return (Vec::new(), Vec::new());
    };
    let widgets = context.viewport(|viewport| {
        viewport
            .prev_pass
            .widgets
            .layers()
            .flat_map(|(_, widgets)| widgets.iter().copied())
            .map(|widget| (widget.id.accesskit_id(), widget))
            .collect::<std::collections::HashMap<_, _>>()
    });
    let controls = update
        .nodes
        .iter()
        .filter_map(|(id, node)| {
            let widget = widgets.get(id)?;
            Some((node.label()?.to_owned(), widget.interact_rect))
        })
        .collect();
    let texts = update
        .nodes
        .iter()
        .filter(|(_, node)| node.role() == accesskit::Role::Label)
        .filter_map(|(_, node)| node.value().map(str::to_owned))
        .collect();
    (controls, texts)
}

/// Clicks the control whose label starts with `prefix`.
fn lcd_click(
    context: &eframe::egui::Context,
    detail: &mut ronie_ui::default::BlockDetail,
    prefix: &str,
) {
    use eframe::egui;

    let (controls, _) = lcd_frame(context, detail, Vec::new());
    let at = controls
        .iter()
        .find(|(label, _)| label.starts_with(prefix))
        .map(|(_, rect)| rect.center())
        .unwrap_or_else(|| panic!("no {prefix}: {controls:?}"));
    let button = |pressed| egui::Event::PointerButton {
        pos: at,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: egui::Modifiers::NONE,
    };
    lcd_frame(context, detail, vec![egui::Event::PointerMoved(at)]);
    lcd_frame(context, detail, vec![button(true)]);
    lcd_frame(context, detail, vec![button(false)]);
}

fn lcd_context() -> eframe::egui::Context {
    let context = eframe::egui::Context::default();
    context.enable_accesskit();
    ronie_ui::fonts::Fonts::install(&context);
    context
}

#[test]
fn the_lcd_shows_dots_while_the_capture_downloads() {
    let backend = Arc::new(MockBackend::from_scenario(&scenario("main-detail")));
    let mut detail = catalog_pedal(&backend);
    let context = lcd_context();
    lcd_frame(&context, &mut detail, Vec::new());
    let (_, texts) = lcd_frame(&context, &mut detail, Vec::new());
    assert!(texts.iter().any(|text| text == "MODEL 1104"), "{texts:?}");

    lcd_click(&context, &mut detail, "Capture display: Model 1104");
    let request = detail.take_switch_request().expect("a download");
    assert_eq!(request.model.id, ronie_core::ModelId(1105));
    let (_, texts) = lcd_frame(&context, &mut detail, Vec::new());
    assert!(texts.iter().any(|text| text == "···"), "{texts:?}");
    assert!(
        !texts.iter().any(|text| text.starts_with("MODEL")),
        "{texts:?}"
    );
}

#[test]
fn rapid_lcd_clicks_end_on_the_landed_capture() {
    use ronie_core::{ModelId, Tone};

    let backend = Arc::new(MockBackend::from_scenario(&scenario("main-detail")));
    let mut detail = catalog_pedal(&backend);
    let context = lcd_context();
    lcd_frame(&context, &mut detail, Vec::new());

    lcd_click(&context, &mut detail, "Capture display:");
    let first = detail.take_switch_request().expect("a download");
    lcd_click(&context, &mut detail, "Capture display:");
    let second = detail.take_switch_request().expect("the next download");
    assert_eq!(first.model.id, ModelId(1105));
    assert_eq!(
        second.model.id,
        ModelId(1106),
        "steps from the capture on its way"
    );
    assert!(
        detail.take_switch_request().is_none(),
        "one switch at a time"
    );

    let stored = |request: &ronie_ui::services::ModelSwitchRequest, id: i64| {
        Tone {
            models: vec![request.model.clone()],
            ..request.tone.clone()
        }
        .for_storage(ModelId(id), &[])
        .expect("stored tone")
    };
    let late = first
        .ticket
        .publish(stored(&first, 1105), ModelId(1105), vec![1, 2, 3].into());
    assert!(
        !matches!(late, Ok(true)),
        "the superseded switch cannot land"
    );
    assert!(!detail.finish_switch(first.ticket.generation()));
    assert!(
        second
            .ticket
            .publish(stored(&second, 1106), ModelId(1106), vec![1, 2, 3].into())
            .expect("publish")
    );
    assert!(detail.finish_switch(second.ticket.generation()));

    let (controls, texts) = lcd_frame(&context, &mut detail, Vec::new());
    assert!(texts.iter().any(|text| text == "MODEL 1106"), "{texts:?}");
    assert!(!texts.iter().any(|text| text == "···"), "{texts:?}");
    assert!(
        controls
            .iter()
            .any(|(label, _)| label == "Capture display: Model 1106"),
        "{controls:?}"
    );
    assert_eq!(active_model(&backend, "blk-1"), Some(1106));
}

#[test]
fn a_knob_beside_a_small_lcd_keeps_its_clicks() {
    use eframe::egui;

    let (mut driver, backend) = crowded_capture_pedalboard(5);
    let display = rect_of(&driver, "Capture display: Muff 1").expect("LCD");
    let render = rect_of(&driver, "Pedal render").expect("render");
    assert!(render.height() < 190.0, "a small render: {render:?}");
    // The knob nearest the LCD, and a point that both hit areas cover.
    let (label, knob) = ["GAIN IN:", "OUTPUT:", "MIX:"]
        .into_iter()
        .filter_map(|prefix| {
            let label = driver.label_starting_with(prefix)?;
            let rect = rect_of(&driver, &label)?;
            Some((label, rect))
        })
        .min_by(|(_, a), (_, b)| {
            a.center()
                .distance(display.center())
                .total_cmp(&b.center().distance(display.center()))
        })
        .expect("a knob");
    let shared = knob.intersect(display);
    assert!(shared.is_positive(), "{knob:?} and {display:?} overlap");
    let at = shared.center();
    let prefix = label.split(':').next().expect("prefix").to_owned();

    let _ = driver.click_at(at);
    driver.settle();
    assert_eq!(
        active_model(&backend, "blk-1"),
        Some(1104),
        "a click on the knob"
    );
    assert!(
        driver
            .focused()
            .is_some_and(|focused| focused.starts_with(&prefix)),
        "the knob takes the click: {:?}",
        driver.focused()
    );

    let button = |pressed| egui::Event::PointerButton {
        pos: at,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: egui::Modifiers::NONE,
    };
    driver.play(vec![egui::Event::PointerMoved(at)]);
    for _ in 0..2 {
        driver.play(vec![button(true)]);
        driver.play(vec![button(false)]);
    }
    driver.settle();
    assert_eq!(
        active_model(&backend, "blk-1"),
        Some(1104),
        "a double-click on the knob"
    );
}

#[test]
fn an_lcd_step_is_one_undo_step() {
    let (mut driver, backend) = capture_pedalboard();
    let _ = driver.click("Capture display: Muff 1").expect("LCD");
    driver.settle();
    assert_eq!(active_model(&backend, "blk-1"), Some(1105));
    let _ = driver.click("Undo").expect("undo");
    driver.settle();
    assert_eq!(active_model(&backend, "blk-1"), Some(1104), "one undo");
    assert!(driver.is_visible("Capture display: Muff 1"));
}

#[test]
fn a_downloading_display_says_so() {
    let backend = Arc::new(MockBackend::from_scenario(&scenario("main-detail")));
    let mut detail = catalog_pedal(&backend);
    let context = lcd_context();
    lcd_frame(&context, &mut detail, Vec::new());
    lcd_click(&context, &mut detail, "Capture display: Model 1104");
    let _request = detail.take_switch_request().expect("a download");
    let (controls, _) = lcd_frame(&context, &mut detail, Vec::new());
    assert!(
        controls
            .iter()
            .any(|(label, _)| label == "Capture display: Model 1105 (loading)"),
        "{controls:?}"
    );
}

// ---------------------------------------------------------------- The amp capture lights

/// The amp of main-mono, open on the stage, with `count` captures named "Take 1" and up.
fn amp_driver(count: usize) -> (UiDriver, Arc<MockBackend>) {
    let (mut driver, backend) = edited_driver("main-mono", |chain| {
        let models: Vec<_> = (0..count)
            .map(|index| serde_json::json!({ "id": 1101 + index, "name": format!("Take {}", index + 1) }))
            .collect();
        chain["chain"][1]["tone"]["models"] = models.into();
        chain["chain"][1]["tone"]["local"] = true.into();
    });
    let _ = driver.click(VOX).expect("amp button");
    driver.settle();
    (driver, backend)
}

/// The lights of the open amp: label and whether lit, in order.
fn lights(driver: &UiDriver, count: usize) -> Vec<(String, bool)> {
    (1..=count)
        .filter_map(|index| {
            let label = format!("Take {index}");
            driver
                .controls()
                .iter()
                .find(|control| control.label == label)
                .map(|control| (label, control.checked))
        })
        .collect()
}

#[test]
fn an_amp_with_captures_shows_its_lights() {
    let (driver, _) = amp_driver(6);
    assert!(
        driver.is_visible("Captures"),
        "{:?}",
        driver.visible_labels()
    );
    assert!(driver.shows_text("CAPTURE"));
    let lit = lights(&driver, 6);
    assert_eq!(lit.len(), 6);
    assert_eq!(lit.iter().filter(|(_, on)| *on).count(), 1);
    assert!(lit[0].1, "the active capture is lit");
    assert!(
        driver.shows_text("Take 1"),
        "its short name beside the lights"
    );
}

#[test]
fn clicking_a_light_selects_its_capture() {
    let (mut driver, backend) = amp_driver(6);
    let _ = driver.click("Take 4").expect("light");
    driver.settle();
    assert_eq!(active_model(&backend, "blk-2"), Some(1104));
    let lit = lights(&driver, 6);
    assert!(lit[3].1 && lit.iter().filter(|(_, on)| *on).count() == 1);
}

#[test]
fn a_single_capture_amp_has_no_lights() {
    let (driver, _) = amp_driver(1);
    assert!(!driver.is_visible("Captures"));
    assert!(!driver.shows_text("CAPTURE"));
    assert!(!driver.is_visible("Take 1"));
}

#[test]
fn nine_captures_page_the_light_window() {
    let (mut driver, _) = amp_driver(9);
    let shown = lights(&driver, 9);
    assert_eq!(shown.len(), 8, "at most eight lights");
    assert!(shown[0].1, "the active light is inside the window");
    assert!(driver.is_visible("Next captures"));
    let _ = driver.click("Next captures").expect("page right");
    driver.settle();
    let shown = lights(&driver, 9);
    assert_eq!(shown.len(), 8);
    assert!(
        shown.iter().all(|(label, _)| label != "Take 1"),
        "paged by one"
    );
    let _ = driver.click("Take 9").expect("last light");
    driver.settle();
    let shown = lights(&driver, 9);
    assert!(shown.iter().any(|(label, on)| label == "Take 9" && *on));
    let _ = driver.click("Previous captures").expect("page left");
    driver.settle();
    let shown = lights(&driver, 9);
    assert_eq!(
        shown.first().map(|(label, _)| label.as_str()),
        Some("Take 1")
    );
    assert!(
        shown.iter().all(|(_, on)| !*on),
        "paged past the active one"
    );
}

#[test]
fn eight_captures_need_no_paging() {
    let (driver, _) = amp_driver(8);
    assert_eq!(lights(&driver, 8).len(), 8);
    assert!(!driver.is_visible("Next captures"));
    assert!(!driver.is_visible("Previous captures"));
}

#[test]
fn the_active_light_follows_a_capture_change_from_the_picker() {
    let (mut driver, backend) = amp_driver(6);
    driver
        .chrome_mut()
        .detail_mut()
        .expect("detail")
        .select_model(
            ronie_core::ModelId(1103),
            ronie_ui::services::session::SessionStatus::SignedIn,
        )
        .expect("picker select");
    driver.settle();
    assert_eq!(active_model(&backend, "blk-2"), Some(1103));
    let lit = lights(&driver, 6);
    assert!(lit[2].1, "{lit:?}");
}

#[test]
fn cabs_and_pedals_have_no_lights() {
    let (driver, _) = capture_pedalboard();
    assert!(driver.is_visible("Capture display: Muff 1"));
    assert!(!driver.is_visible("Captures"));
    assert!(!driver.is_visible("Muff 2"));
    let (mut cab, _) = edited_driver("main-mono", |chain| {
        chain["chain"][2]["tone"]["models"] = serde_json::json!([
            { "id": 1103, "name": "Mic 1" }, { "id": 1203, "name": "Mic 2" }, { "id": 1303, "name": "Mic 3" }
        ]);
    });
    let _ = cab
        .click("Chain: UK Greenback - Marshall 1960TV 4x12 M201")
        .expect("cab button");
    cab.settle();
    assert!(!cab.is_visible("Captures"));
}
