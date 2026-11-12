//! Rendered gear skins: the embedded renders from `tools/render`, decoded once per context, and
//! drawn with knobs that turn.
//!
//! Each skin has a transparent body, a filmstrip of 32 frames for each live knob, and a manifest
//! that says where each knob (and, on a pedal, the blank LCD glass) sits on the body, in body
//! pixels.

use std::collections::HashMap;
use std::sync::Arc;

use eframe::egui::{self, Color32, ColorImage, Pos2, Rect, TextureHandle, TextureOptions};
use ronie_core::ChainBlock;

/// The block setting a live knob turns.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum KnobControl {
    /// "GAIN IN": the block's input gain.
    Gain,
    /// "OUTPUT": the block's output gain.
    Output,
    /// "MIX": the block's dry/wet mix.
    Mix,
}

impl KnobControl {
    fn parse(text: &str) -> Option<Self> {
        match text {
            "gain" => Some(Self::Gain),
            "output" => Some(Self::Output),
            "mix" => Some(Self::Mix),
            _ => None,
        }
    }
}

/// One live knob on a skin.
#[derive(Clone, Debug, PartialEq)]
pub struct SkinKnob {
    /// The setting it turns.
    pub control: KnobControl,
    /// The text printed on the render, such as "GAIN IN".
    pub label: String,
    /// Centre of the knob's square crop, in body pixels.
    pub center: [f32; 2],
    /// Side of the square crop, in body pixels.
    pub side: f32,
    /// Number of frames, from position 0 to position 1.
    pub frames: u32,
    /// Frames per row in the strip.
    pub columns: u32,
}

impl SkinKnob {
    /// The UV rectangle of the strip frame nearest `position`, a knob position from 0 to 1.
    #[must_use]
    pub fn frame_uv(&self, position: f32) -> Rect {
        let last = self.frames.saturating_sub(1);
        let index = (position.clamp(0.0, 1.0) * last as f32).round() as u32;
        let columns = self.columns.max(1);
        let rows = self.frames.div_ceil(columns).max(1);
        let (col, row) = (index % columns, index / columns);
        let size = egui::vec2(1.0 / columns as f32, 1.0 / rows as f32);
        let min = Pos2::new(col as f32 * size.x, row as f32 * size.y);
        Rect::from_min_size(min, size)
    }

    /// Where the knob's crop sits when the body is drawn in `body`.
    #[must_use]
    pub fn rect_in(&self, body: Rect, manifest: &SkinManifest) -> Rect {
        let scale = body.width() / manifest.width as f32;
        Rect::from_center_size(
            body.min + egui::vec2(self.center[0], self.center[1]) * scale,
            egui::Vec2::splat(self.side * scale),
        )
    }
}

/// The blank LCD glass on a pedal skin, where the app draws the active capture's name.
#[derive(Clone, Debug, PartialEq)]
pub struct SkinLcd {
    /// Centre of the glass, in body pixels.
    pub center: [f32; 2],
    /// Width and height of the glass's axis-aligned bounds, in body pixels.
    pub size: [f32; 2],
}

impl SkinLcd {
    /// Where the glass sits when the body is drawn in `body`.
    #[must_use]
    pub fn rect_in(&self, body: Rect, manifest: &SkinManifest) -> Rect {
        let scale = body.width() / manifest.width as f32;
        Rect::from_center_size(
            body.min + egui::vec2(self.center[0], self.center[1]) * scale,
            egui::vec2(self.size[0], self.size[1]) * scale,
        )
    }
}

/// Size of a skin's body, its live knobs and its LCD.
#[derive(Clone, Debug, PartialEq)]
pub struct SkinManifest {
    /// Skin identity, as from [`ronie_core::gear::GearFamily::skin_id`].
    pub id: String,
    /// Body width in pixels.
    pub width: u32,
    /// Body height in pixels.
    pub height: u32,
    /// Live knobs.
    pub knobs: Vec<SkinKnob>,
    /// The LCD glass, on pedals.
    pub lcd: Option<SkinLcd>,
}

impl SkinManifest {
    fn parse(text: &str) -> Option<Self> {
        let value: serde_json::Value = serde_json::from_str(text).ok()?;
        let number = |value: &serde_json::Value| value.as_f64().map(|n| n as f32);
        let knobs = value
            .get("knobs")?
            .as_array()?
            .iter()
            .map(|knob| {
                let center = knob.get("center")?.as_array()?;
                Some(SkinKnob {
                    control: KnobControl::parse(knob.get("control")?.as_str()?)?,
                    label: knob.get("label")?.as_str()?.to_owned(),
                    center: [number(center.first()?)?, number(center.get(1)?)?],
                    side: number(knob.get("side")?)?,
                    frames: u32::try_from(knob.get("frames")?.as_u64()?).ok()?,
                    columns: u32::try_from(knob.get("columns")?.as_u64()?).ok()?,
                })
            })
            .collect::<Option<Vec<_>>>()?;
        // A missing or malformed LCD leaves the skin without one; the body and knobs still draw.
        let lcd = value.get("lcd").and_then(|lcd| {
            let pair = |key: &str| -> Option<[f32; 2]> {
                let pair = lcd.get(key)?.as_array()?;
                Some([number(pair.first()?)?, number(pair.get(1)?)?])
            };
            Some(SkinLcd {
                center: pair("center")?,
                size: pair("size")?,
            })
        });
        Some(Self {
            id: value.get("id")?.as_str()?.to_owned(),
            width: u32::try_from(value.get("width")?.as_u64()?).ok()?,
            height: u32::try_from(value.get("height")?.as_u64()?).ok()?,
            knobs,
            lcd,
        })
    }

    /// The largest rectangle with the body's aspect inside `area`, no taller than `max_height`,
    /// centred in `area`.
    #[must_use]
    pub fn fit(&self, area: Rect, max_height: f32) -> Rect {
        let aspect = self.width as f32 / self.height.max(1) as f32;
        let mut height = area.height().min(max_height);
        let mut width = height * aspect;
        if width > area.width() {
            width = area.width();
            height = width / aspect;
        }
        Rect::from_center_size(area.center(), egui::vec2(width, height))
    }
}

struct SkinAsset {
    id: &'static str,
    manifest: &'static str,
    body: &'static [u8],
    gain: Option<&'static [u8]>,
    output: Option<&'static [u8]>,
    mix: Option<&'static [u8]>,
}

macro_rules! skin_asset {
    ($id:literal) => {
        SkinAsset {
            id: $id,
            manifest: include_str!(concat!("../../assets/skins/", $id, "/skin.json")),
            body: include_bytes!(concat!("../../assets/skins/", $id, "/body.webp")),
            gain: None,
            output: None,
            mix: None,
        }
    };
    ($id:literal, knobs) => {
        SkinAsset {
            gain: Some(include_bytes!(concat!(
                "../../assets/skins/",
                $id,
                "/knob-gain.webp"
            ))),
            output: Some(include_bytes!(concat!(
                "../../assets/skins/",
                $id,
                "/knob-output.webp"
            ))),
            mix: Some(include_bytes!(concat!(
                "../../assets/skins/",
                $id,
                "/knob-mix.webp"
            ))),
            ..skin_asset!($id)
        }
    };
}

const SKINS: &[SkinAsset] = &[
    skin_asset!("od-green", knobs),
    skin_asset!("od-gold", knobs),
    skin_asset!("dist-black", knobs),
    skin_asset!("fuzz", knobs),
    skin_asset!("boost", knobs),
    skin_asset!("comp", knobs),
    skin_asset!("default-dark", knobs),
    skin_asset!("default-light", knobs),
    skin_asset!("highgain", knobs),
    skin_asset!("modern-us", knobs),
    skin_asset!("british", knobs),
    skin_asset!("chime", knobs),
    skin_asset!("clean", knobs),
    skin_asset!("bass", knobs),
    skin_asset!("default-clean", knobs),
    skin_asset!("default-crunch", knobs),
    skin_asset!("default-hi-gain", knobs),
    skin_asset!("cab-412-modern"),
    skin_asset!("cab-412-vintage"),
    skin_asset!("cab-212-chime"),
    skin_asset!("cab-112-clean"),
    skin_asset!("cab-810-bass"),
    skin_asset!("cab-default"),
];

fn asset(id: &str) -> Option<&'static SkinAsset> {
    SKINS.iter().find(|asset| asset.id == id)
}

fn decode(bytes: &[u8]) -> Option<ColorImage> {
    let image = image::load_from_memory_with_format(bytes, image::ImageFormat::WebP)
        .ok()?
        .to_rgba8();
    let size = [image.width() as usize, image.height() as usize];
    Some(ColorImage::from_rgba_unmultiplied(size, image.as_raw()))
}

/// The manifest of a skin, if the app has its render.
#[must_use]
pub fn manifest(id: &str) -> Option<SkinManifest> {
    SkinManifest::parse(asset(id)?.manifest)
}

/// The decoded body of a skin.
#[must_use]
pub fn body_image(id: &str) -> Option<ColorImage> {
    decode(asset(id)?.body)
}

/// The decoded filmstrip of one live knob of a skin.
#[must_use]
pub fn knob_strip(id: &str, control: KnobControl) -> Option<ColorImage> {
    let asset = asset(id)?;
    decode(match control {
        KnobControl::Gain => asset.gain?,
        KnobControl::Output => asset.output?,
        KnobControl::Mix => asset.mix?,
    })
}

/// The skin of a block's gear, when the app has a render for its family.
#[must_use]
pub fn skin_for(block: &ChainBlock) -> Option<BlockSkin> {
    use ronie_core::gear::{GearCategory, GearHints, identify};

    let tone = block.tone.as_ref()?;
    let model = block
        .active_model_id
        .and_then(|id| tone.models.iter().find(|model| model.id == id));
    let family = identify(&GearHints::from_tone(tone, model)).family;
    let id = asset(family.skin_id()?)?.id;
    let label = match family.category() {
        GearCategory::Pedal => "Pedal render",
        GearCategory::Cab => "Cab render",
        _ => "Amp render",
    };
    Some(BlockSkin {
        id,
        label,
        category: family.category(),
    })
}

/// The skin a block shows, and the accessible name of its render.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlockSkin {
    /// Skin identity.
    pub id: &'static str,
    /// "Pedal render", "Amp render" or "Cab render".
    pub label: &'static str,
    /// The kind of gear the render shows.
    pub category: ronie_core::gear::GearCategory,
}

/// A skin's textures, uploaded once per context.
pub struct SkinTextures {
    /// Body size and knobs.
    pub manifest: SkinManifest,
    /// The body.
    pub body: TextureHandle,
    /// The filmstrip of each live knob.
    pub strips: Vec<(KnobControl, TextureHandle)>,
}

impl SkinTextures {
    /// The filmstrip of one live knob.
    #[must_use]
    pub fn strip(&self, control: KnobControl) -> Option<&TextureHandle> {
        self.strips
            .iter()
            .find(|(each, _)| *each == control)
            .map(|(_, texture)| texture)
    }
}

type SkinCache = HashMap<&'static str, Option<Arc<SkinTextures>>>;

/// The textures of a skin, decoded and uploaded on first use. `None` when the skin has no render
/// or its files do not decode.
#[must_use]
pub fn textures(ctx: &egui::Context, id: &str) -> Option<Arc<SkinTextures>> {
    let asset = asset(id)?;
    let cache_id = egui::Id::unique("ronie-skin-textures");
    if let Some(cached) = ctx.data(|data| {
        data.get_temp::<SkinCache>(cache_id)
            .and_then(|cache| cache.get(asset.id).cloned())
    }) {
        return cached;
    }
    let loaded = load(ctx, asset).map(Arc::new);
    ctx.data_mut(|data| {
        data.get_temp_mut_or_default::<SkinCache>(cache_id)
            .insert(asset.id, loaded.clone());
    });
    loaded
}

fn load(ctx: &egui::Context, asset: &SkinAsset) -> Option<SkinTextures> {
    let manifest = SkinManifest::parse(asset.manifest)?;
    let options = TextureOptions::LINEAR;
    let body = ctx.load_texture(format!("skin-{}", asset.id), decode(asset.body)?, options);
    let strips = manifest
        .knobs
        .iter()
        .filter_map(|knob| {
            let image = knob_strip(asset.id, knob.control)?;
            let name = format!("skin-{}-{:?}", asset.id, knob.control);
            Some((knob.control, ctx.load_texture(name, image, options)))
        })
        .collect();
    Some(SkinTextures {
        manifest,
        body,
        strips,
    })
}

/// Draws a skin's body in `body`, with each live knob at the position `position` gives it.
pub fn paint(
    painter: &egui::Painter,
    skin: &SkinTextures,
    body: Rect,
    position: impl Fn(KnobControl) -> f32,
    tint: Color32,
) {
    let full = Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0));
    painter.image(skin.body.id(), body, full, tint);
    for knob in &skin.manifest.knobs {
        if let Some(strip) = skin.strip(knob.control) {
            painter.image(
                strip.id(),
                knob.rect_in(body, &skin.manifest),
                knob.frame_uv(position(knob.control)),
                tint,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE: &str = r#""id": "x", "width": 600, "height": 800, "knobs": []"#;

    #[test]
    fn a_malformed_lcd_is_dropped_and_the_rest_of_the_manifest_kept() {
        for lcd in [
            r#""lcd": 3"#,
            r#""lcd": {"center": [1, 2]}"#,
            r#""lcd": {"center": [1], "size": [3, 4]}"#,
            r#""lcd": {"center": [1, 2], "size": ["a", 4]}"#,
        ] {
            let manifest = SkinManifest::parse(&format!("{{{BASE}, {lcd}}}"))
                .unwrap_or_else(|| panic!("{lcd} dropped the manifest"));
            assert_eq!(manifest.width, 600);
            assert!(manifest.lcd.is_none(), "{lcd}");
        }
    }

    #[test]
    fn a_well_formed_lcd_is_parsed() {
        let text = format!(r#"{{{BASE}, "lcd": {{"center": [1.5, 2], "size": [3, 4.5]}}}}"#);
        let manifest = SkinManifest::parse(&text).unwrap();
        assert_eq!(
            manifest.lcd,
            Some(SkinLcd {
                center: [1.5, 2.0],
                size: [3.0, 4.5],
            })
        );
    }
}
