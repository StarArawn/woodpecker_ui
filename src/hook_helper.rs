use bevy::{prelude::*, utils::HashMap};
use bevy_trait_query::One;

use crate::{context::Widget, CurrentWidget};

/// A helper resource that keeps track of context(hierachy state) entities and state tied to entities.
#[derive(Resource, Default, Debug, Clone)]
pub struct HookHelper {
    internal_context: HashMap<Entity, HashMap<String, Entity>>,
    parents: HashMap<Entity, Entity>,
    state: HashMap<Entity, HashMap<String, Entity>>,
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

    /// Looks up the T state for an entity.
    /// State entities are just entities parented to the entity passed in.
    pub fn get_state<T: Component>(&self, current_widget: CurrentWidget) -> Option<Entity> {
        let type_name: String = std::any::type_name::<T>().into();
        self.state
            .get(&*current_widget)
            .map(|context_types| context_types.get(&type_name).map(|e| *e))
            .flatten()
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
        query: Query<(Entity, &Parent, One<&dyn Widget>), Changed<Parent>>,
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
}
