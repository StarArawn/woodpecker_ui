use bevy::reflect::Reflect;

/// Units which describe spacing and size
#[derive(Debug, Reflect, Clone, Copy, PartialEq)]
pub enum Units {
    /// A number of pixels
    Pixels(f32),
    /// A percentage of the parent dimension
    /// between 0.0 and 100.0
    Percentage(f32),
    /// Automatically determine the value
    Auto,
}

impl Default for Units {
    fn default() -> Self {
        Units::Auto
    }
}

impl From<f32> for Units {
    fn from(value: f32) -> Self {
        Units::Pixels(value)
    }
}

impl Units {
    /// Converts the units to an f32 value
    pub fn value_or(&self, auto: f32) -> f32 {
        match self {
            Units::Pixels(pixels) => *pixels,
            Units::Percentage(percentage) => percentage / 100.0,
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
