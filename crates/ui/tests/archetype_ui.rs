//! The Archetype layout: light header with the chain as gear icons, stage, light global panel
//! and black utility bar.

use std::sync::Arc;

use ronie_ui::{
    backend::MockBackend,
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
