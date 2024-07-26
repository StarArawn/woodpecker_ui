use bevy::{
    ecs::query::{QueryData, WorldQuery},
    prelude::*,
    utils::HashMap,
};
use bevy_trait_query::One;

use crate::{context::Widget, CurrentWidget};

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

    /// Looks up the T state for an entity.
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

    pub fn compare<Q: QueryData + WidgetCompareTrait, T: WidgetCompareTrait + PartialEq>(
        &mut self,
        current_widget: CurrentWidget,
        commands: &mut Commands,
        query1: &Query<Q, Without<PreviousWidget>>,
        query2: &Query<Q, With<PreviousWidget>>,
    ) -> bool
    where
        // for <'a> <Q::ReadOnly as WorldQuery>::Item<'a>: PartialEq + WidgetCompareTrait,
        Q: for<'a> bevy::ecs::query::QueryData<ReadOnly: WorldQuery<Item<'a> = &'a T>>,
    {
        let prev_state_entity = self
            .prev_state_entities
            .entry(*current_widget)
            .or_insert_with(|| commands.spawn(PreviousWidget).id());
        let should_update = {
            if let Ok(item1) = query1.get(*current_widget) {
                // Replace previous entity state with new state.
                item1.insert_components(commands, *prev_state_entity);
                if let Ok(item2) = query2.get(*current_widget) {
                    item1 != item2
                } else {
                    false
                }
            } else {
                false
            }
        };
        should_update
    }
}

#[derive(Component)]
pub struct PreviousWidget;

trait WidgetCompareTrait: Clone {
    fn insert_components(&self, commands: &mut Commands, entity: Entity);
}

macro_rules! impl_tuple_query_data {
    ($(($name: ident, $state: ident)),*) => {
        #[allow(non_snake_case)]
        #[allow(clippy::unused_unit)]
        impl<$($name: Component + PartialEq + Clone),*> WidgetCompareTrait for (&$($name,)*) { 
            fn insert_components(&self, commands: &mut Commands, entity: Entity) {
                let ($($name,)*) = self.clone();
                commands.entity(entity)
                $(
                    .insert($name.clone())
                )*;
            }
        }

    };
}

bevy::utils::all_tuples!(impl_tuple_query_data, 1, 15, F, S);
