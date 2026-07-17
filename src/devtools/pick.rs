use bevy::{prelude::*, window::PrimaryWindow};

use crate::{
    hook_helper::StateMarker,
    layout::system::WidgetLayout,
    picking_backend::{walk_hit_candidates, PointerWorldPosition},
    prelude::PreviousWidget,
    styles::WoodpeckerStyle,
    WoodpeckerContext,
};

use super::{panel::DevtoolsRoot, DevtoolsHover, DevtoolsState};

pub(crate) fn click_to_select_system(
    mouse: Res<ButtonInput<MouseButton>>,
    primary_window: Single<&Window, With<PrimaryWindow>>,
    pointer_world: PointerWorldPosition,
    context: Res<WoodpeckerContext>,
    mut state: ResMut<DevtoolsState>,
    mut hover: ResMut<DevtoolsHover>,
    devtools_root_query: Query<Entity, With<DevtoolsRoot>>,
    layout_query: Query<(&WidgetLayout, &WoodpeckerStyle)>,
    children_query: Query<&Children>,
    state_marker_query: Query<(), With<StateMarker>>,
    prev_widget_query: Query<(), With<PreviousWidget>>,
) {
    if !state.pick_mode {
        return;
    }
    let Some(screen_pos) = primary_window.cursor_position() else {
        hover.0 = None;
        return;
    };
    let Some(cursor) = pointer_world.convert(screen_pos) else {
        return;
    };

    let devtools_root = devtools_root_query.iter().next();
    let root = context.get_root_widget();

    // Runs every frame while pick mode is armed (not throttled) so the hover preview tracks
    // the cursor live -- fine cost-wise, since pick mode is a rare, deliberately-armed,
    // short-lived interaction, comparable to the picking backend's own per-frame hit-testing.
    //
    // Reaches every widget with a `WidgetLayout`, not just `Pickable` ones -- most decorative
    // `Element`/text widgets don't opt into `Pickable`, so `is_candidate` accepts everything;
    // only `excluded` narrows the walk, skipping the devtools panel's own subtree (an
    // `Observer` entity, spawned by every `.observe()` call, never has a `WidgetLayout`, so
    // it's implicitly excluded already without needing to check for it here).
    let mut candidates = vec![];
    walk_hit_candidates(
        root,
        cursor,
        &layout_query,
        &children_query,
        &|entity| {
            Some(entity) == devtools_root
                || state_marker_query.contains(entity)
                || prev_widget_query.contains(entity)
        },
        &|_entity| true,
        &mut |entity, layout| candidates.push((entity, layout.order)),
    );
    // `layout.order` (see `layout::system::order_children_for_paint`) is already a single,
    // globally-comparable, correctly-nested paint-order rank on its own -- the topmost
    // candidate is simply whichever has the highest `order`, no separate ranking pass needed.
    let hit = candidates
        .into_iter()
        .max_by_key(|&(_, order)| order)
        .map(|(entity, _)| entity);
    hover.0 = hit;

    if mouse.just_pressed(MouseButton::Left) {
        if let Some(entity) = hit {
            state.selected = Some(entity);
        }
        state.pick_mode = false;
        hover.0 = None;
    }
}
