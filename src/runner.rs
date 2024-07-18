use bevy::prelude::*;
use bevy_trait_query::One;
use std::collections::BTreeSet;

use crate::{
    children::WidgetChildren, context::Widget, prelude::WidgetMapper, CurrentWidget,
    WoodpeckerContext,
};

pub(crate) fn system(world: &mut World) {
    let mut context = world.remove_resource::<WoodpeckerContext>().unwrap();
    let _root_widget = context.get_root_widget();

    // Ordering is important so lets use a BTreeSet!
    let mut re_render_list = BTreeSet::default();

    // STEP 1: Run update systems and mark widgets as needing to be re-rendered
    // Note: re-rendering means to re-build the sub-tree at X point in the tree.
    world.resource_scope(|_world: &mut World, mut widget_mapper: Mut<WidgetMapper>| {
        widget_mapper.clear_added_this_frame();
    });

    for widget_entity in world
        .query::<(Entity, One<&dyn Widget>)>()
        .iter(world)
        .map(|(e, _)| e)
        .collect::<Vec<_>>()
    {
        update_widgets(world, widget_entity, &mut context, &mut re_render_list);
    }

    let mut removed_list = BTreeSet::default();

    // STEP 2: Run render systems which should spawn new widgets.
    for widget_entity in re_render_list.iter() {
        info!("re-rendering: {}", widget_entity);
        // Skip removed widgets.
        if removed_list.contains(widget_entity) {
            continue;
        }
        // Pull widget data.
        let mut widget_query = world.query::<One<&dyn Widget>>();
        let Ok(widget) = widget_query.get(world, *widget_entity) else {
            error!("Woodpecker UI: Missing widget data!");
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

        // Run the render function and apply changes to the bevy world.
        world.insert_resource(CurrentWidget(*widget_entity));
        render.run((), world);
        render.apply_deferred(world);
        world.remove_resource::<CurrentWidget>();

        // Step 3: If there are children that have been added process them now!
        if let Some(mut children) = world
            .entity_mut(*widget_entity)
            .get::<WidgetChildren>()
            .map(|wc| wc.clone())
        {
            children.process_world(world);
            world.entity_mut(*widget_entity).insert(children);
        }

        // STEP 4: Despawn unmounted widgets.
        world.resource_scope(|world: &mut World, mut widget_mapper: Mut<WidgetMapper>| {
            // Note: Children here are only the imediate children attached to the parent(widget_entity).
            let children = widget_mapper.get_all_children(*widget_entity);
            for child in children.iter() {
                // Only remove if the child was not added this frame.
                if !widget_mapper.added_this_frame(*child) {
                    info!("Removing: {child}");
                    // Remove from the mapper.
                    widget_mapper.remove_by_entity_id(*widget_entity, *child);
                    // Depsawn and despawn recursive.
                    removed_list.insert(*child);
                    // Entity and its children were despawned lets make sure all of the descendents are removed from the mapper!
                    for child in get_all_children(world, *child) {
                        let parent = world.query::<&Parent>().get(world, child).expect("Unknown dangling child! This is an error with woodpecker UI source please file a bug report.").get();
                        widget_mapper.remove_by_entity_id(parent, child);
                        removed_list.insert(child);
                    }
                    // Do this last so the parent query still works.
                    world.entity_mut(*child).despawn_recursive();
                }
            }
        });

        if is_uninitialized {
            context.remove_uninitialized(widget_name);
        }
    }

    world.insert_resource(context);
}

// Runs the update system which tells us which entities should "re-render".
fn update_widgets(
    world: &mut World,
    widget_entity: Entity,
    context: &mut WoodpeckerContext,
    re_render_list: &mut BTreeSet<Entity>,
) {
    let mut widget_query = world.query::<One<&dyn Widget>>();
    let Ok(widget) = widget_query.get(world, widget_entity) else {
        error!("Woodpecker UI: Missing widget data!");
        return;
    };

    let is_uninitialized = context.get_uninitialized(widget.get_name_local());
    let Some(update) = context.get_update_system(widget.get_name_local()) else {
        error!("Woodpecker UI: Please register widgets and their systems!");
        return;
    };

    if is_uninitialized {
        update.initialize(world);
    }

    world.insert_resource(CurrentWidget(widget_entity));
    if update.run((), world) {
        update.apply_deferred(world);
        re_render_list.insert(widget_entity);
        // Mark children for re-render.
        let children = get_all_children(world, widget_entity);
        re_render_list.extend(children);
    }
    world.remove_resource::<CurrentWidget>();
}

// Recursively gets all children down the tree for a given entity.
fn get_all_children(world: &mut World, parent_entity: Entity) -> Vec<Entity> {
    let mut children = vec![];
    let Ok(bevy_children) = world
        .query::<&Children>()
        .get(world, parent_entity)
        .map(|c| c.iter().map(|e| *e).collect::<Vec<_>>())
    else {
        return vec![];
    };
    for child in bevy_children.into_iter() {
        children.push(child);
        get_all_children(world, child);
    }
    children
}
