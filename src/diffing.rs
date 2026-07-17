use bevy::{ecs::reflect::AppTypeRegistry, prelude::*, reflect::PartialReflect};
use core::any::TypeId;

use crate::{
    children::{diff_passed_children, Mounted, WidgetChildren},
    diffable_prop::ReflectDiffableProp,
    hook_helper::HookHelper,
    layout::system::diff_watched_layout,
    previous_snapshot::PreviousSnapshot,
    widgets::{
        animation::{diff_animation_timeline, diff_spring},
        popover::diff_popover_content,
        text_box::diff_text_box_state,
        transition::diff_transition,
        window::diff_window_wrapper_layout,
    },
};

/// Computes whether `entity` (a widget entity) needs to re-render this frame, updating its
/// [`PreviousSnapshot`] to reflect the values just compared.
///
/// Any component on `entity` that derives `Reflect` and opts into
/// [`crate::diffable_prop::DiffableProp`] is compared against its last-diffed snapshot. A few
/// things also force a render regardless of prop diffing: redeclared children, a fresh
/// `Mounted` marker, `Transition`'s playing-state, `AnimationTimeline`'s playing-state,
/// `Spring`'s settled-state, and `PassedChildren` (which can't derive `Reflect`).
pub(crate) fn diff_widget_entity(world: &mut World, entity: Entity) -> bool {
    if let Some(children) = world.get::<WidgetChildren>(entity) {
        if children.children_changed() {
            return true;
        }
    }

    // `Mounted` is (re)inserted whenever a parent (re)declares this widget; the first check
    // to observe it forces a render and removes it, so later checks fall through to the value diff.
    if world.get::<Mounted>(entity).is_some() {
        world.entity_mut(entity).remove::<Mounted>();
        return true;
    }

    if diff_transition(world, entity) {
        return true;
    }

    if diff_animation_timeline(world, entity) {
        return true;
    }

    if diff_spring(world, entity) {
        return true;
    }

    if diff_passed_children(world, entity) {
        return true;
    }

    if diff_popover_content(world, entity) {
        return true;
    }

    if diff_watched_layout(world, entity) {
        return true;
    }

    if diff_window_wrapper_layout(world, entity) {
        return true;
    }

    if diff_text_box_state(world, entity) {
        return true;
    }

    // Cheap: `AppTypeRegistry` wraps an `Arc`, so cloning it just bumps a refcount and lets
    // us drop the borrow of `world` used to fetch it before we need `world` again below.
    let type_registry = world.resource::<AppTypeRegistry>().clone();
    let registry = type_registry.read();

    let mut changed = false;
    let mut updates: Vec<(TypeId, Box<dyn PartialReflect>)> = Vec::new();

    // Props live on the widget's own entity; state/context live on separate entities (see
    // hook_helper.rs) and must be discovered and walked too.
    let mut entities_to_diff = vec![entity];
    if let Some(hook_helper) = world.get_resource::<HookHelper>() {
        entities_to_diff.extend(hook_helper.own_state_entities(entity));
        for context_entity in hook_helper.visible_context_entities(entity) {
            if !entities_to_diff.contains(&context_entity) {
                entities_to_diff.push(context_entity);
            }
        }
    }

    for source_entity in entities_to_diff {
        let Ok(entity_ref) = world.get_entity(source_entity) else {
            continue;
        };
        for &component_id in entity_ref.archetype().components() {
            let Some(info) = world.components().get_info(component_id) else {
                continue;
            };
            let Some(type_id) = info.type_id() else {
                continue;
            };
            let Some(registration) = registry.get(type_id) else {
                continue;
            };
            if registration.data::<ReflectDiffableProp>().is_none() {
                continue;
            }
            let Some(reflect_component) = registration.data::<ReflectComponent>() else {
                warn!(
                    "Woodpecker UI: {} opts into DiffableProp but is missing \
                     #[reflect(Component)] -- it will never be diffed.",
                    registration.type_info().type_path()
                );
                continue;
            };
            let Some(current) = reflect_component.reflect(entity_ref) else {
                continue;
            };
            let current = current.as_partial_reflect();

            let previous = world
                .get::<PreviousSnapshot>(entity)
                .and_then(|snapshot| snapshot.get(type_id));
            let is_changed = match previous {
                // No snapshot yet for this type -- newly observed, treat as changed.
                None => true,
                Some(previous) => match current.reflect_partial_eq(previous) {
                    Some(equal) => !equal,
                    // The type doesn't support equality testing -- err on the side of
                    // re-rendering rather than silently going stale.
                    None => {
                        warn!(
                            "Woodpecker UI: {} does not support reflection-based equality \
                             testing; it will always be treated as changed. Add \
                             #[reflect(PartialEq)] to fix this.",
                            registration.type_info().type_path()
                        );
                        true
                    }
                },
            };

            if is_changed {
                changed = true;
                match current.reflect_clone() {
                    Ok(cloned) => updates.push((type_id, cloned.into_partial_reflect())),
                    Err(_) => {
                        warn!(
                            "Woodpecker UI: could not clone {} for diffing; it will always be \
                             treated as changed. Consider deriving Clone and adding \
                             #[reflect(Clone)].",
                            registration.type_info().type_path()
                        );
                    }
                }
            }
        }
    }
    drop(registry);

    if !updates.is_empty() {
        let mut entity_mut = world.entity_mut(entity);
        if !entity_mut.contains::<PreviousSnapshot>() {
            entity_mut.insert(PreviousSnapshot::default());
        }
        let mut snapshot = entity_mut.get_mut::<PreviousSnapshot>().unwrap();
        for (type_id, value) in updates {
            snapshot.set(type_id, value);
        }
    }

    changed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::children::PassedChildren;

    #[derive(Component, Reflect, Clone, PartialEq, Default)]
    #[reflect(Component, DiffableProp, PartialEq)]
    struct DiffedProp(i32);

    #[derive(Component, Reflect, Clone, PartialEq, Default)]
    #[reflect(Component, PartialEq)]
    struct NonDiffedProp(i32);

    fn test_app() -> App {
        let mut app = App::new();
        app.register_type::<DiffedProp>();
        app.register_type::<NonDiffedProp>();
        app
    }

    #[test]
    fn first_diff_of_a_new_entity_is_always_changed() {
        let mut app = test_app();
        let entity = app.world_mut().spawn(DiffedProp(1)).id();
        assert!(diff_widget_entity(app.world_mut(), entity));
    }

    #[test]
    fn unchanged_value_is_not_changed_on_subsequent_diffs() {
        let mut app = test_app();
        let entity = app.world_mut().spawn(DiffedProp(1)).id();
        assert!(
            diff_widget_entity(app.world_mut(), entity),
            "first diff always changed"
        );
        assert!(
            !diff_widget_entity(app.world_mut(), entity),
            "re-inserting/observing the same value must not be treated as changed"
        );
        assert!(
            !diff_widget_entity(app.world_mut(), entity),
            "should remain settled indefinitely while nothing changes"
        );
    }

    #[test]
    fn mutated_value_is_detected_as_changed() {
        let mut app = test_app();
        let entity = app.world_mut().spawn(DiffedProp(1)).id();
        diff_widget_entity(app.world_mut(), entity);
        assert!(!diff_widget_entity(app.world_mut(), entity));

        app.world_mut().get_mut::<DiffedProp>(entity).unwrap().0 = 2;
        assert!(
            diff_widget_entity(app.world_mut(), entity),
            "mutating the diffed prop's value must be detected"
        );
        assert!(
            !diff_widget_entity(app.world_mut(), entity),
            "must settle again after the change is observed once"
        );
    }

    #[test]
    fn non_diffable_component_never_affects_the_result() {
        let mut app = test_app();
        let entity = app
            .world_mut()
            .spawn((DiffedProp(1), NonDiffedProp(1)))
            .id();
        diff_widget_entity(app.world_mut(), entity);
        assert!(!diff_widget_entity(app.world_mut(), entity));

        app.world_mut().get_mut::<NonDiffedProp>(entity).unwrap().0 = 99;
        assert!(
            !diff_widget_entity(app.world_mut(), entity),
            "a component that never opted into DiffableProp must not force a re-render"
        );
    }

    #[test]
    fn mounted_marker_forces_one_render_then_is_consumed() {
        let mut app = test_app();
        let entity = app.world_mut().spawn((DiffedProp(1), Mounted)).id();
        assert!(
            diff_widget_entity(app.world_mut(), entity),
            "a freshly-mounted widget must render at least once"
        );
        assert!(
            app.world().get::<Mounted>(entity).is_none(),
            "Mounted must be consumed (removed) once observed"
        );

        // Mounted's early return skips recording DiffedProp's snapshot on the same call, so
        // the next call still reports changed once more, for DiffedProp's first snapshot.
        assert!(
            diff_widget_entity(app.world_mut(), entity),
            "the prop snapshot is recorded on this call, so it still reports changed once more"
        );
        assert!(
            !diff_widget_entity(app.world_mut(), entity),
            "must settle once both Mounted and the initial prop snapshot have been consumed"
        );
    }

    #[test]
    fn passed_children_change_forces_a_render_independent_of_other_props() {
        let mut app = test_app();
        let entity = app
            .world_mut()
            .spawn((DiffedProp(1), PassedChildren::default()))
            .id();
        // Settle both PassedChildren's and DiffedProp's initial snapshots (each early-return
        // check skips snapshotting anything after it, so settling takes one call per check).
        diff_widget_entity(app.world_mut(), entity);
        diff_widget_entity(app.world_mut(), entity);
        assert!(
            !diff_widget_entity(app.world_mut(), entity),
            "must be fully settled by now"
        );

        let mut passed = WidgetChildren::default();
        #[derive(Component, Default, Clone)]
        struct Leaf;
        impl crate::context::Widget for Leaf {}
        passed.add::<Leaf>(Leaf);
        app.world_mut()
            .entity_mut(entity)
            .insert(PassedChildren(passed));

        assert!(
            diff_widget_entity(app.world_mut(), entity),
            "a changed PassedChildren value must force a render even though it can't derive Reflect"
        );
    }
}
