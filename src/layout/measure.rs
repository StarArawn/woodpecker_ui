use bevy::{math::Vec2, prelude::Component};
use taffy::AvailableSpace;

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
