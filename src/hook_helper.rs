use bevy::{platform::collections::HashMap, prelude::*};
use bevy_trait_query::One;

use crate::{context::Widget, portal::LogicalParent, CurrentWidget};

/// A helper resource that keeps track of context(hierachy state) entities and state tied to entities.
#[derive(Resource, Default, Debug, Clone)]
pub struct HookHelper {
    internal_context: HashMap<Entity, HashMap<String, Entity>>,
    parents: HashMap<Entity, Entity>,
    state: HashMap<Entity, HashMap<String, Entity>>,
    prev_state_entities: HashMap<Entity, Entity>,
}

#[derive(Component)]
pub struct StateMarker;

impl HookHelper {
    /// Finds a state entity or creates a new one using commands.
    /// State entities are just entities parented to the entity passed in.
    /// They are useful because they can persist across widget bundle inserts.
    pub fn use_state<T: Component>(
        &mut self,
        commands: &mut Commands,
        current_widget: CurrentWidget,
        initial_state: T,
    ) -> Entity {
        let type_name: String = std::any::type_name::<T>().into();
        if let Some(state_entity) = self.get_state::<T>(current_widget) {
            state_entity
        } else {
            let state_entity = commands
                .spawn((StateMarker, initial_state))
                .insert(ChildOf(*current_widget))
                .id();

            let context_types = self.state.entry(*current_widget).or_default();

            context_types.insert(type_name, state_entity);

            state_entity
        }
    }

    /// Looks up the T state for an entity and returns an `Option<Entity>`
    /// None is returned if the state is not found. Unlike use_state this does
    /// not create new state rather it only looks for existing state.
    /// State entities are just entities parented to the entity passed in.
    pub fn get_state<T: Component>(&self, current_widget: CurrentWidget) -> Option<Entity> {
        let type_name: String = std::any::type_name::<T>().into();
        self.state
            .get(&*current_widget)
            .and_then(|context_types| context_types.get(&type_name).copied())
    }

    /// Traverses the widget tree(bevy hierarchy) and finds the context entity
    /// associated with the given T type.
    pub fn use_context<T: Component>(
        &mut self,
        commands: &mut Commands,
        current_widget: CurrentWidget,
        initial_context: T,
    ) -> Entity {
        let type_name: String = std::any::type_name::<T>().into();
        if let Some(context_entity) = self.traverse_find_context_entity(&type_name, current_widget)
        {
            context_entity
        } else {
            let context_entity = commands
                .spawn((StateMarker, initial_context))
                .insert(ChildOf(*current_widget))
                .id();

            let context_types = self.internal_context.entry(*current_widget).or_default();

            context_types.insert(type_name, context_entity);

            context_entity
        }
    }

    /// Like [`Self::use_context`], but never walks up to an ancestor's context of the same
    /// type -- always finds-or-creates one registered directly on `current_widget` itself.
    ///
    /// Needed by any widget that provides a context of some type `T` purely for its *own*
    /// internal bookkeeping (not meant to be a value descendants should read/inherit), when
    /// that widget might itself be nested under an unrelated ancestor that separately
    /// provides its own `T` context for a different purpose. `use_context`'s ordinary
    /// ancestor walk can't tell those two purposes apart -- it just finds the nearest `T`
    /// context and hands back whichever one is closer, even if that's the wrong one. Found via
    /// `VirtualList`, which nested inside an ordinary `ScrollContextProvider`/`ScrollBox` page
    /// silently adopted *that* `ScrollContext` instead of registering its own private one, so
    /// the two widgets' unrelated scroll bookkeeping (a virtualized list's internal scroll
    /// state vs. the whole page's) fought over the same entity every render -- each write
    /// overwriting the other's values, which cascaded into a permanent, never-settling resize
    /// loop across the entire page.
    pub fn use_own_context<T: Component>(
        &mut self,
        commands: &mut Commands,
        current_widget: CurrentWidget,
        initial_context: T,
    ) -> Entity {
        let type_name: String = std::any::type_name::<T>().into();
        if let Some(context_entity) = self
            .internal_context
            .get(&*current_widget)
            .and_then(|context_types| context_types.get(&type_name))
        {
            *context_entity
        } else {
            let context_entity = commands
                .spawn((StateMarker, initial_context))
                .insert(ChildOf(*current_widget))
                .id();

            let context_types = self.internal_context.entry(*current_widget).or_default();

            context_types.insert(type_name, context_entity);

            context_entity
        }
    }

    /// Like use_context but does not spawn a new context entity it only
    /// looks for an existing one.
    pub fn get_context<T: Component>(&self, current_widget: CurrentWidget) -> Option<Entity> {
        let type_name: String = std::any::type_name::<T>().into();
        self.traverse_find_context_entity(&type_name, current_widget)
    }

    /// Returns every context entity visible to `current_widget` -- its own provided
    /// contexts plus any provided by an ancestor, nearest provider winning. Used by the
    /// reflection-based diffing in `crate::diffing` to discover re-render dependencies.
    pub(crate) fn visible_context_entities(&self, current_widget: Entity) -> Vec<Entity> {
        let mut seen_type_names: bevy::platform::collections::HashSet<String> =
            bevy::platform::collections::HashSet::default();
        let mut result = Vec::new();
        let mut current = current_widget;
        loop {
            if let Some(context_types) = self.internal_context.get(&current) {
                for (type_name, entity) in context_types {
                    if seen_type_names.insert(type_name.clone()) {
                        result.push(*entity);
                    }
                }
            }
            match self.parents.get(&current) {
                Some(&parent) => current = parent,
                None => break,
            }
        }
        result
    }

    /// Returns every state entity `current_widget` owns via `use_state` (its own private
    /// per-instance state, as opposed to inherited context). Used alongside
    /// `visible_context_entities` by the generic reflection-based diffing.
    pub(crate) fn own_state_entities(&self, current_widget: Entity) -> Vec<Entity> {
        self.state
            .get(&current_widget)
            .map(|context_types| context_types.values().copied().collect())
            .unwrap_or_default()
    }

    // Traverse up tree to find parent widget with the context.
    fn traverse_find_context_entity(
        &self,
        type_name: &String,
        current_entity: CurrentWidget,
    ) -> Option<Entity> {
        if let Some(context_entity) = self
            .internal_context
            .get(&*current_entity)
            .and_then(|context_types| context_types.get(type_name))
        {
            return Some(*context_entity);
        }

        // Walk up tree if nothing was found above.
        if let Some(parent) = self.parents.get(&*current_entity) {
            return self.traverse_find_context_entity(type_name, CurrentWidget(*parent));
        }

        None
    }

    pub(crate) fn update_context_helper(
        mut context_helper: ResMut<HookHelper>,
        query: Query<
            (Entity, &ChildOf, Option<&LogicalParent>, One<&dyn Widget>),
            (Changed<ChildOf>, Without<PreviousWidget>),
        >,
        mut removed: RemovedComponents<ChildOf>,
    ) {
        // Add any that were added or changed. A portaled entity's `ChildOf` points at
        // `OverlayRoot`/an explicit portal target, not where it's logically declared --
        // `LogicalParent` (set once at spawn, alongside `ChildOf`) is preferred here so
        // context/theme ancestry still resolves as if it had never moved.
        for (entity, parent, logical_parent, _) in query.iter() {
            let effective_parent = logical_parent.map_or(parent.parent(), |lp| lp.0);
            context_helper.parents.insert(entity, effective_parent);
        }

        // Remove any that were removed.
        for entity in removed.read() {
            context_helper.parents.remove(&entity);
        }
    }

    /// Creates and or returns an entity used for tracking a widgets previous state.
    pub fn get_previous_widget(
        &mut self,
        commands: &mut Commands,
        current_widget: CurrentWidget,
    ) -> Entity {
        let prev_state_entity = self
            .prev_state_entities
            .entry(*current_widget)
            .or_insert_with(|| {
                commands
                    .spawn(PreviousWidget)
                    .insert(ChildOf(*current_widget))
                    .id()
            });

        *prev_state_entity
    }
}

/// A tag component used to mark previous widget entities.
#[derive(Component)]
pub struct PreviousWidget;

#[cfg(test)]
mod tests {
    use bevy::ecs::system::RunSystemOnce;

    use super::*;
    use crate::WidgetRegisterExt;

    #[derive(Component, Reflect, Default, Clone)]
    struct TestWidget;
    impl Widget for TestWidget {}

    #[derive(Component)]
    struct TestContext;

    /// A portaled entity's `ChildOf` points at `OverlayRoot`/an explicit portal target, not
    /// where it's logically declared -- context ancestry must still resolve through
    /// `LogicalParent`, exactly as if the entity had never physically moved. Builds:
    /// `grandparent` (provides a `TestContext`) -> `logical_parent` -> `portaled` (physically
    /// `ChildOf(overlay_root)`, `LogicalParent(logical_parent)`) -> `portaled_child` (a normal,
    /// non-portaled descendant of `portaled`). `portaled_child` must still find the context
    /// provided on `grandparent`.
    #[test]
    fn context_lookup_resolves_through_logical_parent_not_physical_child_of() {
        let mut app = App::new();
        app.register_widget::<TestWidget>();
        app.init_resource::<HookHelper>();

        let overlay_root = app.world_mut().spawn_empty().id();
        let grandparent = app.world_mut().spawn(TestWidget).id();
        let context_entity = app
            .world_mut()
            .spawn((StateMarker, TestContext, ChildOf(grandparent)))
            .id();
        app.world_mut()
            .resource_mut::<HookHelper>()
            .internal_context
            .entry(grandparent)
            .or_default()
            .insert(std::any::type_name::<TestContext>().into(), context_entity);

        let logical_parent = app
            .world_mut()
            .spawn((TestWidget, ChildOf(grandparent)))
            .id();
        let portaled = app
            .world_mut()
            .spawn((
                TestWidget,
                ChildOf(overlay_root),
                LogicalParent(logical_parent),
            ))
            .id();
        let portaled_child = app.world_mut().spawn((TestWidget, ChildOf(portaled))).id();

        app.world_mut()
            .run_system_once(HookHelper::update_context_helper)
            .unwrap();

        let found = app
            .world()
            .resource::<HookHelper>()
            .get_context::<TestContext>(CurrentWidget(portaled_child));
        assert_eq!(
            found,
            Some(context_entity),
            "a portaled entity's descendant must still resolve context provided on its \
             *logical* ancestor, not its physical ChildOf chain through OverlayRoot"
        );
    }

    /// Regression test for the bug `use_own_context` was added to fix: a widget nested inside
    /// an unrelated ancestor that already provides a context of the *same* component type
    /// (e.g. `VirtualList` nested inside a page's own `ScrollContextProvider`/`ScrollBox`, both
    /// providing `ScrollContext`) must get its own private context entity, not silently adopt
    /// the ancestor's -- confirmed here against the plain `TestContext` marker, standing in for
    /// `ScrollContext`.
    #[test]
    fn use_own_context_does_not_adopt_an_ancestors_context() {
        let mut app = App::new();
        app.register_widget::<TestWidget>();
        app.init_resource::<HookHelper>();

        let parent = app.world_mut().spawn(TestWidget).id();
        let parent_context = app
            .world_mut()
            .spawn((StateMarker, TestContext, ChildOf(parent)))
            .id();
        app.world_mut()
            .resource_mut::<HookHelper>()
            .internal_context
            .entry(parent)
            .or_default()
            .insert(std::any::type_name::<TestContext>().into(), parent_context);

        let child = app.world_mut().spawn((TestWidget, ChildOf(parent))).id();

        app.world_mut()
            .run_system_once(HookHelper::update_context_helper)
            .unwrap();

        let found_via_use_context = app
            .world_mut()
            .run_system_once(
                move |mut hooks: ResMut<HookHelper>, mut commands: Commands| {
                    hooks.use_context(&mut commands, CurrentWidget(child), TestContext)
                },
            )
            .unwrap();
        assert_eq!(found_via_use_context, parent_context);

        let found_via_use_own_context = app
            .world_mut()
            .run_system_once(
                move |mut hooks: ResMut<HookHelper>, mut commands: Commands| {
                    hooks.use_own_context(&mut commands, CurrentWidget(child), TestContext)
                },
            )
            .unwrap();
        assert_ne!(
            found_via_use_own_context, parent_context,
            "use_own_context must never adopt an ancestor's context of the same type"
        );
    }
}
