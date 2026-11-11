//! The embedded gear skins: one per rendered family, clean edges, knobs where the manifest says.

use ronie_core::gear::{GearCategory, GearFamily};
use ronie_ui::default::skins::{self, KnobControl};

const FAMILIES: [GearFamily; 23] = [
    GearFamily::OdGreen,
    GearFamily::OdGold,
    GearFamily::Distortion,
    GearFamily::Fuzz,
    GearFamily::Boost,
    GearFamily::Compressor,
    GearFamily::PedalDefaultDark,
    GearFamily::PedalDefaultLight,
    GearFamily::HeadHighGain,
    GearFamily::HeadModernUs,
    GearFamily::HeadBritish,
    GearFamily::ComboChime,
    GearFamily::ComboClean,
    GearFamily::HeadBass,
    GearFamily::AmpDefaultClean,
    GearFamily::AmpDefaultCrunch,
    GearFamily::AmpDefaultHiGain,
    GearFamily::Cab412Modern,
    GearFamily::Cab412Vintage,
    GearFamily::Cab212,
    GearFamily::Cab112,
    GearFamily::Cab810,
    GearFamily::CabDefault,
];

#[test]
fn every_rendered_family_has_a_skin() {
    for family in FAMILIES {
        let id = family.skin_id().expect("rendered family");
        let manifest = skins::manifest(id).unwrap_or_else(|| panic!("{id} has no manifest"));
        assert_eq!(manifest.id, id);
        assert!(skins::body_image(id).is_some(), "{id} has no body");
    }
}

#[test]
fn families_without_a_render_have_no_skin() {
    for family in [
        GearFamily::FullRig,
        GearFamily::Outboard,
        GearFamily::Space,
        GearFamily::Unknown,
    ] {
        assert!(family.skin_id().and_then(skins::manifest).is_none());
    }
    assert!(skins::manifest("no-such-skin").is_none());
}

#[test]
fn bodies_are_sharp_enough_for_a_300_point_stage_at_2x() {
    for family in FAMILIES {
        let id = family.skin_id().unwrap();
        let body = skins::body_image(id).unwrap();
        let manifest = skins::manifest(id).unwrap();
        assert_eq!(
            body.size,
            [manifest.width as usize, manifest.height as usize],
            "{id}"
        );
        assert!(body.size[1] >= 600, "{id} is {:?}", body.size);
    }
}

#[test]
fn bodies_have_no_haze_at_their_edges() {
    for family in FAMILIES {
        let id = family.skin_id().unwrap();
        let body = skins::body_image(id).unwrap();
        let [w, h] = body.size;
        let alpha = |x: usize, y: usize| body.pixels[y * w + x].a();
        for x in 0..w {
            for y in [0, 1, h - 2, h - 1] {
                assert!(
                    alpha(x, y) <= 2,
                    "{id} has alpha {} at ({x}, {y})",
                    alpha(x, y)
                );
            }
        }
        for y in 0..h {
            for x in [0, 1, w - 2, w - 1] {
                assert!(
                    alpha(x, y) <= 2,
                    "{id} has alpha {} at ({x}, {y})",
                    alpha(x, y)
                );
            }
        }
    }
}

#[test]
fn pedals_and_amps_have_three_live_knobs_and_cabs_none() {
    for family in FAMILIES {
        let id = family.skin_id().unwrap();
        let manifest = skins::manifest(id).unwrap();
        let mut controls: Vec<_> = manifest.knobs.iter().map(|knob| knob.control).collect();
        controls.sort();
        if family.category() == GearCategory::Cab {
            assert!(controls.is_empty(), "{id}");
        } else {
            assert_eq!(
                controls,
                [KnobControl::Gain, KnobControl::Output, KnobControl::Mix],
                "{id}"
            );
        }
    }
}

#[test]
fn knobs_sit_inside_the_body_and_their_strips_hold_every_frame() {
    for family in FAMILIES {
        let id = family.skin_id().unwrap();
        let manifest = skins::manifest(id).unwrap();
        for knob in &manifest.knobs {
            let half = knob.side / 2.0;
            assert!(knob.center[0] - half >= 0.0 && knob.center[0] + half <= manifest.width as f32);
            assert!(
                knob.center[1] - half >= 0.0 && knob.center[1] + half <= manifest.height as f32
            );
            let strip = skins::knob_strip(id, knob.control).expect("strip");
            let rows = knob.frames.div_ceil(knob.columns) as usize;
            assert_eq!(
                strip.size,
                [
                    knob.columns as usize * knob.side as usize,
                    rows * knob.side as usize
                ],
                "{id} {:?}",
                knob.control
            );
        }
    }
}

#[test]
fn a_frame_uv_walks_the_strip_row_by_row() {
    let manifest = skins::manifest("od-green").unwrap();
    let knob = &manifest.knobs[0];
    let first = knob.frame_uv(0.0);
    assert_eq!(first.min, eframe::egui::pos2(0.0, 0.0));
    let last = knob.frame_uv(1.0);
    assert!((last.max.x - 1.0).abs() < 1e-6 && (last.max.y - 1.0).abs() < 1e-6);
    let ninth = knob.frame_uv(8.0 / 31.0);
    assert!((ninth.min.x).abs() < 1e-6 && (ninth.min.y - 0.25).abs() < 1e-6);
}
