// The fundamental bones of how Woodpecker UI works; more complex widget-handling
// mechanisms live in entity_mapping.rs, hook_helper.rs, and children.rs, but most of it
// is driven from here.

use bevy::{ecs::change_detection::Tick, platform::collections::HashMap, prelude::*};
use bevy_trait_query::One;

use crate::{
    children::WidgetChildren, context::Widget, hook_helper::StateMarker, metrics::WidgetMetrics,
    prelude::PreviousWidget, CurrentWidget, WoodpeckerContext,
};

pub(crate) fn system(world: &mut World) {
    let mut context = world.remove_resource::<WoodpeckerContext>().unwrap();
    let root_widget = context.get_root_widget();

    let mut new_ticks = HashMap::new();

    let mut widget_query_state =
        QueryState::<One<&dyn Widget>, Without<PreviousWidget>>::new(world);

    let widgets_list = {
        let _ = info_span!("Query Widget Entities", name = "Query Widget Entities").entered();
        vec![root_widget]
            .into_iter()
            .chain(get_all_children(world, root_widget))
            .filter(|e| {
                if world.get_entity(*e).is_err() {
                    return false;
                }
                (!world.entity(*e).contains::<PreviousWidget>()
                    && !world.entity(*e).contains::<Observer>())
                    && !world
                        .entity(*e)
                        .contains::<crate::hook_helper::StateMarker>()
            })
            .collect::<Vec<_>>()
    };

    let mut metrics = world.remove_resource::<WidgetMetrics>().unwrap();
    metrics.clear_last_frame();

    {
        let _ = info_span!(
            "Update and render widgets",
            name = "Update and render widgets"
        )
        .entered();
        for widget_entity in widgets_list {
            update_widgets(
                world,
                widget_entity,
                &mut context,
                &mut metrics,
                &mut new_ticks,
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
            widget_entity,
            widget_query_state,
        );
    }
}

// Recursively gets all widget children down the tree for a given entity.
pub fn get_all_children(world: &mut World, parent_entity: Entity) -> Vec<Entity> {
    let mut children = vec![];
    let Some(bevy_children) = world
        .entity(parent_entity)
        .get::<Children>()
        .map(|c| c.iter().collect::<Vec<_>>())
    else {
        return vec![];
    };
    for child in bevy_children.into_iter() {
        if world.get_entity(child).is_err() {
            continue;
        }
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
        debug!("Woodpecker UI: Missing widget data, this can be safely ignored in most cases.");
        return false;
    };

    let local_name = widget.get_name_local();

    // Universal, reflection-based diff: replaces what used to be per-widget-type,
    // macro-generated prop/state/context diffing. See `diffing::diff_widget_entity`.
    let generically_changed = crate::diffing::diff_widget_entity(world, widget_entity);

    let is_uninitialized = context.get_uninitialized(local_name.clone());
    let Some(update) = context.get_update_system(local_name.clone()) else {
        error!("Woodpecker UI: Please register widgets and their systems!");
        return false;
    };

    if is_uninitialized {
        update.initialize(world);
    }

    world.insert_resource(CurrentWidget(widget_entity));
    // Save/restore the tick around this run so widgets sharing the same system get
    // consistent change detection between updates.
    let old_tick = update.get_last_run();
    let should_update = update.run_without_applying_deferred((), world).unwrap();
    // TODO: Do we actually care for update which honestly should be readonly?
    update.apply_deferred(world);
    let new_tick = update.get_last_run();
    new_ticks.insert(local_name, new_tick);
    update.set_last_run(old_tick);
    world.remove_resource::<CurrentWidget>();

    generically_changed || should_update
}

fn run_render_system(
    world: &mut World,
    context: &mut WoodpeckerContext,
    metrics: &mut WidgetMetrics,
    new_ticks: &mut HashMap<String, Tick>,
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
    render.run_without_applying_deferred((), world).unwrap();
    let new_tick = render.get_last_run();
    new_ticks.insert(widget_name.clone(), new_tick);
    render.set_last_run(old_tick);
    render.apply_deferred(world);
    world.remove_resource::<CurrentWidget>();

    // Step 4: If there are children that have been added process them now! This is also
    // where stale children (whose key wasn't re-declared this pass) get despawned -- see
    // `WidgetMapper::finish_reconciliation`, called from `WidgetChildren::process_world`.
    if let Some(mut children) = world
        .entity_mut(widget_entity)
        .get::<WidgetChildren>()
        .cloned()
    {
        children.process_world(world);
        world.entity_mut(widget_entity).insert(children);
    }

    // A this point we should have initialized both the update and render systems.
    context.remove_uninitialized(widget_name);
}

#[cfg(test)]
mod tests {
    use crate::{
        children::WidgetChildren, prelude::WidgetMapper, CurrentWidget, ObserverCache,
        WidgetRegisterExt, WoodpeckerContext,
    };
    use bevy::prelude::*;

    use super::get_all_children;

    #[derive(Component, Reflect, Default, Clone)]
    struct TestRootWidget;

    impl crate::context::Widget for TestRootWidget {
        fn update() -> impl System<In = (), Out = bool>
        where
            Self: Sized,
        {
            // Always re-render, so we get to exercise render_test_root every frame.
            IntoSystem::into_system(|| true)
        }

        fn render() -> impl System<In = (), Out = ()>
        where
            Self: Sized,
        {
            IntoSystem::into_system(render_test_root)
        }
    }

    #[derive(Component, Reflect, Default, Clone)]
    struct TestLeafWidget;
    impl crate::context::Widget for TestLeafWidget {}

    #[derive(Event, Clone)]
    struct TestClick;

    fn render_test_root(current_widget: Res<CurrentWidget>, mut query: Query<&mut WidgetChildren>) {
        let Ok(mut children) = query.get_mut(**current_widget) else {
            return;
        };
        *children = WidgetChildren::default();
        children.add::<TestLeafWidget>(TestLeafWidget);
        children.observe(*current_widget, |_trigger: On<TestClick>| {});
        children.apply(current_widget.as_parent());
    }

    fn find_observer_entity(world: &World, target: Entity) -> Option<Entity> {
        world
            .entity(target)
            .get::<Children>()?
            .iter()
            .find(|e| world.get_entity(*e).is_ok_and(|e| e.contains::<Observer>()))
    }

    /// Regression test: `run_render_system` used to despawn every observer a widget owned
    /// before each re-render, even when nothing changed, forcing recreation every frame.
    /// Drives the real `runner::system` entry point across two frames and asserts the
    /// child entity and its observer both survive unchanged.
    #[test]
    fn observer_and_child_survive_across_full_runner_frames() {
        let mut app = App::new();
        app.register_widget::<TestRootWidget>();
        app.register_widget::<TestLeafWidget>();
        app.world_mut().insert_resource(WidgetMapper::new());
        app.world_mut().insert_resource(ObserverCache::default());
        app.world_mut()
            .insert_resource(crate::metrics::WidgetMetrics::default());

        let root_entity = app
            .world_mut()
            .spawn((TestRootWidget, WidgetChildren::default()))
            .id();
        app.world_mut()
            .resource_mut::<WoodpeckerContext>()
            .set_root_widget(root_entity);

        super::system(app.world_mut());

        let child_entity_1 = get_all_children(app.world_mut(), root_entity)
            .into_iter()
            .next()
            .expect("root should have a child after first frame");
        let observer_entity_1 = find_observer_entity(app.world(), child_entity_1)
            .expect("child should have an observer after first frame");

        super::system(app.world_mut());

        let child_entity_2 = get_all_children(app.world_mut(), root_entity)
            .into_iter()
            .next()
            .expect("root should have a child after second frame");
        assert_eq!(
            child_entity_1, child_entity_2,
            "child entity must be reused across frames"
        );

        let observer_entity_2 = find_observer_entity(app.world(), child_entity_2)
            .expect("child should still have an observer after second frame");
        assert_eq!(
            observer_entity_1, observer_entity_2,
            "observer entity must survive an unchanged re-render across real runner::system \
             frames, not be despawned and recreated"
        );
    }
}
