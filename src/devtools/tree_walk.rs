use bevy::prelude::*;

use crate::{
    hook_helper::StateMarker,
    prelude::{PreviousWidget, WidgetRender},
    widgets::TreeNode,
    WoodpeckerContext,
};

use super::{
    dump, highlight, panel::DevtoolsRoot, DevtoolsHover, DevtoolsRefreshTimer, DevtoolsSnapshot,
    DevtoolsState,
};

/// True for any entity the inspector should show: not a hook-state/context entity, not a
/// diff-snapshot entity, not an `Observer` entity (every `.observe()` call spawns one as a
/// real `ChildOf` child of its target widget -- `runner::system`'s own widget list filters
/// these out too, via the same `Without<Observer>` check), and not part of the devtools
/// panel's own subtree (which lives in the very same tree it inspects, so it must explicitly
/// exclude itself).
fn is_inspectable(world: &World, entity: Entity, devtools_root: Option<Entity>) -> bool {
    Some(entity) != devtools_root
        && world.get::<StateMarker>(entity).is_none()
        && world.get::<PreviousWidget>(entity).is_none()
        && world.get::<Observer>(entity).is_none()
}

/// The tree label for `entity` -- its widget type name plus, when available, a short preview
/// of its actual text content (e.g. `Element: "Type to filter..."`), since most widgets in a
/// real app are generic `Element`/`WButton` types and are otherwise indistinguishable from
/// their siblings in a bare type-name list. Always ends with `#<index>` so entries stay
/// distinct even when the preview is empty or repeated.
fn entity_label(world: &World, entity: Entity) -> String {
    let type_name = world
        .get::<Name>(entity)
        .map(|name| name.as_str().to_string())
        .unwrap_or_else(|| "<entity>".to_string());

    let preview = match world.get::<WidgetRender>(entity) {
        Some(WidgetRender::Text { content }) => Some(truncate(content)),
        _ => None,
    };

    match preview {
        Some(preview) => format!("{type_name} #{}: \"{preview}\"", entity.index()),
        None => format!("{type_name} #{}", entity.index()),
    }
}

fn truncate(text: &str) -> String {
    const MAX_CHARS: usize = 28;
    let collapsed: String = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.chars().count() > MAX_CHARS {
        let head: String = collapsed.chars().take(MAX_CHARS).collect();
        format!("{head}…")
    } else {
        collapsed
    }
}

fn build_children(world: &World, entity: Entity, devtools_root: Option<Entity>) -> Vec<TreeNode> {
    let Some(children) = world.get::<Children>(entity) else {
        return Vec::new();
    };
    children
        .iter()
        .filter(|&child| is_inspectable(world, child, devtools_root))
        .map(|child| TreeNode {
            key: child.to_bits().to_string(),
            label: entity_label(world, child),
            children: build_children(world, child, devtools_root),
        })
        .collect()
}

/// Exclusive: rebuilding the tree and dumping a selected entity's components both need
/// arbitrary, untyped `&World` access (see `dump::dump_entity`) that a plain `Query` can't
/// express. Throttled via `DevtoolsRefreshTimer` -- re-walking a widget-rich app's whole tree
/// every frame would be wasted churn for a tool whose whole point is human-paced inspection.
pub(crate) fn refresh_snapshot_system(world: &mut World) {
    if !world.resource::<DevtoolsState>().open {
        return;
    }

    let delta = world.resource::<Time>().delta();
    let just_finished = {
        let mut timer = world.resource_mut::<DevtoolsRefreshTimer>();
        timer.0.tick(delta);
        timer.0.just_finished()
    };
    if !just_finished {
        return;
    }

    let devtools_root = world
        .query_filtered::<Entity, With<DevtoolsRoot>>()
        .iter(world)
        .next();
    let root_widget = world.resource::<WoodpeckerContext>().get_root_widget();

    let tree = build_children(world, root_widget, devtools_root);

    let state = world.resource::<DevtoolsState>().clone();
    let hovered = world.resource::<DevtoolsHover>().0;
    let target = state.selected.or(hovered);

    // A stale selection (the inspected app re-rendered and despawned/respawned this entity)
    // fails soft: clear it and show a placeholder, never panic or keep dumping garbage.
    if let Some(selected) = state.selected {
        if world.get_entity(selected).is_err() {
            world.resource_mut::<DevtoolsState>().selected = None;
        }
    }

    // Empty when nothing is selected -- `panel.rs` renders its own placeholder in that case,
    // rather than this being baked into the string here.
    let detail_text = state
        .selected
        .filter(|&e| world.get_entity(e).is_ok())
        .map(|entity| dump::dump_entity_and_state(world, entity))
        .unwrap_or_default();

    let highlight = target
        .filter(|&e| world.get_entity(e).is_ok())
        .and_then(|e| highlight::highlight_rect(world, e));

    let new_snapshot = DevtoolsSnapshot {
        tree,
        detail_text,
        highlight,
    };
    let mut snapshot = world.resource_mut::<DevtoolsSnapshot>();
    if *snapshot != new_snapshot {
        *snapshot = new_snapshot;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Event, Clone)]
    struct DummyEvent;

    #[test]
    fn excludes_state_marker_previous_widget_observers_and_devtools_subtree() {
        let mut world = World::new();
        let devtools = world.spawn(Name::new("DevtoolsRoot")).id();
        let root = world.spawn(Name::new("Root")).id();
        let normal_child = world.spawn(Name::new("Element")).id();
        let state_child = world.spawn((Name::new("State"), StateMarker)).id();
        let prev_child = world.spawn((Name::new("Prev"), PreviousWidget)).id();
        let devtools_child = world.spawn(Name::new("PanelButton")).id();
        // Every `.observe()` call spawns a real `ChildOf` child entity carrying `Observer` --
        // this must not show up in the tree as if it were a widget.
        let observer_child = world.spawn(Observer::new(|_: On<DummyEvent>| {})).id();

        world.entity_mut(root).add_children(&[
            normal_child,
            state_child,
            prev_child,
            observer_child,
            devtools,
        ]);
        // The devtools panel's own subtree lives under `devtools`, in the very same tree as
        // everything else -- this is what the self-exclusion check has to filter out.
        world.entity_mut(devtools).add_children(&[devtools_child]);

        let tree = build_children(&world, root, Some(devtools));
        let labels: Vec<_> = tree.iter().map(|n| n.label.clone()).collect();

        assert_eq!(labels, vec![format!("Element #{}", normal_child.index())]);
    }
}
