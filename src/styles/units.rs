use bevy::reflect::Reflect;

/// Units which describe spacing and size
#[derive(Default, Debug, Reflect, Clone, Copy, PartialEq)]
pub enum Units {
    /// A number of pixels
    Pixels(f32),
    /// A percentage of the parent dimension
    Percentage(f32),
    #[default]
    /// Automatically determine the value
    Auto,
}

impl Units {
    /// Converts the units to an f32 value
    pub fn value_or(&self, parent_value: f32, auto: f32) -> f32 {
        match self {
            Units::Pixels(pixels) => *pixels,
            Units::Percentage(percentage) => (percentage / 100.0) * parent_value,
            Units::Auto => auto,
        }
    }

    /// Returns true if the value is in pixels
    pub fn is_pixels(&self) -> bool {
        matches!(self, Units::Pixels(_))
    }

    /// Returns true if the value is a percentage
    pub fn is_percentage(&self) -> bool {
        matches!(self, Units::Percentage(_))
    }

    /// Returns true if the value is auto
    pub fn is_auto(&self) -> bool {
        matches!(self, Units::Auto)
    }
}
