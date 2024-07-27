#![allow(dead_code)]
pub mod content;
pub mod scroll_bar;
pub mod scroll_box;

use crate::prelude::*;
use bevy::prelude::*;

/// Context data provided by a [`ScrollBox`](crate::ScrollBox) widget
#[derive(Component, Default, Reflect, Debug, Copy, Clone, PartialEq)]
pub struct ScrollContext {
    pub(super) scroll_x: f32,
    pub(super) scroll_y: f32,
    pub(super) content_width: f32,
    pub(super) content_height: f32,
    pub(super) scrollbox_width: f32,
    pub(super) scrollbox_height: f32,
    pub(super) pad_x: f32,
    pub(super) pad_y: f32,
    pub(super) mode: ScrollMode,
    pub(super) is_dragging: bool,
    pub(super) start_pos: Vec2,
    pub(super) start_offset: Vec2,
}

#[non_exhaustive]
#[derive(Default, Debug, Reflect, Copy, Clone, PartialEq, Eq)]
pub enum ScrollMode {
    /// Clamps the scroll offset to stay within the scroll range
    #[default]
    Clamped,
    /// Allows infinite scrolling
    Infinite,
}

impl ScrollContext {
    /// Get the current x-axis scroll offset
    pub fn scroll_x(&self) -> f32 {
        self.scroll_x
    }

    /// Get the current y-axis scroll offset
    pub fn scroll_y(&self) -> f32 {
        self.scroll_y
    }

    /// The width of the content
    pub fn content_width(&self) -> f32 {
        if self.content_width > self.scrollbox_width {
            self.content_width + self.pad_x
        } else {
            self.content_width
        }
    }

    /// The height of the content
    pub fn content_height(&self) -> f32 {
        if self.content_height > self.scrollbox_height {
            self.content_height + self.pad_y
        } else {
            self.content_height
        }
    }

    /// The total amount that can be scrolled along the x-axis
    pub fn scrollable_width(&self) -> f32 {
        (self.content_width() - self.scrollbox_width).max(0.0)
    }

    /// The total amount that can be scrolled along the y-axis
    pub fn scrollable_height(&self) -> f32 {
        (self.content_height() - self.scrollbox_height).max(0.0)
    }

    /// The current scroll mode
    pub fn mode(&self) -> ScrollMode {
        self.mode
    }

    /// Set the scroll offset along the x-axis
    ///
    /// This automatically accounts for the scroll mode
    pub fn set_scroll_x(&mut self, x: f32) {
        let min = -self.scrollable_width();
        self.scroll_x = match self.mode {
            ScrollMode::Clamped => ScrollContext::clamped(x, min, 0.0),
            ScrollMode::Infinite => x,
        }
    }

    /// Set the scroll offset along the y-axis
    ///
    /// This automatically accounts for the scroll mode
    pub fn set_scroll_y(&mut self, y: f32) {
        let min = -self.scrollable_height();
        self.scroll_y = match self.mode {
            ScrollMode::Clamped => ScrollContext::clamped(y, min, 0.0),
            ScrollMode::Infinite => y,
        };
    }

    /// The percent scrolled along the x-axis
    pub fn percent_x(&self) -> f32 {
        let width = self.scrollable_width();
        if width <= f32::EPSILON {
            // Can't divide by zero
            0.0
        } else {
            self.scroll_x / width
        }
    }

    /// The percent scrolled along the y-axis
    pub fn percent_y(&self) -> f32 {
        let height = self.scrollable_height();
        if height <= f32::EPSILON {
            // Can't divide by zero
            0.0
        } else {
            self.scroll_y / height
        }
    }

    /// Clamps a given value between a range
    fn clamped(value: f32, min: f32, max: f32) -> f32 {
        value.clamp(min, max)
    }
}

#[derive(Component, Widget, Reflect, Default, PartialEq, Clone)]
#[widget_systems(update, render)]
pub struct ScrollContextProvider {
    initial_value: ScrollContext,
}

#[derive(Bundle, Default, Clone)]
pub struct ScrollContextProviderBundle {
    pub provider: ScrollContextProvider,
    pub children: WidgetChildren,
    pub styles: WoodpeckerStyle,
}

pub fn update(
    current_widget: Res<CurrentWidget>,
    query: Query<(Ref<ScrollContextProvider>, Ref<WidgetChildren>)>,
) -> bool {
    let Ok((sp, children)) = query.get(**current_widget) else {
        return false;
    };

    sp.is_changed() || children.children_changed()
}

pub fn render(
    mut commands: Commands,
    mut context: ResMut<HookHelper>,
    current_widget: Res<CurrentWidget>,
    mut query: Query<(&mut WidgetChildren, &ScrollContextProvider)>,
) {
    let Ok((mut children, provider)) = query.get_mut(**current_widget) else {
        return;
    };

    // Setup scroll context.
    let entity = context.use_context::<ScrollContext>(&mut commands, *current_widget);
    commands.entity(entity).insert(provider.initial_value);

    children.apply(current_widget.as_parent());
}

/// Maps a value from one range to another range
fn map_range(value: f32, from_range: (f32, f32), to_range: (f32, f32)) -> f32 {
    let from_diff = from_range.1 - from_range.0;
    if from_diff <= f32::EPSILON {
        value
    } else {
        to_range.0 + (value - from_range.0) * (to_range.1 - to_range.0) / from_diff
    }
}
