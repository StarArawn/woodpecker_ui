use bevy::prelude::*;
use woodpecker_ui::prelude::*;

use crate::mock_data::GuildRoster as GuildRosterData;
use crate::theme;

/// Own `hooks.use_state`-owned sort state -- `GuildRosterData` is static seed data; only
/// *how it's displayed* is reactive here.
#[derive(Component, Default, Clone, PartialEq, Reflect)]
#[reflect(Component, DiffableProp, PartialEq)]
struct GuildSortState {
    column: Option<usize>,
    ascending: bool,
}

/// Watches `GuildRosterData` and renders it with a real, working `Table` + `TableSort` --
/// `examples/dashboard/overview.rs`'s own `Table` usage is static and never exercises sorting.
#[derive(Widget, Component, Reflect, Clone, PartialEq, Default)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(guild_roster_render)]
#[require(WidgetChildren, WoodpeckerStyle = default_style(), WatchedResource<GuildRosterData>)]
pub struct GuildRoster;

fn default_style() -> WoodpeckerStyle {
    WoodpeckerStyle {
        width: 480.0.into(),
        // Fixed height (content scrolls inside it) -- a full roster's rows can run well
        // past a reasonable window size otherwise, same reasoning as `CharacterSheet`.
        height: 360.0.into(),
        ..Default::default()
    }
}

fn guild_roster_render(
    mut commands: Commands,
    mut hooks: ResMut<HookHelper>,
    current_widget: Res<CurrentWidget>,
    mut query: Query<(&WatchedResource<GuildRosterData>, &mut WidgetChildren)>,
    state_query: Query<&GuildSortState>,
) {
    let Ok((roster, mut children)) = query.get_mut(**current_widget) else {
        return;
    };

    let state_entity = hooks.use_state(&mut commands, *current_widget, GuildSortState::default());
    let current_widget = *current_widget;
    let default_state = GuildSortState::default();
    let state = state_query.get(state_entity).unwrap_or(&default_state);

    let mut members = roster.0.clone();
    if let Some(col) = state.column {
        members.sort_by(|a, b| {
            let ord = match col {
                0 => a.rank.cmp(&b.rank),
                1 => a.name.cmp(&b.name),
                2 => a.level.cmp(&b.level),
                3 => a.class.cmp(&b.class),
                _ => a.score.cmp(&b.score),
            };
            if state.ascending {
                ord
            } else {
                ord.reverse()
            }
        });
    }

    let rows = members
        .iter()
        .map(|m| {
            TableRow::from(vec![
                m.rank.clone(),
                m.name.clone(),
                m.level.to_string(),
                m.class.clone(),
                m.score.to_string(),
            ])
        })
        .collect();

    let content = WidgetChildren::default()
        .with_child::<Table>((
            Table {
                columns: vec![
                    TableColumn::fixed("Rank", 80.0),
                    TableColumn::flexible("Name"),
                    TableColumn::fixed("Level", 60.0),
                    TableColumn::fixed("Class", 90.0),
                    TableColumn::fixed("Score", 80.0),
                ],
                rows,
                ..Default::default()
            },
            TableStyles {
                header_background: theme::BG_INPUT,
                zebra_background: theme::BG_WINDOW_HOVER,
                border_color: theme::BORDER_DARK,
                corner_radius: theme::RADIUS_SM,
                ..Default::default()
            },
        ))
        .with_key("table")
        .with_observe(
            current_widget,
            move |trigger: On<Change<TableSort>>, mut state_query: Query<&mut GuildSortState>| {
                let Ok(mut state) = state_query.get_mut(state_entity) else {
                    return;
                };
                if state.column == Some(trigger.data.column_index) {
                    state.ascending = !state.ascending;
                } else {
                    state.column = Some(trigger.data.column_index);
                    state.ascending = true;
                }
            },
        );

    *children = theme::scrollable_panel(content);

    children.apply(current_widget.as_parent());
}
