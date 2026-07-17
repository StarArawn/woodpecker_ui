use bevy::prelude::*;

/// A drop shadow rendered behind a widget's quad, following CSS `box-shadow`'s parameters.
///
/// Only applies to widgets with [`crate::prelude::WidgetRender::Quad`]. Renders as a single
/// blurred, uniformly-rounded rectangle -- it uses `border_radius`'s `top_left` corner as the
/// shadow's radius, since the underlying renderer only supports one radius per shadow rather
/// than four independent corners.
#[derive(Debug, Reflect, Copy, Clone, PartialEq)]
pub struct WidgetBoxShadow {
    /// Horizontal offset of the shadow from the widget's bounds, in pixels.
    pub x_offset: f32,
    /// Vertical offset of the shadow from the widget's bounds, in pixels.
    pub y_offset: f32,
    /// How far the shadow's edges extend past the widget's bounds before blurring, in pixels.
    pub spread: f32,
    /// Blur radius in pixels, following the CSS `box-shadow` blur-radius parameter.
    pub blur_radius: f32,
    /// The shadow's color.
    pub color: Color,
}

impl Default for WidgetBoxShadow {
    fn default() -> Self {
        Self {
            x_offset: 0.0,
            y_offset: 4.0,
            spread: 0.0,
            blur_radius: 12.0,
            color: Color::Srgba(Srgba {
                red: 0.0,
                green: 0.0,
                blue: 0.0,
                alpha: 0.35,
            }),
        }
    }
}

impl WidgetBoxShadow {
    /// Creates a new shadow with the given color and the rest of the fields defaulted.
    pub fn new(color: Color) -> Self {
        Self {
            color,
            ..Default::default()
        }
    }

    /// Sets the horizontal/vertical offset and returns itself.
    pub fn offset(mut self, x_offset: f32, y_offset: f32) -> Self {
        self.x_offset = x_offset;
        self.y_offset = y_offset;
        self
    }

    /// Sets the spread and returns itself.
    pub fn spread(mut self, spread: f32) -> Self {
        self.spread = spread;
        self
    }

    /// Sets the blur radius and returns itself.
    pub fn blur_radius(mut self, blur_radius: f32) -> Self {
        self.blur_radius = blur_radius;
        self
    }
}
