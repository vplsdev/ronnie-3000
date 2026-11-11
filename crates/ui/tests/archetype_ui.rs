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

#[test]
fn add_a_block_opens_the_tone_browser() {
    let (mut driver, _) = archetype_driver("main-mono");
    let _ = driver.click("Add a block").expect("plus");
    driver.settle();
    assert!(driver.chrome_mut().browser_mut().is_some());
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
    let _ = driver.click("Remove block").expect("remove");
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
    let _ = driver.click("Remove block").expect("remove");
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
