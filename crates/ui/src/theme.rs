//! Shared dark theme tokens for every native surface.

/// Colours taken from the Ronnie logo: chrome lettering over a flame ramp.
pub mod palette {
    use eframe::egui::Color32;

    /// Deep red at the cool edge of the flames.
    pub const EMBER: Color32 = Color32::from_rgb(0x90, 0x14, 0x10);
    /// Saturated flame red. Errors and clipping use this colour.
    pub const FIRE: Color32 = Color32::from_rgb(0xd8, 0x1e, 0x10);
    /// Orange-red flame. Active controls and accents use this colour.
    pub const FLAME: Color32 = Color32::from_rgb(0xf0, 0x50, 0x10);
    /// Orange flame. Warnings, links, and lit chips use this colour.
    pub const BLAZE: Color32 = Color32::from_rgb(0xf5, 0x96, 0x14);
    /// Hot center of the flames.
    pub const HOT: Color32 = Color32::from_rgb(0xff, 0xd8, 0x70);
    /// Silver of the logo lettering.
    pub const CHROME: Color32 = Color32::from_rgb(0xd0, 0xd0, 0xd4);
    /// Unlit tuner bars, a burnt-out ember.
    pub const COOLED: Color32 = Color32::from_rgb(0x2a, 0x10, 0x0c);

    /// Fill of a floating panel, a card, or a tile body.
    pub const PANEL: Color32 = Color32::from_rgb(21, 21, 23);
    /// Fill of a raised surface, such as a chip, a placeholder, or a knob face.
    pub const SURFACE_RAISED: Color32 = Color32::from_rgb(28, 28, 30);
    /// Solid edge of a floating menu, a field, or a card.
    pub const PANEL_EDGE: Color32 = Color32::from_rgb(63, 63, 70);
    /// Translucent hairline border.
    pub const HAIRLINE: Color32 = Color32::from_rgba_unmultiplied_const(84, 84, 88, 166);
    /// Fill of the model list.
    pub const LIST_FILL: Color32 = Color32::from_rgb(57, 57, 61);

    /// Solid grey for muted labels, line icons, and placeholder avatars.
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(141, 141, 147);
    /// Dim grey for captions, hints, and labels that are not selected or not enabled.
    pub const TEXT_SUBTLE: Color32 = Color32::from_rgb(93, 93, 97);
    /// Translucent label for secondary copy, such as status text and empty-state text.
    pub const TEXT_SECONDARY: Color32 = Color32::from_rgba_unmultiplied_const(235, 235, 245, 153);
    /// Faint label on a graph axis or a graph marker.
    pub const GRAPH_LABEL: Color32 = Color32::from_rgba_unmultiplied_const(235, 235, 245, 89);
    /// Faint light fill for a selected pill or an unlit meter cell.
    pub const TINT_FILL: Color32 = Color32::from_rgba_unmultiplied_const(235, 235, 245, 46);
    /// Translucent fill of a control track or a pill.
    pub const CONTROL_FILL: Color32 = Color32::from_rgba_unmultiplied_const(120, 120, 128, 92);
    /// Light fill of a row or an icon button under the pointer.
    pub const HOVER_FILL: Color32 = Color32::from_rgba_premultiplied(20, 20, 20, 20);
    /// Fill of an icon plate on a tile under the pointer.
    pub const PLATE_HOVER: Color32 = Color32::from_rgb(70, 70, 76);
    /// Dark veil on artwork, under an icon plate or over a tile that is not ready.
    pub const ART_SCRIM: Color32 = Color32::from_black_alpha(166);
    /// Dark veil over a card or a toggle that is disabled or locked.
    pub const DISABLED_VEIL: Color32 = Color32::from_black_alpha(140);
    /// Dim layer behind a modal or over a busy card.
    pub const BACKDROP_DIM: Color32 = Color32::from_black_alpha(128);
    /// Veil over a faceplate knob that is switched off.
    pub const KNOB_VEIL: Color32 = Color32::from_rgba_unmultiplied_const(28, 28, 30, 140);
    /// Light metal of a knob cap highlight and of its pointer edge.
    pub const KNOB_METAL: Color32 = Color32::from_rgb(169, 169, 169);

    /// The one colour outside the logo: a perfectly tuned note.
    pub const IN_TUNE: Color32 = Color32::from_rgb(0x22, 0xd0, 0x5a);
    /// Tuner note that is close to the target.
    pub const TUNE_NEAR: Color32 = BLAZE;
    /// Tuner note that is far from the target.
    pub const TUNE_FAR: Color32 = FIRE;

    /// Maps 0 (quiet) to 1 (hot) along the flames of the logo.
    #[must_use]
    pub fn fire_ramp(position: f32) -> Color32 {
        const STOPS: [(f32, Color32); 5] = [
            (0.0, EMBER),
            (0.35, FIRE),
            (0.6, FLAME),
            (0.82, BLAZE),
            (1.0, HOT),
        ];
        let position = position.clamp(0.0, 1.0);
        for pair in STOPS.windows(2) {
            let [(from_at, from), (to_at, to)] = [pair[0], pair[1]];
            if position <= to_at {
                let fraction = (position - from_at) / (to_at - from_at);
                let mix = |a: u8, b: u8| {
                    (f32::from(a) + (f32::from(b) - f32::from(a)) * fraction).round() as u8
                };
                return Color32::from_rgb(
                    mix(from.r(), to.r()),
                    mix(from.g(), to.g()),
                    mix(from.b(), to.b()),
                );
            }
        }
        HOT
    }
}

/// Compatibility palette and common control dimensions.
#[derive(Debug, Clone, PartialEq)]
pub struct Theme {
    /// Root black.
    pub black: [u8; 4],
    /// Primary white.
    pub white: [u8; 4],
    /// Card body background.
    pub surface: [u8; 4],
    /// Raised controls.
    pub surface_raised: [u8; 4],
    /// Floating panel background.
    pub panel: [u8; 4],
    /// Muted text and pressed fills.
    pub muted: [u8; 4],
    /// Subtle labels.
    pub subtle: [u8; 4],
    /// Hairline borders.
    pub border: [u8; 4],
    /// Link accent.
    pub link: [u8; 4],
    /// Cool logo accent.
    pub brand_ember: [u8; 4],
    /// Warm logo accent.
    pub brand_blaze: [u8; 4],
    /// Hot logo accent.
    pub brand_fire: [u8; 4],
    /// Floating-panel corner radius.
    pub panel_corner: f32,
    /// Icon glyph size inside a chrome box.
    pub icon_size: i32,
    /// Standard icon-button hit target.
    pub icon_box_size: i32,
    /// Primary knob footprint.
    pub knob_primary: i32,
    /// Secondary knob footprint.
    pub knob_secondary: i32,
    /// Disabled control opacity.
    pub disabled_opacity: f32,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            black: [0, 0, 0, 255],
            white: [255, 255, 255, 255],
            surface: [0x15, 0x15, 0x17, 0xff],
            surface_raised: [0x1c, 0x1c, 0x1e, 0xff],
            panel: [0x14, 0x14, 0x16, 0xff],
            muted: [235, 235, 245, 153],
            subtle: [235, 235, 245, 102],
            border: [84, 84, 88, 166],
            link: palette::BLAZE.to_array(),
            brand_ember: palette::EMBER.to_array(),
            brand_blaze: palette::BLAZE.to_array(),
            brand_fire: palette::FIRE.to_array(),
            panel_corner: 14.0,
            icon_size: 14,
            icon_box_size: 20,
            knob_primary: 48,
            knob_secondary: 36,
            disabled_opacity: 0.45,
        }
    }
}

use eframe::egui;

/// Colours for one layout, looked up at run time so that a layout can turn light or dark.
///
/// The reference layout uses [`Look::reference`]; the Archetype layout installs
/// [`Look::archetype`] before it draws. Code that opts in reads the active look with
/// [`Look::get`]; everything else keeps the constant [`palette`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Look {
    /// Window background behind every panel.
    pub background: egui::Color32,
    /// Raised panels such as the global controls.
    pub panel: egui::Color32,
    /// Hairlines and panel edges.
    pub edge: egui::Color32,
    /// Primary text.
    pub text: egui::Color32,
    /// Secondary text such as values and captions.
    pub text_muted: egui::Color32,
    /// The utility bar at the bottom of the window.
    pub utility: egui::Color32,
    /// Text on the utility bar.
    pub utility_text: egui::Color32,
    /// Accent for active states.
    pub accent: egui::Color32,
    /// Chain icon of the selected block.
    pub icon_selected: egui::Color32,
    /// Chain icon of an active block.
    pub icon_on: egui::Color32,
    /// Chain icon of a bypassed block.
    pub icon_bypassed: egui::Color32,
}

impl Look {
    /// Today's dark colours.
    #[must_use]
    pub const fn reference() -> Self {
        Self {
            background: egui::Color32::BLACK,
            panel: palette::SURFACE_RAISED,
            edge: palette::HAIRLINE,
            text: egui::Color32::WHITE,
            text_muted: palette::TEXT_MUTED,
            utility: egui::Color32::BLACK,
            utility_text: egui::Color32::WHITE,
            accent: palette::FIRE,
            icon_selected: egui::Color32::WHITE,
            icon_on: palette::TEXT_MUTED,
            icon_bypassed: palette::TEXT_SUBTLE,
        }
    }

    /// The light Archetype colours.
    #[must_use]
    pub const fn archetype() -> Self {
        Self {
            background: egui::Color32::from_rgb(0xf3, 0xf3, 0xf1),
            panel: egui::Color32::from_rgb(0xfb, 0xfb, 0xfa),
            edge: egui::Color32::from_rgb(0xd9, 0xd9, 0xd6),
            text: egui::Color32::from_rgb(0x11, 0x11, 0x11),
            text_muted: egui::Color32::from_rgb(0x5f, 0x5f, 0x5c),
            utility: egui::Color32::from_rgb(0x0b, 0x0b, 0x0b),
            utility_text: egui::Color32::from_rgb(0xf2, 0xf2, 0xf2),
            accent: egui::Color32::from_rgb(0xff, 0x6a, 0x1a),
            icon_selected: egui::Color32::from_rgb(0x11, 0x11, 0x11),
            icon_on: egui::Color32::from_rgb(0x6b, 0x6b, 0x68),
            icon_bypassed: egui::Color32::from_rgb(0xb9, 0xb9, 0xb6),
        }
    }

    fn id() -> egui::Id {
        egui::Id::unique("ronie-look")
    }

    /// Makes `look` the active look for this context.
    pub fn install(ctx: &egui::Context, look: Self) {
        ctx.data_mut(|data| data.insert_temp(Self::id(), look));
    }

    /// The active look, or the reference look when none was installed.
    #[must_use]
    pub fn get(ctx: &egui::Context) -> Self {
        ctx.data(|data| data.get_temp(Self::id()))
            .unwrap_or_else(Self::reference)
    }
}
