use crate::prelude::*;
use bevy::prelude::*;

/// One action in a [`SpeedDial`]'s expanded list.
#[derive(Reflect, Clone, PartialEq, Default, Debug)]
pub struct SpeedDialAction {
    /// A [`crate::icons`] glyph shown on the action's own round button (rendered via
    /// [`IconFont`]).
    pub icon: String,
}

/// Fired when a [`SpeedDial`] action is clicked. The dial closes itself in response (it owns
/// its own open/closed state, unlike most of this crate's other interactive widgets) -- only
/// `index` needs to escape to the caller.
#[derive(Reflect, Clone, PartialEq, Default, Debug)]
pub struct SpeedDialActionClicked {
    /// The clicked action's index into [`SpeedDial::actions`].
    pub index: usize,
}

/// [`SpeedDial`]'s themed colors.
#[derive(Component, Reflect, Clone, Copy, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
pub struct SpeedDialStyles {
    /// The main FAB's background color.
    pub fab_background: Color,
    /// The main FAB's glyph color.
    pub fab_color: Color,
    /// Each action button's background color.
    pub action_background: Color,
    /// Each action button's glyph color.
    pub action_color: Color,
}

impl Default for SpeedDialStyles {
    fn default() -> Self {
        Self::from_theme(&Theme::default())
    }
}

impl ThemedStyle for SpeedDialStyles {
    fn from_theme(theme: &Theme) -> Self {
        Self {
            fab_background: theme.primary,
            fab_color: Color::WHITE,
            action_background: theme.background_light,
            action_color: theme.text,
        }
    }
}

#[derive(Component, Default, Reflect, Clone, Copy, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
struct SpeedDialState {
    open: bool,
}

/// A floating action button that expands into a column of related actions on click -- e.g. a
/// bottom-right "+" that expands into "New document" / "New folder" / "Upload". Composes
/// [`Popover`] directly (trigger = the main FAB, content = the action column) rather than a
/// parallel floating-content implementation. Owns its own open/closed state internally (most of
/// this crate's other interactive widgets are caller-controlled, but a speed dial closing
/// itself after an action click -- or when clicked again -- is the whole point of the pattern).
#[derive(Widget, Component, Reflect, Clone, PartialEq, Default)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(WoodpeckerStyle, WidgetChildren, SpeedDialStyles)]
pub struct SpeedDial {
    /// The closed-state FAB glyph (e.g. [`icons::PLUS`]) -- automatically swapped for
    /// [`icons::X`] while expanded.
    pub icon: String,
    /// The actions shown when expanded.
    pub actions: Vec<SpeedDialAction>,
    /// Which side the action column expands toward -- typically [`PopoverPlacement::Top`] for
    /// a bottom-corner-anchored dial.
    pub placement: PopoverPlacement,
}

fn fab_button(
    size: f32,
    background: Color,
    color: Color,
    glyph: &str,
    icon_font: &IconFont,
    shadow: Option<WidgetBoxShadow>,
) -> impl Bundle + Clone {
    (
        IconButton,
        IconButtonStyles {
            width: size.into(),
            height: size.into(),
            normal: WoodpeckerStyle {
                background_color: background,
                border_radius: Corner::all(size / 2.0),
                justify_content: Some(WidgetAlignContent::Center),
                align_items: Some(WidgetAlignItems::Center),
                box_shadow: shadow,
                ..Default::default()
            },
            hovered: WoodpeckerStyle {
                background_color: background.with_alpha(0.85),
                border_radius: Corner::all(size / 2.0),
                justify_content: Some(WidgetAlignContent::Center),
                align_items: Some(WidgetAlignItems::Center),
                box_shadow: shadow,
                ..Default::default()
            },
        },
        WidgetChildren::default().with_child::<Element>((
            Element,
            WoodpeckerStyle {
                font_size: size * 0.5,
                color,
                font: Some(icon_font.0.id()),
                ..Default::default()
            },
            WidgetRender::Text {
                content: glyph.into(),
            },
        )),
    )
}

fn render(
    mut commands: Commands,
    current_widget: Res<CurrentWidget>,
    mut hooks: ResMut<HookHelper>,
    icon_font: Res<IconFont>,
    theme: Res<Theme>,
    mut query: Query<(&SpeedDial, &SpeedDialStyles, &mut WidgetChildren)>,
    state_query: Query<&SpeedDialState>,
) {
    let Ok((speed_dial, dial_styles, mut children)) = query.get_mut(**current_widget) else {
        return;
    };

    let state_entity = hooks.use_state(&mut commands, *current_widget, SpeedDialState::default());
    let default_state = SpeedDialState::default();
    let state = state_query.get(state_entity).unwrap_or(&default_state);
    let current_widget = *current_widget;
    let open = state.open;

    let fab_glyph = if open {
        icons::X
    } else {
        speed_dial.icon.as_str()
    };

    let mut action_column = WidgetChildren::default();
    for (index, action) in speed_dial.actions.iter().enumerate() {
        action_column
            .add::<Element>((
                Element,
                WoodpeckerStyle {
                    margin: Edge::all(0.0).bottom(8.0),
                    ..Default::default()
                },
                WidgetChildren::default().with_child::<IconButton>(fab_button(
                    36.0,
                    dial_styles.action_background,
                    dial_styles.action_color,
                    &action.icon,
                    &icon_font,
                    None,
                )),
            ))
            .observe(
                current_widget,
                move |_trigger: On<Pointer<Click>>,
                      mut commands: Commands,
                      mut state_query: Query<&mut SpeedDialState>| {
                    if let Ok(mut state) = state_query.get_mut(state_entity) {
                        state.open = false;
                    }
                    commands.trigger(Change {
                        target: current_widget.entity(),
                        data: SpeedDialActionClicked { index },
                    });
                },
            );
        action_column.add_key(format!("action-{index}"));
    }

    *children = WidgetChildren::default();
    children
        .add::<Popover>((PopoverBundle {
            popover: Popover {
                visible: open,
                placement: speed_dial.placement,
            },
            // Not the bundle's own default (`PopoverStyles::default()`, always `Theme::default()`
            // regardless of the live theme this render already has access to) -- see the same
            // fix in `date_picker.rs` for the full reasoning.
            styles: PopoverStyles::from_theme(&theme),
            trigger: PassedChildren(WidgetChildren::default().with_child::<IconButton>(
                fab_button(
                    48.0,
                    dial_styles.fab_background,
                    dial_styles.fab_color,
                    fab_glyph,
                    &icon_font,
                    Some(theme.elevation.md),
                ),
            )),
            content: PopoverContent(WidgetChildren::default().with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    flex_direction: WidgetFlexDirection::Column,
                    align_items: Some(WidgetAlignItems::Center),
                    ..Default::default()
                },
                action_column,
            ))),
            ..Default::default()
        },))
        .observe(
            current_widget,
            move |_trigger: On<Pointer<Click>>, mut state_query: Query<&mut SpeedDialState>| {
                if let Ok(mut state) = state_query.get_mut(state_entity) {
                    state.open = !state.open;
                }
            },
        );
    children.add_key("popover");

    children.apply(current_widget.as_parent());
}
