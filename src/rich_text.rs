use std::ops::Range;

use bevy::{color::Color, reflect::Reflect};

/// Color Text
#[derive(Default, Debug, Clone, Reflect, PartialEq)]
pub struct ColorText {
    /// The color of the text
    pub color: Color,
    /// The range of the text in the original string.
    pub range: Range<usize>,
}

/// A list of colored text
///
/// Note this does not content the actual text rather it contains
/// ranges that we use with parley to render the text.
#[derive(Debug, Default, Clone, Reflect, PartialEq)]
pub struct Highlighted {
    /// A list of a colored text ranges.
    pub color_text: Vec<ColorText>,
}

/// Rich text support
/// currently only supports colors.
#[derive(Default, Debug, Clone, Reflect)]
pub struct RichText {
    pub(crate) text: String,
    pub(crate) highlighted: Highlighted,
    current_index: usize,
}

impl RichText {
    /// Creates a new instance of rich text.
    pub fn new() -> Self {
        Self {
            text: String::new(),
            highlighted: Highlighted::default(),
            current_index: 0,
        }
    }

    /// Creates a new RichText from highlighting
    pub fn from_hightlighted(text: &str, highlighted: Highlighted) -> Self {
        Self {
            text: text.to_string(),
            highlighted,
            current_index: 0,
        }
    }

    /// Adds a new text string with a specific color
    pub fn with_color_text(mut self, text: &str, color: bevy::prelude::Color) -> Self {
        self.text = format!("{}{}", self.text, text);
        self.highlighted.color_text.push(ColorText {
            color,
            range: self.current_index..text.len(),
        });
        self.current_index += text.len();
        self
    }
}
