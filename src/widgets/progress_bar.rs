use crate::prelude::*;
use bevy::prelude::*;

/// A collection of styles for [`ProgressBar`].
#[derive(Component, Reflect, Clone, Copy, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
pub struct ProgressBarStyles {
    /// The track (background) color.
    pub track_color: Color,
    /// The fill (progress) color.
    pub fill_color: Color,
    /// The bar's height, in logical pixels.
    pub height: f32,
}

impl Default for ProgressBarStyles {
    fn default() -> Self {
        Self::from_theme(&Theme::default())
    }
}

impl ThemedStyle for ProgressBarStyles {
    fn from_theme(theme: &Theme) -> Self {
        Self {
            track_color: theme.background_light,
            fill_color: theme.primary,
            height: 8.0,
        }
    }
}

/// Per-widget state -- notices `ProgressBar::value` changing so `fill_transition` can
/// (re)start from wherever the fill last settled. A plain value, not a separate `Transition`
/// component on `ProgressBar` itself, since that would let `update_transitions` overwrite the
/// track's own style every frame.
#[derive(Component, Debug, Reflect, Clone, PartialEq, Default)]
#[reflect(Component, DiffableProp, PartialEq)]
struct ProgressBarState {
    previous_value: f32,
    fill_transition: Transition,
}

/// A determinate progress bar. Purely presentational (no dragging/interactivity) -- for a
/// draggable value picker see [`crate::widgets::Slider`].
#[derive(Widget, Component, Reflect, Clone, PartialEq, Default)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(
    WoodpeckerStyle = default_style(),
    WidgetChildren,
    WidgetRender = WidgetRender::Quad,
    ProgressBarStyles
)]
pub struct ProgressBar {
    /// The current progress, clamped to 0..1.
    pub value: f32,
}

fn default_style() -> WoodpeckerStyle {
    WoodpeckerStyle {
        width: Units::Percentage(100.0),
        ..Default::default()
    }
}

fn fill_style(bar_styles: &ProgressBarStyles, value: f32) -> WoodpeckerStyle {
    WoodpeckerStyle {
        width: Units::Percentage(value * 100.0),
        height: Units::Percentage(100.0),
        background_color: bar_styles.fill_color,
        border_radius: Corner::all(bar_styles.height / 2.0),
        ..Default::default()
    }
}

fn render(
    mut commands: Commands,
    mut hooks: ResMut<HookHelper>,
    current_widget: Res<CurrentWidget>,
    mut query: Query<(
        &ProgressBar,
        &ProgressBarStyles,
        &mut WoodpeckerStyle,
        &mut WidgetChildren,
    )>,
    mut state_query: Query<&mut ProgressBarState>,
) {
    let Ok((bar, bar_styles, mut styles, mut children)) = query.get_mut(**current_widget) else {
        return;
    };

    let value = bar.value.clamp(0.0, 1.0);

    let state_entity = hooks.use_state(
        &mut commands,
        *current_widget,
        ProgressBarState {
            previous_value: value,
            fill_transition: Transition::default(),
        },
    );
    let Ok(mut state) = state_query.get_mut(state_entity) else {
        return;
    };

    styles.height = bar_styles.height.into();
    styles.background_color = bar_styles.track_color;
    styles.border_radius = Corner::all(bar_styles.height / 2.0);

    state.fill_transition.easing = TransitionEasing::QuadraticOut;
    state.fill_transition.timeout = 300.0;
    state.fill_transition.style_a = fill_style(bar_styles, state.previous_value);
    state.fill_transition.style_b = fill_style(bar_styles, value);
    if state.previous_value != value {
        state.fill_transition.start();
        state.previous_value = value;
    }

    *children = WidgetChildren::default().with_child::<Element>((
        Element,
        fill_style(bar_styles, value),
        WidgetRender::Quad,
        state.fill_transition,
    ));
    children.add_key("fill");

    children.apply(current_widget.as_parent());
}
