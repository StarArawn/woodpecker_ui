use super::TabContext;
use crate::prelude::*;
use bevy::prelude::*;

/// [`TabButton`]'s themed inactive/active styles -- a separate sibling component (rather than
/// the previous hardcoded `Default` impl) so it live-resyncs on a [`Theme`] swap, matching
/// [`ButtonStyles`]' own pattern. A caller wanting custom colors provides its own
/// `TabButtonStyles` (plus [`ThemeOverride`] to opt out of future resyncs) instead of setting
/// fields directly on `TabButton`.
#[derive(Component, Reflect, Clone, Copy, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
pub struct TabButtonStyles {
    /// Styles for an unselected tab.
    pub inactive: ButtonStyles,
    /// Styles for the selected tab.
    pub active: ButtonStyles,
    /// Color of the active tab's underline.
    pub underline_color: Color,
}

impl Default for TabButtonStyles {
    fn default() -> Self {
        Self::from_theme(&Theme::default())
    }
}

impl ThemedStyle for TabButtonStyles {
    fn from_theme(theme: &Theme) -> Self {
        let base_styles = WoodpeckerStyle {
            align_items: Some(WidgetAlignItems::Center),
            height: 40.0.into(),
            font_size: theme.font_size,
            color: theme.text,
            padding: Edge::all(0.0).left(16.0).right(16.0),
            border: Edge::all(0.0).bottom(2.0),
            border_color: Color::NONE,
            ..Default::default()
        };
        Self {
            inactive: ButtonStyles {
                normal: WoodpeckerStyle {
                    background_color: theme.background_light,
                    ..base_styles
                },
                hovered: WoodpeckerStyle {
                    background_color: theme.background_mid,
                    ..base_styles
                },
            },
            // The active tab merges into the content panel below it (same background) and
            // is marked instead by an accent-colored underline, rather than only relying on
            // the color blend to signal selection.
            active: ButtonStyles {
                normal: WoodpeckerStyle {
                    background_color: theme.background,
                    ..base_styles
                },
                hovered: WoodpeckerStyle {
                    background_color: theme.background,
                    ..base_styles
                },
            },
            underline_color: theme.primary,
        }
    }
}

/// Per-widget state
#[derive(Component, Debug, Reflect, Clone, Copy, PartialEq, Default)]
#[reflect(Component, DiffableProp, PartialEq)]
struct TabButtonState {
    /// Previous state (Used for animations)
    previous_active: bool,
    /// The underline transition
    underline_transition: Transition,
}

fn get_transition() -> Transition {
    Transition {
        easing: TransitionEasing::QuadraticInOut,
        timeout: 150.0,
        playing: false,
        ..Default::default()
    }
}

/// Tab button
#[derive(Widget, Component, Clone, PartialEq, Reflect, Default)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(TabButtonStyles)]
pub struct TabButton {
    /// The index(it should match the index of the content)
    pub index: usize,
    /// The title of the tab.
    pub title: String,
}

/// A tab button bundle
#[derive(Bundle, Default, Clone)]
pub struct TabButtonBundle {
    /// Tab button
    pub tab_button: TabButton,
    /// internal styles
    pub internal_styles: WoodpeckerStyle,
    /// internal children
    pub internal_children: WidgetChildren,
}

fn render(
    mut commands: Commands,
    current_widget: Res<CurrentWidget>,
    mut hooks: ResMut<HookHelper>,
    mut query: Query<(&TabButton, &TabButtonStyles, &mut WidgetChildren)>,
    context_query: Query<&TabContext>,
    mut state_query: Query<&mut TabButtonState>,
) {
    let Ok((tab_button, tab_button_styles, mut children)) = query.get_mut(**current_widget) else {
        return;
    };

    let context_entity = hooks.use_context(&mut commands, *current_widget, TabContext::default());

    let Ok(context) = context_query.get(context_entity) else {
        return;
    };

    let is_active = context.current_index == tab_button.index;
    let button_styles = if is_active {
        tab_button_styles.active
    } else {
        tab_button_styles.inactive
    };

    let state_entity = hooks.use_state(
        &mut commands,
        *current_widget,
        TabButtonState {
            previous_active: is_active,
            underline_transition: get_transition(),
        },
    );
    let Ok(mut state) = state_query.get_mut(state_entity) else {
        return;
    };

    if !state.underline_transition.is_playing() {
        state.underline_transition = Transition {
            playing: false,
            ..get_transition()
        };
    }
    let old_active = state.previous_active;
    if old_active != is_active {
        state.underline_transition.start();
        state.previous_active = is_active;
    }

    let underline_visible = WoodpeckerStyle {
        position: WidgetPosition::Absolute,
        left: 0.0.into(),
        right: 0.0.into(),
        bottom: 0.0.into(),
        height: 2.0.into(),
        background_color: tab_button_styles.underline_color,
        opacity: 1.0,
        ..Default::default()
    };
    let underline_hidden = WoodpeckerStyle {
        opacity: 0.0,
        ..underline_visible
    };
    let underline_transition = Transition {
        style_a: if old_active {
            underline_visible
        } else {
            underline_hidden
        },
        style_b: if is_active {
            underline_visible
        } else {
            underline_hidden
        },
        ..state.underline_transition
    };

    // Actual button.
    let index = tab_button.index;
    children
        .add::<WButton>((
            WButton,
            button_styles,
            WidgetChildren::default()
                .with_child::<Element>((
                    Element,
                    WoodpeckerStyle {
                        font_size: button_styles.normal.font_size,
                        color: button_styles.normal.color,
                        text_wrap: TextWrap::None,
                        ..Default::default()
                    },
                    WidgetRender::Text {
                        content: tab_button.title.clone(),
                    },
                ))
                .with_key("title")
                .with_child::<Element>((
                    Element,
                    if is_active {
                        underline_visible
                    } else {
                        underline_hidden
                    },
                    WidgetRender::Quad,
                    underline_transition,
                ))
                .with_key("underline"),
        ))
        .observe(
            *current_widget,
            move |_trigger: On<Pointer<Click>>, mut context_query: Query<&mut TabContext>| {
                let Ok(mut context) = context_query.get_mut(context_entity) else {
                    return;
                };
                context.current_index = index;
            },
        );

    children.apply(current_widget.as_parent());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_and_inactive_differ_and_track_the_passed_in_theme() {
        let dark = TabButtonStyles::from_theme(&Theme::dark());
        assert_ne!(
            dark.active.normal.background_color, dark.inactive.normal.background_color,
            "active and inactive must render distinguishably"
        );
        assert_eq!(dark.underline_color, Theme::dark().primary);

        let light = TabButtonStyles::from_theme(&Theme::light());
        assert_ne!(
            dark.inactive.normal.background_color, light.inactive.normal.background_color,
            "from_theme must track the passed-in theme, not a fixed default"
        );
    }
}
