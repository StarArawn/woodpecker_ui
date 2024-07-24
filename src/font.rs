use bevy::{prelude::*, utils::HashMap};
use bevy_vello::{
    text::VelloFont,
    vello::{
        self,
        glyph::skrifa::{FontRef, MetadataProvider},
    },
};
use cosmic_text::Family;

use crate::{prelude::WoodpeckerStyle, render::VARIATIONS};

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

/// Returns the width of the font with the given content string
pub fn measure_width(font: &FontRef, content: &str, font_size: f32) -> f32 {
    let font_size = vello::skrifa::instance::Size::new(font_size);
    let charmap = font.charmap();
    let axes = font.axes();
    let var_loc = axes.location(VARIATIONS);
    let glyph_metrics = font.glyph_metrics(font_size, &var_loc);
    let mut width = 0.0;
    content.chars().for_each(|ch| {
        if ch == '\n' {
            return;
        }
        let gid = charmap.map(ch).unwrap_or_default();
        let advance = glyph_metrics.advance_width(gid).unwrap_or_default();
        width += advance;
    });
    width
}

#[derive(Resource)]
pub(crate) struct FontManager {
    font_system: cosmic_text::FontSystem,
    font_data: HashMap<Handle<VelloFont>, Vec<u8>>,
    vello_to_family: HashMap<Handle<VelloFont>, String>,
}

impl Default for FontManager {
    fn default() -> Self {
        Self {
            font_system: cosmic_text::FontSystem::new(),
            vello_to_family: Default::default(),
            font_data: Default::default(),
        }
    }
}

impl FontManager {
    pub fn get_vello_font(&mut self, vello_font: &Handle<VelloFont>) -> FontRef {
        let font_data = self.font_data.get(vello_font).unwrap();
        let font_ref = FontRef::from_index(&font_data, 0).unwrap();
        font_ref
    }

    pub fn layout(
        &mut self,
        avaliable_space: Vec2,
        style: &WoodpeckerStyle,
        font_handle: &Handle<VelloFont>,
        content: &str,
        word_wrap: bool,
    ) -> Option<cosmic_text::Buffer> {
        if !self.vello_to_family.contains_key(font_handle) {
            return None;
        }

        // Text metrics indicate the font size and line height of a buffer
        // Per mozilla the default line height is roughly font_size * 1.2
        let metrics = cosmic_text::Metrics::new(
            style.font_size,
            style.line_height.unwrap_or(style.font_size * 1.2),
        );

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

        Some(buffer)
    }
}

pub fn load_fonts(
    mut font_manager: ResMut<FontManager>,
    mut event_reader: EventReader<AssetEvent<VelloFont>>,
    assets: Res<Assets<VelloFont>>,
) {
    for event in event_reader.read() {
        match event {
            AssetEvent::Added { id } => {
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
            _ => {}
        }
    }
}
