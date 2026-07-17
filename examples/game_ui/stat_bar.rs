use bevy::prelude::*;
use woodpecker_ui::prelude::*;

/// A collection of styles for [`StatBar`] -- shape mirrors `woodpecker_ui`'s own
/// `ProgressBarStyles`.
#[derive(Component, Reflect, Clone, Copy, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
pub struct StatBarStyles {
    pub track_color: Color,
    pub fill_color: Color,
    pub height: f32,
}

impl Default for StatBarStyles {
    fn default() -> Self {
        Self {
            track_color: Color::srgb(0.1, 0.1, 0.1),
            fill_color: Color::WHITE,
            height: 18.0,
        }
    }
}

/// Per-widget state: the previously-seen `value` (so `render()` can tell an actual value
/// change apart from an unrelated re-render) plus the fill's own `Transition`, owned here
/// rather than required directly on the widget's own entity -- see the note on `StatBar` below.
#[derive(Component, Default, Clone, PartialEq, Reflect)]
#[reflect(Component, DiffableProp, PartialEq)]
pub struct StatBarState {
    previous_value: f32,
    fill_transition: Transition,
}

/// A `Transition`-animated value bar (health/mana/stamina/XP) -- unlike `woodpecker_ui`'s own
/// `ProgressBar` (which snaps the fill width instantly), this glides between values via two
/// full-style snapshots (`style_a`/`style_b`) handed to a `Transition` on value change.
///
/// The `Transition` is only ever attached to the *fill child* entity, never this widget's own
/// entity: `update_transitions` overwrites `*styles` wholesale every frame for any entity
/// carrying one, which would stomp the track's own background/height/border-radius.
#[derive(Widget, Component, Reflect, Clone, PartialEq, Default)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(
    WoodpeckerStyle = default_style(),
    WidgetChildren,
    WidgetRender = WidgetRender::Quad,
    StatBarStyles
)]
pub struct StatBar {
    /// The current value, pre-normalized by the caller to 0..1.
    pub value: f32,
    /// Optional centered label text (e.g. "482/600").
    pub label: Option<String>,
}

fn default_style() -> WoodpeckerStyle {
    WoodpeckerStyle {
        width: Units::Percentage(100.0),
        ..Default::default()
    }
}

fn fill_style(bar_styles: &StatBarStyles, value: f32) -> WoodpeckerStyle {
    WoodpeckerStyle {
        width: Units::Percentage(value.clamp(0.0, 1.0) * 100.0),
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
        &StatBar,
        &StatBarStyles,
        &mut WoodpeckerStyle,
        &mut WidgetChildren,
    )>,
    mut state_query: Query<&mut StatBarState>,
) {
    let Ok((stat_bar, bar_styles, mut styles, mut children)) = query.get_mut(**current_widget)
    else {
        return;
    };

    let value = stat_bar.value.clamp(0.0, 1.0);

    let state_entity = hooks.use_state(
        &mut commands,
        *current_widget,
        StatBarState {
            previous_value: value,
            fill_transition: Transition {
                playing: false,
                style_a: fill_style(bar_styles, value),
                style_b: fill_style(bar_styles, value),
                ..Default::default()
            },
        },
    );

    let Ok(mut state) = state_query.get_mut(state_entity) else {
        return;
    };

    if state.previous_value != value {
        let previous_value = state.previous_value;
        state.previous_value = value;
        state.fill_transition.easing = TransitionEasing::QuadraticOut;
        state.fill_transition.timeout = 400.0;
        state.fill_transition.style_a = fill_style(bar_styles, previous_value);
        state.fill_transition.style_b = fill_style(bar_styles, value);
        state.fill_transition.start();
    }

    styles.height = bar_styles.height.into();
    styles.background_color = bar_styles.track_color;
    styles.border_radius = Corner::all(bar_styles.height / 2.0);

    *children = WidgetChildren::default()
        .with_child::<Element>((Element, WidgetRender::Quad, state.fill_transition))
        .with_key("fill");

    if let Some(label) = stat_bar.label.as_ref() {
        children.add::<Element>((
            Element,
            WoodpeckerStyle {
                position: WidgetPosition::Absolute,
                width: Units::Percentage(100.0),
                height: Units::Percentage(100.0),
                justify_content: Some(WidgetAlignContent::Center),
                align_items: Some(WidgetAlignItems::Center),
                ..Default::default()
            },
            WidgetChildren::default().with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    font_size: (bar_styles.height * 0.6).clamp(9.0, 12.0),
                    color: Color::WHITE,
                    text_wrap: TextWrap::None,
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: label.clone(),
                },
            )),
        ));
        children.add_key("label");
    }

    children.apply(current_widget.as_parent());
}
