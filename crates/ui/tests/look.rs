//! The runtime look: light Archetype colours and the dark reference colours.

use eframe::egui::{self, Color32};
use ronie_ui::theme::Look;

fn luminance(color: Color32) -> f64 {
    let channel = |value: u8| {
        let c = f64::from(value) / 255.0;
        if c <= 0.039_28 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * channel(color.r()) + 0.7152 * channel(color.g()) + 0.0722 * channel(color.b())
}

fn contrast(a: Color32, b: Color32) -> f64 {
    let (la, lb) = (luminance(a), luminance(b));
    let (hi, lo) = if la > lb { (la, lb) } else { (lb, la) };
    (hi + 0.05) / (lo + 0.05)
}

#[test]
fn archetype_look_matches_the_spec() {
    let look = Look::archetype();
    assert_eq!(look.background, Color32::from_rgb(0xf3, 0xf3, 0xf1));
    assert_eq!(look.panel, Color32::from_rgb(0xfb, 0xfb, 0xfa));
    assert_eq!(look.edge, Color32::from_rgb(0xd9, 0xd9, 0xd6));
    assert_eq!(look.text, Color32::from_rgb(0x11, 0x11, 0x11));
    assert_eq!(look.text_muted, Color32::from_rgb(0x5f, 0x5f, 0x5c));
    assert_eq!(look.utility, Color32::from_rgb(0x0b, 0x0b, 0x0b));
    assert_eq!(look.utility_text, Color32::from_rgb(0xf2, 0xf2, 0xf2));
    assert_eq!(look.accent, Color32::from_rgb(0xff, 0x6a, 0x1a));
    assert_eq!(look.icon_selected, Color32::from_rgb(0x11, 0x11, 0x11));
    assert_eq!(look.icon_on, Color32::from_rgb(0x6b, 0x6b, 0x68));
    assert_eq!(look.icon_bypassed, Color32::from_rgb(0xb9, 0xb9, 0xb6));
}

#[test]
fn get_falls_back_to_reference() {
    let context = egui::Context::default();
    assert_eq!(Look::get(&context), Look::reference());
}

#[test]
fn install_then_get_round_trips() {
    let context = egui::Context::default();
    Look::install(&context, Look::archetype());
    assert_eq!(Look::get(&context), Look::archetype());
    Look::install(&context, Look::reference());
    assert_eq!(Look::get(&context), Look::reference());
}

#[test]
fn archetype_text_is_readable() {
    let look = Look::archetype();
    for ground in [look.background, look.panel] {
        assert!(contrast(look.text, ground) >= 4.5);
        assert!(contrast(look.text_muted, ground) >= 4.5);
    }
    assert!(contrast(look.utility_text, look.utility) >= 4.5);
}

#[test]
fn reference_look_keeps_todays_black_chrome() {
    let look = Look::reference();
    assert_eq!(look.background, Color32::BLACK);
    assert_eq!(look.utility, Color32::BLACK);
}
