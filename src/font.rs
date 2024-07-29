use std::hash::{DefaultHasher, Hash, Hasher};

use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
};
use bevy_vello::{text::VelloFont, vello::glyph::skrifa::FontRef};
use cosmic_text::{Buffer, Family};

use crate::prelude::WoodpeckerStyle;

/// The text alignment of the font.
#[derive(Debug, Clone, Copy, Default, Reflect, PartialEq)]
pub enum TextAlign {
    #[default]
    Left,
    Right,
    Center,
    Justified,
    End,
}

impl From<TextAlign> for cosmic_text::Align {
    fn from(val: TextAlign) -> cosmic_text::Align {
        match val {
            TextAlign::Left => cosmic_text::Align::Left,
            TextAlign::Right => cosmic_text::Align::Right,
            TextAlign::Center => cosmic_text::Align::Center,
            TextAlign::Justified => cosmic_text::Align::Justified,
            TextAlign::End => cosmic_text::Align::End,
        }
    }
}

/// Used to keep track of fonts and to measure text with a given font
/// Internally this uses cosmic text to layout and measure text.
#[derive(Resource)]
pub struct FontManager {
    font_system: cosmic_text::FontSystem,
    font_data: HashMap<Handle<VelloFont>, Vec<u8>>,
    vello_to_family: HashMap<Handle<VelloFont>, String>,
    buffer_cache: HashMap<u64, Buffer>,
    fonts: HashSet<Handle<VelloFont>>,
}

impl Default for FontManager {
    fn default() -> Self {
        Self {
            font_system: cosmic_text::FontSystem::new(),
            vello_to_family: Default::default(),
            font_data: Default::default(),
            buffer_cache: Default::default(),
            fonts: HashSet::default(),
        }
    }
}

impl FontManager {
    /// Used for vello rendering.
    pub(crate) fn get_vello_font(&mut self, vello_font: &Handle<VelloFont>) -> FontRef {
        let font_data = self.font_data.get(vello_font).unwrap();
        let font_ref = FontRef::from_index(font_data, 0).unwrap();
        font_ref
    }

    /// Adds a font handle to the font manager to keep it alive.
    pub fn add(&mut self, handle: &Handle<VelloFont>) {
        self.fonts.insert(handle.clone());
    }

    /// Computes the layout for a given piece of text and an avaliable space.
    /// It will use line height and font size from the styles.
    /// This function returns a cosmic text buffer.
    /// It also caches the resulting cosmic text buffer that it returns.
    pub fn layout(
        &mut self,
        avaliable_space: Vec2,
        style: &WoodpeckerStyle,
        font_handle: &Handle<VelloFont>,
        content: &str,
        word_wrap: bool,
    ) -> Option<cosmic_text::Buffer> {
        // Per mozilla the default line height is roughly font_size * 1.2
        let line_height = style.line_height.unwrap_or(style.font_size * 1.2);

        let mut hasher = DefaultHasher::default();
        content.hash(&mut hasher);
        (avaliable_space.x as u32).hash(&mut hasher);
        (avaliable_space.y as u32).hash(&mut hasher);
        font_handle.hash(&mut hasher);
        (style.font_size as u32).hash(&mut hasher);
        (line_height as u32).hash(&mut hasher);
        let key = hasher.finish();

        if let Some(buffer) = self.buffer_cache.get(&key) {
            return Some(buffer.clone());
        }

        if !self.vello_to_family.contains_key(font_handle) {
            return None;
        }

        // Text metrics indicate the font size and line height of a buffer
        let metrics = cosmic_text::Metrics::new(style.font_size, line_height);

        // A Buffer provides shaping and layout for a UTF-8 string, create one per text widget
        let mut buffer = cosmic_text::Buffer::new(&mut self.font_system, metrics);

        buffer.set_size(
            &mut self.font_system,
            Some(avaliable_space.x),
            Some(avaliable_space.y),
        );

        if word_wrap {
            buffer.set_wrap(&mut self.font_system, cosmic_text::Wrap::Word);
        } else {
            buffer.set_wrap(&mut self.font_system, cosmic_text::Wrap::None);
        }

        let attrs = cosmic_text::Attrs {
            family: Family::Name(self.vello_to_family.get(font_handle).unwrap()),
            ..cosmic_text::Attrs::new()
        };

        // Add some text!
        buffer.set_text(
            &mut self.font_system,
            content,
            attrs,
            cosmic_text::Shaping::Advanced,
        );

        for buffer_line in buffer.lines.iter_mut() {
            buffer_line.set_align(style.text_alignment.map(|a| a.into()));
        }

        // Perform shaping as desired
        buffer.shape_until_scroll(&mut self.font_system, true);

        self.buffer_cache.insert(key, buffer.clone());

        Some(buffer)
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
            let font_asset = assets.get(*id).unwrap();
            let font_data: &[u8] = font_asset.font.data.data();
            let font_data = font_data.to_vec();

            let face = cosmic_text::ttf_parser::Face::parse(&font_data, 0).unwrap();
            let family = face
                .names()
                .into_iter()
                .find(|name| name.name_id == cosmic_text::ttf_parser::name_id::FAMILY)
                .expect("Couldn't find font family.");

            font_manager.vello_to_family.insert(
                Handle::Weak(*id),
                family
                    .to_string()
                    .expect("Couldn't get string from family name."),
            );

            font_manager
                .font_system
                .db_mut()
                .load_font_data(font_data.clone());

            font_manager.font_data.insert(Handle::Weak(*id), font_data);
        }
    }
}
