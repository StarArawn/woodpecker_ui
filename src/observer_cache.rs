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

    pub fn despawn_for_target(&mut self, world: &mut World, target: Entity) {
        for entities in self.observer_entities.values_mut() {
            let mut removed = vec![];
            for ((slot, e_target), ob_entity) in entities.iter() {
                if target == *e_target {
                    removed.push((*slot, *e_target));
                    if world.get_entity(*ob_entity).is_ok() {
                        world.despawn(*ob_entity);
                    }
                }
            }

            for removed in removed {
                entities.remove(&removed);
            }
        }
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
