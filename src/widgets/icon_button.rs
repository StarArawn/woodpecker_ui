use crate::prelude::*;
use bevy::prelude::*;

/// A collection of styles for icon buttons.
#[derive(Component, Clone, Copy, PartialEq, Reflect)]
#[reflect(Component, DiffableProp, PartialEq)]
pub struct IconButtonStyles {
    /// Normal Styles
    pub normal: WoodpeckerStyle,
    /// Hovered Styles
    pub hovered: WoodpeckerStyle,
    /// Width of the icon
    pub width: Units,
    /// Height of the icon
    pub height: Units,
}

impl Default for IconButtonStyles {
    fn default() -> Self {
        Self::from_theme(&Theme::default())
    }
}

impl ThemedStyle for IconButtonStyles {
    fn from_theme(theme: &Theme) -> Self {
        let normal = WoodpeckerStyle {
            background_color: theme.background_mid,
            border_radius: Corner::all(theme.control_radius),
            // Without these, `justify_content`/`align_items` default to `None` (flex-start /
            // stretch) -- a bare glyph child then sits top-left instead of centered. Easy to
            // miss with small ASCII glyphs (plenty of whitespace hides it), impossible to miss
            // with a Phosphor icon that fills nearly its whole line-height box (see
            // `crate::icons`'s doc comment on the font's metrics).
            justify_content: Some(WidgetAlignContent::Center),
            align_items: Some(WidgetAlignItems::Center),
            ..Default::default()
        };
        Self {
            normal,
            hovered: WoodpeckerStyle {
                background_color: theme.background_light,
                ..normal
            },
            width: 32.0.into(),
            height: 32.0.into(),
        }
    }
}

#[derive(Component, Debug, Default, PartialEq, Clone, Reflect)]
#[reflect(Component, DiffableProp, PartialEq)]
pub struct IconButtonState {
    pub hovering: bool,
}

/// A generic button widget used for easy buttons!
#[derive(Component, Widget, Default, Reflect, PartialEq, Clone)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(WidgetRender = WidgetRender::Quad, WidgetChildren, WoodpeckerStyle = IconButtonStyles::default().normal, IconButtonStyles, Pickable)]
pub struct IconButton;

pub fn render(
    current_widget: Res<CurrentWidget>,
    mut commands: Commands,
    mut hooks: ResMut<HookHelper>,
    mut query: Query<(
        &mut WoodpeckerStyle,
        &IconButtonStyles,
        &mut WidgetChildren,
        &mut WidgetRender,
    )>,
    state_query: Query<&IconButtonState>,
) {
    let Ok((mut styles, button_styles, mut children, mut render)) = query.get_mut(**current_widget)
    else {
        return;
    };

    let state_entity = hooks.use_state(&mut commands, *current_widget, IconButtonState::default());

    let default_state = IconButtonState::default();
    let state = state_query.get(state_entity).unwrap_or(&default_state);

    if state.hovering {
        render.set_color(button_styles.hovered.background_color);
        *styles = WoodpeckerStyle {
            width: button_styles.width,
            height: button_styles.height,
            ..button_styles.hovered
        };
    } else {
        render.set_color(button_styles.normal.background_color);
        *styles = WoodpeckerStyle {
            width: button_styles.width,
            height: button_styles.height,
            ..button_styles.normal
        };
    }

    children.self_hover_state(
        *current_widget,
        state_entity,
        SystemCursorIcon::Pointer,
        |state: &mut IconButtonState, hovering| state.hovering = hovering,
    );

    children.apply(current_widget.as_parent());
}
