use bevy_vello::vello::{
    self,
    glyph::skrifa::{FontRef, MetadataProvider},
};

use crate::render::VARIATIONS;

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
