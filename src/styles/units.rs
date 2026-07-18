use bevy::reflect::Reflect;

/// Units which describe spacing and size
#[derive(Debug, Default, Reflect, Clone, Copy, PartialEq)]
pub enum Units {
    /// A number of pixels
    Pixels(f32),
    /// A percentage of the parent dimension
    /// between 0.0 and 100.0
    Percentage(f32),
    /// A percentage of the parent dimension (0.0..=100.0) combined with a fixed pixel
    /// offset -- e.g. `Calc { percent: 100.0, pixels: -40.0 }` for CSS's `calc(100% - 40px)`.
    /// Unlike `Percentage` (resolved lazily by taffy during layout), this is resolved
    /// eagerly against the parent's own last-committed layout size before the style ever
    /// reaches taffy, since taffy has no primitive for combining a percentage with a fixed
    /// offset in a single value -- see [`super::WoodpeckerStyle::resolve_calc`].
    Calc {
        /// The percentage component, 0.0..=100.0.
        percent: f32,
        /// The fixed pixel offset added to the resolved percentage.
        pixels: f32,
    },
    /// Automatically determine the value
    #[default]
    Auto,
}

impl From<f32> for Units {
    fn from(value: f32) -> Self {
        Units::Pixels(value)
    }
}

impl Units {
    /// Converts the units to an f32 value. `Calc` falls back to `auto` here, the same as
    /// `Auto` -- by the time a style reaches code that calls this, `Calc` should already
    /// have been eliminated by [`super::WoodpeckerStyle::resolve_calc`], since resolving it
    /// needs a parent-size basis this method doesn't have.
    pub fn value_or(&self, auto: f32) -> f32 {
        match self {
            Units::Pixels(pixels) => *pixels,
            Units::Percentage(percentage) => percentage / 100.0,
            Units::Calc { .. } | Units::Auto => auto,
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

    /// Returns true if the value is a `Calc { percent, pixels }` combination.
    pub fn is_calc(&self) -> bool {
        matches!(self, Units::Calc { .. })
    }

    /// Returns true if the value is auto
    pub fn is_auto(&self) -> bool {
        matches!(self, Units::Auto)
    }

    /// Resolves a `Calc` value against `basis` (the relevant axis's parent size, in pixels),
    /// collapsing it to a concrete `Pixels`. Every other variant passes through unchanged --
    /// `Percentage`/`Auto` are resolved later, by taffy itself during layout.
    pub fn resolve_calc(self, basis: f32) -> Units {
        match self {
            Units::Calc { percent, pixels } => Units::Pixels(percent / 100.0 * basis + pixels),
            other => other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_calc_combines_percent_of_basis_with_the_fixed_pixel_offset() {
        let calc = Units::Calc {
            percent: 100.0,
            pixels: -40.0,
        };
        assert_eq!(calc.resolve_calc(300.0), Units::Pixels(260.0));
    }

    #[test]
    fn resolve_calc_leaves_every_other_variant_unchanged() {
        assert_eq!(Units::Pixels(10.0).resolve_calc(300.0), Units::Pixels(10.0));
        assert_eq!(
            Units::Percentage(50.0).resolve_calc(300.0),
            Units::Percentage(50.0)
        );
        assert_eq!(Units::Auto.resolve_calc(300.0), Units::Auto);
    }

    #[test]
    fn is_calc_only_matches_the_calc_variant() {
        assert!(Units::Calc {
            percent: 0.0,
            pixels: 0.0
        }
        .is_calc());
        assert!(!Units::Pixels(0.0).is_calc());
        assert!(!Units::Percentage(0.0).is_calc());
        assert!(!Units::Auto.is_calc());
    }
}
