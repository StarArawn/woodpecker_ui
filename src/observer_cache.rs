use bevy::{platform::collections::HashMap, prelude::*};

#[derive(Resource, Default)]
pub(crate) struct ObserverCache {
    // Widget entity that spawned entity > (id, target_entity) > Observer entity
    observer_entities: HashMap<Entity, HashMap<(usize, Entity), Entity>>,
}

impl ObserverCache {
    pub fn add(
        &mut self,
        widget_entity: Entity,
        slot: usize,
        target_entity: Entity,
        observer_entity: Entity,
    ) {
        let entities = self.observer_entities.entry(widget_entity).or_default();
        entities.insert((slot, target_entity), observer_entity);
    }

    pub fn contains(&self, widget_entity: Entity, slot: usize, target_entity: Entity) -> bool {
        let Some(entities) = self.observer_entities.get(&widget_entity) else {
            return false;
        };

        entities.contains_key(&(slot, target_entity))
    }

    pub fn despawn_for_widget(&mut self, world: &mut World, widget_entity: Entity) {
        let Some(entities) = self.observer_entities.get(&widget_entity) else {
            return;
        };

        for (_target, ob_entity) in entities.iter() {
            world.despawn(*ob_entity);
        }

        self.observer_entities.remove(&widget_entity);
    }
}
