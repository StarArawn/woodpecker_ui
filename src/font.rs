use std::sync::Arc;

use bevy::{
    platform::collections::{HashMap, HashSet},
    prelude::*,
};
use bevy_vello::{prelude::VelloFont, vello::peniko::Brush};

use crate::{
    layout::{measure::LayoutMeasure, system::measure_text},
    prelude::WidgetLayout,
    styles::WoodpeckerStyle,
    DefaultFont,
};

/// The text alignment of the font.
#[derive(Debug, Clone, Copy, Default, Reflect, PartialEq)]
pub enum TextAlign {
    #[default]
    /// Align text to the left.
    Left,
    /// Align text to the right.
    Right,
    /// Align text in the middle.
    Center,
    /// Align text within the content.
    Justified,
    /// Align text at the end of the content.
    End,
}

/// Used to keep track of fonts and to measure text with a given font
/// Internally this uses cosmic text to layout and measure text.
#[derive(Resource)]
pub struct FontManager {
    font_data: HashMap<Handle<VelloFont>, Vec<u8>>,
    vello_to_family: HashMap<Handle<VelloFont>, String>,
    fonts: HashSet<Handle<VelloFont>>,
    /// The parley font context for parley shaping/etc..
    pub font_cx: parley::FontContext,
    /// The parley layout context for parley shaping/etc..
    pub layout_cx: parley::LayoutContext<Brush>,
}

impl Default for FontManager {
    fn default() -> Self {
        Self {
            vello_to_family: Default::default(),
            font_data: Default::default(),
            fonts: HashSet::default(),
            font_cx: parley::FontContext::new(),
            layout_cx: parley::LayoutContext::new(),
        }
    }
}

impl FontManager {
    /// Returns a parley driver for the given engine.
    pub fn driver<'a>(
        &'a mut self,
        engine: &'a mut parley::PlainEditor<Brush>,
    ) -> parley::PlainEditorDriver<'a, Brush> {
        engine.driver(&mut self.font_cx, &mut self.layout_cx)
    }

    /// Retrieves the font family name.
    pub fn get_family(&self, vello_font: &AssetId<VelloFont>) -> String {
        self.vello_to_family
            .get(&Handle::Weak(*vello_font))
            .unwrap()
            .clone()
    }

    /// Adds a font handle to the font manager to keep it alive.
    pub fn add(&mut self, handle: &Handle<VelloFont>) {
        self.fonts.insert(handle.clone());
    }

    /// Measures text for the given layout and font.
    pub fn measure(
        &mut self,
        text: &str,
        styles: &WoodpeckerStyle,
        layout: &WidgetLayout,
        default_font: &DefaultFont,
    ) -> Option<Vec2> {
        measure_text(text, styles, self, default_font, layout, Vec2::splat(1.0)).and_then(|m| {
            match m {
                LayoutMeasure::Fixed(fixed) => Some(fixed.size),
                _ => None,
            }
        })
    }
}

/// Loads vello font assets into the font manager.
pub(crate) fn load_fonts(
    mut font_manager: ResMut<FontManager>,
    mut event_reader: EventReader<AssetEvent<VelloFont>>,
    assets: Res<Assets<VelloFont>>,
) {
    for event in event_reader.read() {
        if let AssetEvent::LoadedWithDependencies { id } = event {
            let Some(font_asset) = assets.get(*id) else {
                continue;
            };
            let font_data: &[u8] = &font_asset.bytes;
            let font_data = font_data.to_vec();

            let face = ttf_parser::Face::parse(&font_data, 0).unwrap();
            let family = face
                .names()
                .into_iter()
                .find(|name| name.name_id == ttf_parser::name_id::FAMILY)
                .expect("Couldn't find font family.");

            let font_family = if family.is_unicode() {
                family
                    .to_string()
                    .expect("Couldn't get string from family name.")
            } else {
                String::from_utf8(family.name.to_vec())
                    .expect("Couldn't get string from family name.")
            };

            info!("Loaded font family: {}", font_family);

            font_manager.font_cx.collection.register_fonts(
                parley::fontique::Blob::new(Arc::new(font_data.clone())),
                None,
            );

            font_manager
                .vello_to_family
                .insert(Handle::Weak(*id), font_family);

            font_manager.font_data.insert(Handle::Weak(*id), font_data);
        }
    }
}
