use bevy::{math::Vec2, prelude::Component};
use taffy::{AvailableSpace, MaybeMath, MaybeResolve};

/// A `Measure` is used to compute the size of a ui node
/// when the size of that node is based on its content.
pub trait Measure: Send + Sync + 'static {
    /// Calculate the size of the node given the constraints.
    fn measure(
        &self,
        width: Option<f32>,
        height: Option<f32>,
        available_width: AvailableSpace,
        available_height: AvailableSpace,
        style: &taffy::Style,
    ) -> Vec2;
}

/// A type to serve as Taffy's node context (which allows the content size of leaf nodes to be computed)
///
/// It has specific variants for common built-in types to avoid making them opaque and needing to box them
/// by wrapping them in a closure and a Custom variant that allows arbitrary measurement closures if required.
#[derive(Component, Default)]
pub enum LayoutMeasure {
    #[default]
    ContentSize,
    #[allow(dead_code)]
    Fixed(FixedMeasure),
    #[allow(dead_code)]
    Custom(Box<dyn Measure>),
    Image(ImageMeasure),
}

impl Measure for LayoutMeasure {
    fn measure(
        &self,
        width: Option<f32>,
        height: Option<f32>,
        available_width: AvailableSpace,
        available_height: AvailableSpace,
        style: &taffy::Style,
    ) -> Vec2 {
        match self {
            LayoutMeasure::ContentSize => {
                let width = width.unwrap_or(match available_width {
                    AvailableSpace::Definite(width) => width,
                    _ => 0.0,
                });
                let height = height.unwrap_or(match available_height {
                    AvailableSpace::Definite(height) => height,
                    _ => 0.0,
                });
                Vec2::new(width, height)
            }
            LayoutMeasure::Fixed(fixed) => {
                fixed.measure(width, height, available_width, available_height, style)
            }
            LayoutMeasure::Custom(custom) => {
                custom.measure(width, height, available_width, available_height, style)
            }
            LayoutMeasure::Image(image) => {
                image.measure(width, height, available_width, available_height, style)
            }
        }
    }
}

/// A `FixedMeasure` is a `Measure` that ignores all constraints and
/// always returns the same size.
#[derive(Default, Clone, Debug)]
pub struct FixedMeasure {
    pub size: Vec2,
}

impl Measure for FixedMeasure {
    fn measure(
        &self,
        _: Option<f32>,
        _: Option<f32>,
        _: AvailableSpace,
        _: AvailableSpace,
        _: &taffy::Style,
    ) -> Vec2 {
        self.size
    }
}

#[derive(Default, Clone, Debug)]
pub struct ImageMeasure {
    pub size: Vec2,
}

impl Measure for ImageMeasure {
    fn measure(
        &self,
        width: Option<f32>,
        height: Option<f32>,
        available_width: AvailableSpace,
        available_height: AvailableSpace,
        style: &taffy::Style,
    ) -> Vec2 {
        // Convert available width/height into an option
        let parent_width = available_width.into_option();
        let parent_height = available_height.into_option();

        // Resolve styles
        let s_aspect_ratio = style.aspect_ratio;
        let s_width = style.size.width.maybe_resolve(parent_width);
        let s_min_width = style.min_size.width.maybe_resolve(parent_width);
        let s_max_width = style.max_size.width.maybe_resolve(parent_width);
        let s_height = style.size.height.maybe_resolve(parent_height);
        let s_min_height = style.min_size.height.maybe_resolve(parent_height);
        let s_max_height: Option<f32> = style.max_size.height.maybe_resolve(parent_height);

        // Determine width and height from styles and known_sizes (if a size is available
        // from any of these sources)
        let width = width.or(s_width
            .or(s_min_width)
            .maybe_clamp(s_min_width, s_max_width));
        let height = height.or(s_height
            .or(s_min_height)
            .maybe_clamp(s_min_height, s_max_height));

        // Use aspect_ratio from style, fall back to inherent aspect ratio
        let aspect_ratio = s_aspect_ratio.unwrap_or_else(|| self.size.x / self.size.y);

        // Apply aspect ratio
        // If only one of width or height was determined at this point, then the other is set beyond this point using the aspect ratio.
        let taffy_size = taffy::Size { width, height }.maybe_apply_aspect_ratio(Some(aspect_ratio));

        // Use computed sizes or fall back to image's inherent size
        Vec2 {
            x: taffy_size
                .width
                .unwrap_or(self.size.x)
                .maybe_clamp(s_min_width, s_max_width),
            y: taffy_size
                .height
                .unwrap_or(self.size.y)
                .maybe_clamp(s_min_height, s_max_height),
        }
    }
}
