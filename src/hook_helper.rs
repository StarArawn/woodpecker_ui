use bevy::{prelude::*, utils::HashMap};
use bevy_trait_query::One;

use crate::{context::{self, Widget}, CurrentWidget};

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
                .set_parent(*current_widget)
                .id();

            let context_types = self.state.entry(*current_widget).or_default();

            context_types.insert(type_name, state_entity);

            state_entity
        }
    }

    /// Looks up the T state for an entity and returns an Option<Entity>
    /// None is returned if the state is not found. Unlike use_state this does
    /// not create new state rather it only looks for existing state.
    /// State entities are just entities parented to the entity passed in.
    pub fn get_state<T: Component>(&self, current_widget: CurrentWidget) -> Option<Entity> {
        let type_name: String = std::any::type_name::<T>().into();
        self.state
            .get(&*current_widget)
            .and_then(|context_types| context_types.get(&type_name).copied())
    }

    pub(crate) fn get_all_state_entities(&self, current_widget: CurrentWidget) -> Option<Vec<&Entity>> {
        self.state.get(&*current_widget).map(|context_types| context_types.values().collect::<Vec<_>>())
    }

    /// Traverses the widget tree(bevy hierarchy) and finds the context entity
    /// associated with the given T type.
    pub fn use_context<T: Component>(
        &mut self,
        commands: &mut Commands,
        current_widget: CurrentWidget,
    ) -> Entity {
        let type_name: String = std::any::type_name::<T>().into();
        if let Some(context_entity) = self.traverse_find_context_entity(&type_name, current_widget)
        {
            context_entity
        } else {
            let context_entity = commands.spawn(StateMarker).set_parent(*current_widget).id();

            let context_types = self.internal_context.entry(*current_widget).or_default();

            context_types.insert(type_name, context_entity);

            context_entity
        }
    }

    /// Like use_context but does not spawn a new context entity it only
    /// looks for an existing one.
    pub fn get_context<T: Component>(
        &self,
        current_widget: CurrentWidget,
    ) -> Option<Entity> {
        let type_name: String = std::any::type_name::<T>().into();
        self.traverse_find_context_entity(&type_name, current_widget)
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
            (Entity, &Parent, One<&dyn Widget>),
            (Changed<Parent>, Without<PreviousWidget>),
        >,
        mut removed: RemovedComponents<Parent>,
    ) {
        // Add any that were added or changed.
        for (entity, parent, _) in query.iter() {
            context_helper.parents.insert(entity, parent.get());
        }

        // Remove any that were removed.
        for entity in removed.read() {
            context_helper.parents.remove(&entity);
        }
    }

    pub fn get_previous_widget(
        &mut self,
        commands: &mut Commands,
        current_widget: CurrentWidget,
    ) -> Entity {
        let prev_state_entity = self
            .prev_state_entities
            .entry(*current_widget)
            .or_insert_with(|| commands.spawn(PreviousWidget).set_parent(*current_widget).id());

        *prev_state_entity
    }

    pub(crate) fn get_previous_widget_no_spawn(&self, current_widget: CurrentWidget) -> Option<&Entity> {
        self.prev_state_entities.get(&*current_widget)
    }
}

#[derive(Component)]
pub struct PreviousWidget;
