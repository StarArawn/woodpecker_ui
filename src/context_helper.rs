use bevy::{prelude::*, utils::HashMap};
use bevy_trait_query::One;

use crate::{context::Widget, CurrentWidget};

#[derive(Resource, Default, Debug, Clone)]
pub struct ContextHelper {
    internal_context: HashMap<Entity, HashMap<String, Entity>>,
    parents: HashMap<Entity, Entity>,
}

impl ContextHelper {
    /// Traverses the widget tree(bevy hierarchy) and finds the context entity
    /// associated with the given T type.
    pub fn use_context<T: Component>(
        &mut self,
        commands: &mut Commands,
        current_entity: CurrentWidget,
    ) -> Entity {
        let type_name: String = std::any::type_name::<T>().into();
        if let Some(context_entity) = self.traverse_find_context_entity(&type_name, current_entity)
        {
            context_entity
        } else {
            let context_entity = commands.spawn_empty().id();

            let context_types = self
                .internal_context
                .entry(*current_entity)
                .or_insert(HashMap::default());

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
            .map(|context_types| context_types.get(type_name))
            .flatten()
        {
            return Some(*context_entity);
        }

        // Walk up tree if nothing was found above.
        if let Some(parent) = self.parents.get(&*current_entity) {
            return self.traverse_find_context_entity(&type_name, CurrentWidget(*parent));
        }

        None
    }

    pub(crate) fn update_context_helper(
        mut context_helper: ResMut<ContextHelper>,
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

mod tests {
    #[test]
    fn test_context() {
        use crate::context_helper::ContextHelper;
        use bevy::ecs::world::CommandQueue;
        use bevy::prelude::*;

        #[derive(Component)]
        struct MyContext;

        // Setup
        let mut world = World::default();
        let mut context_helper = ContextHelper::default();

        // Entities
        let parent = world.spawn_empty().id();
        let child = world.spawn_empty().set_parent(parent).id();

        // Simulate..
        context_helper.parents.insert(child, parent);

        // Test
        let mut command_queue = CommandQueue::default();
        let mut commands = Commands::new(&mut command_queue, &world);
        let parent_context_entity =
            context_helper.use_context::<MyContext>(&mut commands, crate::CurrentWidget(parent));

        let child_context_entity =
            context_helper.use_context::<MyContext>(&mut commands, crate::CurrentWidget(child));

        assert!(parent_context_entity == child_context_entity);
    }
}
