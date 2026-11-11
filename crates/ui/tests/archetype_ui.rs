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
    let _ = driver.click_first("Add Tone").expect("an open slot");
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
