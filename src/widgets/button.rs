use crate::prelude::*;
use bevy::prelude::*;

/// A set of styles used to style a button.
#[derive(Component, Reflect, Clone, Copy, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
pub struct ButtonStyles {
    /// Normal styles(not hovered).
    pub normal: WoodpeckerStyle,
    /// Styles to apply when hovered.
    pub hovered: WoodpeckerStyle,
}

impl Default for ButtonStyles {
    fn default() -> Self {
        Self::from_theme(&Theme::default())
    }
}

impl ThemedStyle for ButtonStyles {
    fn from_theme(theme: &Theme) -> Self {
        // No `width`/`margin` here -- a button should hug its content by default.
        let normal = WoodpeckerStyle {
            background_color: theme.background_mid,
            border_color: theme.border,
            border: Edge::all(1.0),
            border_radius: Corner::all(theme.control_radius),
            padding: Edge::all(0.0).left(16.0).right(16.0),
            font_size: theme.font_size,
            height: theme.control_height.into(),
            // `WoodpeckerStyle::default()`'s own `color` is `Color::WHITE` -- correct against
            // `Theme::dark()`, illegible against `Theme::light()`'s pale button fills. This is
            // the button's *own* text color (e.g. if a caller reads `button_styles.normal.color`
            // for a custom label); it doesn't reach `WButton::text()`'s convenience label,
            // which is a `World`-free static bundle builder and so can't read this at all --
            // see that function's own doc comment for the same limitation there.
            color: theme.text,
            text_alignment: Some(TextAlign::Center),
            justify_content: Some(WidgetAlignContent::Center),
            align_items: Some(WidgetAlignItems::Center),
            ..Default::default()
        };
        Self {
            normal,
            hovered: WoodpeckerStyle {
                background_color: theme.background_light,
                border_color: theme.primary,
                ..normal
            },
        }
    }
}

#[derive(Component, Debug, Default, Reflect, PartialEq, Clone)]
#[reflect(Component, DiffableProp, PartialEq)]
pub struct WButtonState {
    pub hovering: bool,
}

/// The Woodpecker UI Button
#[derive(Component, Widget, Default, Reflect, PartialEq, Clone)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(WidgetRender = WidgetRender::Quad, WidgetChildren, WoodpeckerStyle = ButtonStyles::default().normal, Pickable, ButtonStyles)]
pub struct WButton;

impl WButton {
    /// A `WButton` with a single centered text label -- the most common case, collapsing the
    /// `(WButton, WidgetChildren::default().with_child::<Element>(...))` shape otherwise
    /// hand-assembled at every "just a labeled button" call site.
    ///
    /// The label defaults to `Color::WHITE` -- correct against `Theme::dark()`'s button
    /// background, but low-contrast against a light theme's. This helper builds a static
    /// bundle with no `World`/`Res` access, so it can't read the live `Theme` to pick a
    /// contrasting color itself; an app that swaps themes at runtime and uses `text()` should
    /// either override the label's `color` per call or watch `Theme` and rebuild it.
    pub fn text(label: impl Into<String>) -> impl Bundle + Clone {
        (
            WButton,
            WidgetChildren::default().with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    text_wrap: TextWrap::None,
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: label.into(),
                },
            )),
        )
    }
}

pub fn render(
    current_widget: Res<CurrentWidget>,
    mut commands: Commands,
    mut hooks: ResMut<HookHelper>,
    mut query: Query<(&mut WoodpeckerStyle, &ButtonStyles, &mut WidgetChildren)>,
    state_query: Query<&WButtonState>,
) {
    let Ok((mut styles, button_styles, mut children)) = query.get_mut(**current_widget) else {
        return;
    };

    let state_entity = hooks.use_state(&mut commands, *current_widget, WButtonState::default());
    let Ok(state) = state_query.get(state_entity) else {
        return;
    };

    if state.hovering {
        *styles = button_styles.hovered;
    } else {
        *styles = button_styles.normal;
    }

    children.self_hover_state(
        *current_widget,
        state_entity,
        SystemCursorIcon::Pointer,
        |state: &mut WButtonState, hovering| state.hovering = hovering,
    );

    children.apply(current_widget.as_parent());
}
