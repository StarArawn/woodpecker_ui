// This runner system is the fundamental bones of how Woodpecker UI works.
// Its relatively simple in operation although more complex mechanisms can be found
// elsewhere in the code base for widget handling like:
// - entity_mapping.rs
// - hook_helper.rs
// - children.rs
// Most of the functionality in those files that runs starts here in this file.

use bevy::{
    ecs::component::Tick,
    platform::collections::{HashMap, HashSet},
    prelude::*,
};
use bevy_trait_query::One;

use crate::{
    children::WidgetChildren,
    context::Widget,
    hook_helper::StateMarker,
    metrics::WidgetMetrics,
    prelude::{PreviousWidget, WidgetMapper},
    CurrentWidget, WoodpeckerContext,
};

pub(crate) fn system(world: &mut World) {
    let mut context = world.remove_resource::<WoodpeckerContext>().unwrap();
    let root_widget = context.get_root_widget();

    let mut new_ticks = HashMap::new();

    let mut widget_query_state =
        QueryState::<One<&dyn Widget>, Without<PreviousWidget>>::new(world);

    // STEP 1: Run update systems and mark widgets as needing to be re-rendered
    // Note: re-rendering means to re-build the sub-tree at X point in the tree.
    world.resource_scope(|_world: &mut World, mut widget_mapper: Mut<WidgetMapper>| {
        widget_mapper.clear_added_this_frame();
    });

    let widgets_list = {
        let _ = info_span!("Query Widget Entities", name = "Query Widget Entities").entered();
        vec![root_widget]
            .into_iter()
            .chain(get_all_children(world, root_widget))
            .filter(|e| {
                if world.get_entity(*e).is_err() {
                    return false;
                }
                !world.entity(*e).contains::<PreviousWidget>()
                    && !world
                        .entity(*e)
                        .contains::<crate::hook_helper::StateMarker>()
            })
            .collect::<Vec<_>>()
    };

    let mut removed_list = HashSet::default();
    let mut metrics = world.remove_resource::<WidgetMetrics>().unwrap();
    metrics.clear_last_frame();

    {
        let _ = info_span!(
            "Update and render widgets",
            name = "Update and render widgets"
        )
        .entered();
        for widget_entity in widgets_list {
            // Skip removed widgets.
            if removed_list.contains(&widget_entity) {
                continue;
            }

            update_widgets(
                world,
                widget_entity,
                &mut context,
                &mut metrics,
                &mut new_ticks,
                &mut removed_list,
                &mut widget_query_state,
            );
        }
    }

    metrics.commit_frame();
    world.insert_resource(metrics);

    // Step 5: Restore system ticks
    let tick = world.read_change_tick();
    for (key, system) in context.widgets.iter_mut() {
        if let Some(new_tick) = new_ticks.get(key) {
            system.0.set_last_run(*new_tick);
            system.1.set_last_run(*new_tick);
        } else {
            system.0.set_last_run(tick);
            system.1.set_last_run(tick);
        }
    }

    world.insert_resource(context);
}

// Runs the update system which tells us which entities should "re-render".
fn update_widgets(
    world: &mut World,
    widget_entity: Entity,
    context: &mut WoodpeckerContext,
    metrics: &mut WidgetMetrics,
    new_ticks: &mut HashMap<String, Tick>,
    removed_list: &mut HashSet<Entity>,
    widget_query_state: &mut QueryState<One<&dyn Widget>, Without<PreviousWidget>>,
) {
    // STEP 2: Diff widgets
    if run_update_system(world, widget_entity, context, new_ticks, widget_query_state) {
        // Step 3: Run render system.
        run_render_system(
            world,
            context,
            metrics,
            new_ticks,
            removed_list,
            widget_entity,
            widget_query_state,
        );
    }
}

// Recursively gets all widget children down the tree for a given entity.
fn get_all_children(world: &mut World, parent_entity: Entity) -> Vec<Entity> {
    let mut children = vec![];
    let Some(bevy_children) = world
        .entity(parent_entity)
        .get::<Children>()
        .map(|c| c.iter().collect::<Vec<_>>())
    else {
        return vec![];
    };
    for child in bevy_children.into_iter() {
        // Only widget entities should be traversed here
        if !world.entity(child).contains::<StateMarker>()
            && !world.entity(child).contains::<PreviousWidget>()
        {
            children.push(child);
            children.extend(get_all_children(world, child));
        }
    }
    children
}

fn run_update_system(
    world: &mut World,
    widget_entity: Entity,
    context: &mut WoodpeckerContext,
    new_ticks: &mut HashMap<String, Tick>,
    widget_query_state: &mut QueryState<One<&dyn Widget>, Without<PreviousWidget>>,
) -> bool {
    let Ok(widget) = widget_query_state.get(world, widget_entity) else {
        error!("Woodpecker UI: Missing widget data!");
        return false;
    };

    let local_name = widget.get_name_local();
    let is_uninitialized = context.get_uninitialized(local_name.clone());
    let Some(update) = context.get_update_system(local_name.clone()) else {
        error!("Woodpecker UI: Please register widgets and their systems!");
        return false;
    };

    if is_uninitialized {
        update.initialize(world);
    }

    world.insert_resource(CurrentWidget(widget_entity));
    // Store the original tick.
    // We do this so that between widget updates of the same
    // type we get a consistent "tick", meaning change detection
    // works as expected.
    let old_tick = update.get_last_run();
    let should_update = update.run((), world);
    // Apply commands and other things to world.
    // TODO: Do we actually care for update which honestly
    // should be readonly?
    update.apply_deferred(world);
    // Get the new tick.
    let new_tick = update.get_last_run();
    // Store the new tick after all the widgets have finished
    // we insert this back onto the system.
    new_ticks.insert(local_name, new_tick);
    // Restore the original tick so that when the next
    // widget of the same type runs this we get consistent
    // change detection and events.
    update.set_last_run(old_tick);
    world.remove_resource::<CurrentWidget>();

    should_update
}

fn run_render_system(
    world: &mut World,
    context: &mut WoodpeckerContext,
    metrics: &mut WidgetMetrics,
    new_ticks: &mut HashMap<String, Tick>,
    removed_list: &mut HashSet<Entity>,
    widget_entity: Entity,
    widget_query_state: &mut QueryState<One<&dyn Widget>, Without<PreviousWidget>>,
) {
    // Pull widget data.
    let Ok(widget) = widget_query_state.get(world, widget_entity) else {
        error!("Woodpecker UI: Missing widget data for {}!", widget_entity);
        return;
    };
    let widget_name = widget.get_name_local();

    // Initialize the systems if needed.
    let is_uninitialized = context.get_uninitialized(widget_name.clone());
    let Some(render) = context.get_render_system(widget_name.clone()) else {
        error!("Woodpecker UI: Please register widgets and their systems!");
        return;
    };
    if is_uninitialized {
        render.initialize(world);
    }

    trace!("re-rendering: {}-{}", widget_name, widget_entity);
    metrics.increase_counts();
    // Run the render function and apply changes to the bevy world.
    world.insert_resource(CurrentWidget(widget_entity));
    let old_tick = render.get_last_run();
    render.run((), world);
    let new_tick = render.get_last_run();
    new_ticks.insert(widget_name.clone(), new_tick);
    render.set_last_run(old_tick);
    render.apply_deferred(world);
    world.remove_resource::<CurrentWidget>();

    // Step 4: If there are children that have been added process them now!
    if let Some(mut children) = world
        .entity_mut(widget_entity)
        .get::<WidgetChildren>()
        .cloned()
    {
        children.process_world(world);
        world.entity_mut(widget_entity).insert(children);
    }

    // STEP 5: Despawn unmounted widgets.
    world.resource_scope(|world: &mut World, mut widget_mapper: Mut<WidgetMapper>| {
        // Note: Children here are only the immediate children attached to the parent(widget_entity).
        let children = widget_mapper.get_all_children(widget_entity);
        for child in children.iter() {
            // Only remove if the child was not added this frame.
            if !widget_mapper.added_this_frame(*child) {
                trace!("Removing: {child}");
                // Remove from the mapper.
                widget_mapper.remove_by_entity_id(widget_entity, *child);
                // Despawn and despawn recursive.
                removed_list.insert(*child);
                // Entity and its children were despawned lets make sure all of the descendants are removed from the mapper!
                for child in get_all_children(world, *child) {
                    let parent = world.entity(child).get::<ChildOf>().expect("Unknown dangling child! This is an error with woodpecker UI source please file a bug report.").parent();
                    widget_mapper.remove_by_entity_id(parent, child);
                    removed_list.insert(child);
                }
                // Do this last so the parent query still works.
                world.entity_mut(*child).despawn();
            }
        }
    });

    // A this point we should have initialized both the update and render systems.
    context.remove_uninitialized(widget_name);
}
