use crate::prelude::*;
use bevy::prelude::*;

/// Fired when [`TransferList`] moves its checked items across -- the caller owns `left`/`right`
/// (same controlled-component model [`crate::widgets::Chip`]'s `selected` uses) and is expected
/// to re-declare `TransferList` with these new lists in response.
#[derive(Reflect, Clone, PartialEq, Default, Debug)]
pub struct TransferListChanged {
    /// The new left-side list.
    pub left: Vec<String>,
    /// The new right-side list.
    pub right: Vec<String>,
}

/// [`TransferList`]'s themed colors -- reuses [`ListItemStyles`] for each panel's rows
/// directly rather than a parallel color set.
#[derive(Component, Reflect, Clone, Copy, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
pub struct TransferListStyles {
    /// Each panel's border color.
    pub panel_border: Color,
    /// Panel title text color.
    pub title_color: Color,
    /// Checked-glyph color.
    pub checked_color: Color,
    /// Unchecked-glyph color.
    pub unchecked_color: Color,
}

impl Default for TransferListStyles {
    fn default() -> Self {
        Self::from_theme(&Theme::default())
    }
}

impl ThemedStyle for TransferListStyles {
    fn from_theme(theme: &Theme) -> Self {
        Self {
            panel_border: theme.border,
            title_color: theme.text,
            checked_color: theme.primary,
            unchecked_color: theme.text.with_alpha(0.4),
        }
    }
}

/// Which side's checked-item selection [`TransferListState`] tracks -- entries are moved out
/// (and their checked mark cleared) whenever the caller re-declares `left`/`right` without
/// them, so this never needs to be reset explicitly.
#[derive(Component, Default, Reflect, Clone, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
struct TransferListState {
    left_checked: Vec<String>,
    right_checked: Vec<String>,
}

/// Two side-by-side lists with checkable rows and move-selected controls between them -- e.g.
/// "Available roles" / "Assigned roles". Rows are plain [`ListItem`]s with a checkbox-glyph
/// [`ListItemLeading`] (not the real [`crate::widgets::Checkbox`] widget: that owns its checked
/// state internally with no externally-settable initial value, which can't be reset when an
/// item moves across and needs to un-check -- see its own doc comment) rather than a parallel
/// list implementation. Fires [`Change<TransferListChanged>`] on a move; the caller owns
/// `left`/`right`.
#[derive(Widget, Component, Reflect, Clone, PartialEq, Default)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(
    WoodpeckerStyle = WoodpeckerStyle { flex_direction: WidgetFlexDirection::Row, align_items: Some(WidgetAlignItems::Center), ..Default::default() },
    WidgetChildren,
    TransferListStyles
)]
pub struct TransferList {
    /// The left panel's items.
    pub left: Vec<String>,
    /// The right panel's items.
    pub right: Vec<String>,
    /// The left panel's title.
    pub left_title: String,
    /// The right panel's title.
    pub right_title: String,
}

fn checkbox_glyph(
    checked: bool,
    checked_color: Color,
    unchecked_color: Color,
    icon_font: &IconFont,
) -> WidgetChildren {
    WidgetChildren::default().with_child::<Element>((
        Element,
        WoodpeckerStyle {
            font_size: 14.0,
            color: if checked {
                checked_color
            } else {
                unchecked_color
            },
            font: Some(icon_font.0.id()),
            ..Default::default()
        },
        WidgetRender::Text {
            content: if checked {
                icons::CHECK_SQUARE
            } else {
                icons::SQUARE
            }
            .into(),
        },
    ))
}

/// Builds one side's panel: title, `List` of checkable rows, and the checked-items toggling
/// logic. Pulled out since both sides are structurally identical (just mirrored data/state).
#[allow(clippy::too_many_arguments)]
fn panel(
    title: &str,
    items: &[String],
    checked: &[String],
    checked_color: Color,
    unchecked_color: Color,
    panel_border: Color,
    title_color: Color,
    current_widget: CurrentWidget,
    state_entity: Entity,
    is_left: bool,
    icon_font: &IconFont,
) -> impl Bundle + Clone {
    let mut list_children = WidgetChildren::default();
    for item in items {
        let is_checked = checked.iter().any(|c| c == item);
        let item_value = item.clone();
        list_children
            .add::<ListItem>((
                ListItem {
                    primary_text: item.clone(),
                    dense: true,
                    ..Default::default()
                },
                ListItemLeading(checkbox_glyph(
                    is_checked,
                    checked_color,
                    unchecked_color,
                    icon_font,
                )),
            ))
            .observe(
                current_widget,
                move |_trigger: On<Change<ListItemClicked>>,
                      mut state_query: Query<&mut TransferListState>| {
                    let Ok(mut state) = state_query.get_mut(state_entity) else {
                        return;
                    };
                    let checked = if is_left {
                        &mut state.left_checked
                    } else {
                        &mut state.right_checked
                    };
                    if let Some(pos) = checked.iter().position(|c| c == &item_value) {
                        checked.remove(pos);
                    } else {
                        checked.push(item_value.clone());
                    }
                },
            );
        list_children.add_key(item.clone());
    }

    (
        Element,
        WoodpeckerStyle {
            flex_grow: 1.0,
            flex_direction: WidgetFlexDirection::Column,
            border: Edge::all(1.0),
            border_color: panel_border,
            border_radius: Corner::all(6.0),
            padding: Edge::all(8.0),
            height: 220.0.into(),
            ..Default::default()
        },
        WidgetRender::Quad,
        WidgetChildren::default()
            .with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    font_size: 13.0,
                    color: title_color,
                    margin: Edge::all(0.0).bottom(6.0),
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: format!("{title} ({})", items.len()),
                },
            ))
            .with_key("title")
            .with_child::<List>((List, PassedChildren(list_children)))
            .with_key("list"),
    )
}

fn render(
    mut commands: Commands,
    current_widget: Res<CurrentWidget>,
    mut hooks: ResMut<HookHelper>,
    icon_font: Res<IconFont>,
    mut query: Query<(&TransferList, &TransferListStyles, &mut WidgetChildren)>,
    state_query: Query<&TransferListState>,
) {
    let Ok((transfer_list, styles, mut children)) = query.get_mut(**current_widget) else {
        return;
    };

    let state_entity =
        hooks.use_state(&mut commands, *current_widget, TransferListState::default());
    let default_state = TransferListState::default();
    let state = state_query.get(state_entity).unwrap_or(&default_state);
    let current_widget = *current_widget;

    let left = transfer_list.left.clone();
    let right = transfer_list.right.clone();
    let left_checked = state.left_checked.clone();
    let right_checked = state.right_checked.clone();

    *children = WidgetChildren::default();
    children.add::<Element>(panel(
        &transfer_list.left_title,
        &left,
        &left_checked,
        styles.checked_color,
        styles.unchecked_color,
        styles.panel_border,
        styles.title_color,
        current_widget,
        state_entity,
        true,
        &icon_font,
    ));
    children.add_key("left");

    let list_entity = current_widget.entity();

    children.add::<Element>((
        Element,
        WoodpeckerStyle {
            flex_direction: WidgetFlexDirection::Column,
            align_items: Some(WidgetAlignItems::Center),
            padding: Edge::all(0.0).left(12.0).right(12.0),
            gap: (0.0.into(), 8.0.into()),
            ..Default::default()
        },
        WidgetChildren::default()
            .with_child::<IconButton>((
                IconButton,
                WidgetChildren::default().with_child::<Element>((
                    Element,
                    WoodpeckerStyle {
                        font_size: 14.0,
                        font: Some(icon_font.0.id()),
                        ..Default::default()
                    },
                    WidgetRender::Text {
                        content: icons::CARET_RIGHT.into(),
                    },
                )),
            ))
            .with_observe(
                current_widget,
                // Reads `TransferList`/`TransferListState` fresh from the ECS at click time,
                // rather than capturing a snapshot in the closure: this entity is *reused*
                // (not respawned) across re-renders, and the `ObserverCache` skips re-adding
                // an observer for a slot/target it's already seen -- so a closure that
                // captured `left`/`checked` at declaration time would keep running with
                // whatever was true the very first time this button was declared, forever.
                move |_trigger: On<Pointer<Click>>,
                      mut commands: Commands,
                      list_query: Query<&TransferList>,
                      mut state_query: Query<&mut TransferListState>| {
                    let Ok(list) = list_query.get(list_entity) else {
                        return;
                    };
                    let Ok(mut state) = state_query.get_mut(state_entity) else {
                        return;
                    };
                    if state.left_checked.is_empty() {
                        return;
                    }
                    let new_left: Vec<String> = list
                        .left
                        .iter()
                        .filter(|item| !state.left_checked.contains(item))
                        .cloned()
                        .collect();
                    let mut new_right = list.right.clone();
                    new_right.extend(state.left_checked.iter().cloned());
                    state.left_checked.clear();
                    commands.trigger(Change {
                        target: list_entity,
                        data: TransferListChanged {
                            left: new_left,
                            right: new_right,
                        },
                    });
                },
            )
            .with_key("move-right")
            .with_child::<IconButton>((
                IconButton,
                WidgetChildren::default().with_child::<Element>((
                    Element,
                    WoodpeckerStyle {
                        font_size: 14.0,
                        font: Some(icon_font.0.id()),
                        ..Default::default()
                    },
                    WidgetRender::Text {
                        content: icons::CARET_LEFT.into(),
                    },
                )),
            ))
            .with_observe(
                current_widget,
                // See the "move-right" observer's comment -- same fresh-read-at-click-time
                // reasoning applies here.
                move |_trigger: On<Pointer<Click>>,
                      mut commands: Commands,
                      list_query: Query<&TransferList>,
                      mut state_query: Query<&mut TransferListState>| {
                    let Ok(list) = list_query.get(list_entity) else {
                        return;
                    };
                    let Ok(mut state) = state_query.get_mut(state_entity) else {
                        return;
                    };
                    if state.right_checked.is_empty() {
                        return;
                    }
                    let new_right: Vec<String> = list
                        .right
                        .iter()
                        .filter(|item| !state.right_checked.contains(item))
                        .cloned()
                        .collect();
                    let mut new_left = list.left.clone();
                    new_left.extend(state.right_checked.iter().cloned());
                    state.right_checked.clear();
                    commands.trigger(Change {
                        target: list_entity,
                        data: TransferListChanged {
                            left: new_left,
                            right: new_right,
                        },
                    });
                },
            )
            .with_key("move-left"),
    ));
    children.add_key("controls");

    children.add::<Element>(panel(
        &transfer_list.right_title,
        &right,
        &right_checked,
        styles.checked_color,
        styles.unchecked_color,
        styles.panel_border,
        styles.title_color,
        current_widget,
        state_entity,
        false,
        &icon_font,
    ));
    children.add_key("right");

    children.apply(current_widget.as_parent());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_theme_tracks_the_passed_in_theme() {
        let theme = Theme::dark();
        let styles = TransferListStyles::from_theme(&theme);
        assert_eq!(styles.checked_color, theme.primary);
        assert_eq!(styles.panel_border, theme.border);

        let dark = TransferListStyles::from_theme(&Theme::dark());
        let light = TransferListStyles::from_theme(&Theme::light());
        assert_ne!(
            dark.panel_border, light.panel_border,
            "from_theme must track the passed-in theme, not a fixed default"
        );
    }
}
