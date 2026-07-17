use crate::prelude::*;
use bevy::prelude::*;

/// Fired when a [`Link`] is clicked.
#[derive(Reflect, Clone, PartialEq, Default, Debug)]
pub struct LinkClicked;

/// [`Link`]'s themed colors.
#[derive(Component, Reflect, Clone, Copy, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
pub struct LinkStyles {
    /// Text (and underline) color at rest.
    pub color: Color,
    /// Text (and underline) color while hovered.
    pub hover_color: Color,
}

impl Default for LinkStyles {
    fn default() -> Self {
        Self::from_theme(&Theme::default())
    }
}

impl ThemedStyle for LinkStyles {
    fn from_theme(theme: &Theme) -> Self {
        Self {
            color: theme.primary,
            hover_color: theme.text,
        }
    }
}

#[derive(Component, Default, Reflect, Clone, Copy, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
struct LinkState {
    hovering: bool,
}

/// An inline, clickable text link -- e.g. "Forgot password?", "View all", a breadcrumb
/// segment (see [`crate::widgets::Breadcrumbs`], which composes this directly). Underlines on
/// hover via a bottom border rather than a real text-decoration style (`WoodpeckerStyle` has no
/// such field) and swaps to `LinkStyles::hover_color`. Fires [`Change<LinkClicked>`] on click;
/// this crate has no navigation/routing concept, so a caller's own observer decides what
/// "following" the link actually does.
#[derive(Widget, Component, Reflect, Clone, PartialEq, Default)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(WoodpeckerStyle, WidgetChildren, Pickable, WidgetRender = WidgetRender::Quad, LinkStyles)]
pub struct Link {
    /// The link's text.
    pub label: String,
}

fn render(
    mut commands: Commands,
    current_widget: Res<CurrentWidget>,
    mut hooks: ResMut<HookHelper>,
    mut query: Query<(
        &Link,
        &LinkStyles,
        &mut WoodpeckerStyle,
        &mut WidgetChildren,
    )>,
    state_query: Query<&LinkState>,
) {
    let Ok((link, link_styles, mut styles, mut children)) = query.get_mut(**current_widget) else {
        return;
    };

    let state_entity = hooks.use_state(&mut commands, *current_widget, LinkState::default());
    let default_state = LinkState::default();
    let state = state_query.get(state_entity).unwrap_or(&default_state);
    let current_widget = *current_widget;

    let color = if state.hovering {
        link_styles.hover_color
    } else {
        link_styles.color
    };

    // Always reserve the 1px bottom border (transparent when not hovering) rather than only
    // adding it on hover -- toggling border width itself changes the box's total size, shifting
    // whatever sits below it by that 1px every hover in/out.
    styles.border = Edge::all(0.0).bottom(1.0);
    styles.border_color = if state.hovering { color } else { Color::NONE };

    *children = WidgetChildren::default().with_child::<Element>((
        Element,
        WoodpeckerStyle {
            font_size: 13.0,
            color,
            text_wrap: TextWrap::None,
            ..Default::default()
        },
        WidgetRender::Text {
            content: link.label.clone(),
        },
    ));
    children.add_key("label");

    children
        .self_observe(
            current_widget,
            move |_trigger: On<Pointer<Click>>, mut commands: Commands| {
                commands.trigger(Change {
                    target: current_widget.entity(),
                    data: LinkClicked,
                });
            },
        )
        .self_hover_state(
            current_widget,
            state_entity,
            SystemCursorIcon::Pointer,
            |state: &mut LinkState, hovering| state.hovering = hovering,
        );

    children.apply(current_widget.as_parent());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_theme_tracks_the_passed_in_theme() {
        let theme = Theme::dark();
        let styles = LinkStyles::from_theme(&theme);
        assert_eq!(styles.color, theme.primary);
        assert_eq!(styles.hover_color, theme.text);

        let dark = LinkStyles::from_theme(&Theme::dark());
        let light = LinkStyles::from_theme(&Theme::light());
        assert_ne!(
            dark.hover_color, light.hover_color,
            "from_theme must track the passed-in theme, not a fixed default"
        );
    }
}
