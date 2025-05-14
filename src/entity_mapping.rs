use bevy::{
    platform::collections::{HashMap, HashSet},
    prelude::*,
};

use crate::{context::Widget, runner::get_all_children, ObserverCache, ParentWidget};

/// Maps parent widgets to child widgets.
/// Will overwrite old widgets with new entities if the type and or key has changed.
#[derive(Resource, Default)]
pub struct WidgetMapper {
    parent_entity_to_child: HashMap<ParentWidget, Vec<EntityMappping>>,
    new_this_tick: HashSet<Entity>,
}

/// The mapped entity with a key and entity id.
pub struct EntityMappping {
    /// A widget key normally just the widget's type name
    /// This can be user specified however.
    key: String,
    /// The widget entity.
    entity: Entity,
}

impl WidgetMapper {
    /// Create a new widget mapper.
    pub fn new() -> Self {
        Self {
            parent_entity_to_child: HashMap::default(),
            new_this_tick: HashSet::default(),
        }
    }

    fn add(
        &mut self,
        key: String,
        parent: ParentWidget,
        child_entity: Entity,
        child_position_index: usize,
    ) {
        let child_hashmap = if self.parent_entity_to_child.contains_key(&parent) {
            self.parent_entity_to_child.get_mut(&parent).unwrap()
        } else {
            self.parent_entity_to_child.insert(parent, Vec::default());
            self.parent_entity_to_child.get_mut(&parent).unwrap()
        };

        child_hashmap.insert(
            child_position_index,
            EntityMappping {
                key,
                entity: child_entity,
            },
        );
    }

    #[allow(unused)]
    fn get_key<T: Widget>(child_key: Option<String>) -> String {
        if let Some(child_key) = child_key {
            format!("{}-{}", T::get_name(), child_key)
        } else {
            T::get_name()
        }
    }

    pub(crate) fn get_all_children(&mut self, parent: Entity) -> Vec<Entity> {
        self.parent_entity_to_child
            .get(&ParentWidget(parent))
            .unwrap_or(&vec![])
            .iter()
            .map(|em| em.entity)
            .collect::<Vec<_>>()
    }

    pub(crate) fn clear_added_this_frame(&mut self) {
        self.new_this_tick.clear();
    }

    pub(crate) fn added_this_frame(&self, entity: Entity) -> bool {
        self.new_this_tick.contains(&entity)
    }

    pub(crate) fn get_or_insert_entity_world(
        &mut self,
        world: &mut World,
        observer_cache: &mut ObserverCache,
        widget_name: String,
        parent: ParentWidget,
        child_key: Option<String>,
        child_position_index: usize,
    ) -> Entity {
        let key = if let Some(child_key) = child_key.clone() {
            format!("{}-{}", widget_name, child_key)
        } else {
            widget_name
        };
        if let Some(child_vec) = self.parent_entity_to_child.get(&parent) {
            if let Some(mapping) = child_vec.get(child_position_index) {
                if key == mapping.key {
                    self.new_this_tick.insert(mapping.entity);
                    return mapping.entity;
                } else {
                    let entity_to_remove = mapping.entity;
                    // Remove observers
                    observer_cache.despawn_for_target(world, entity_to_remove);

                    // Remove from the mapper.
                    self.remove_by_entity_id(parent.entity(), entity_to_remove);

                    // Remove children of this child
                    for child in get_all_children(world, entity_to_remove) {
                        if world.get_entity(child).is_err() {
                            continue;
                        }
                        let parent = world.entity(child).get::<ChildOf>().expect("Unknown dangling child! This is an error with woodpecker UI source please file a bug report.").parent();
                        self.remove_by_entity_id(parent, child);
                        observer_cache.despawn_for_target(world, child);
                    }

                    // Remove
                    world.entity_mut(entity_to_remove).despawn();
                }
            }
        }

        let child_entity = world.spawn(ChildOf(*parent)).id();
        self.add(key, parent, child_entity, child_position_index);

        self.new_this_tick.insert(child_entity);

        child_entity
    }

    pub(crate) fn remove_by_entity_id(&mut self, parent: Entity, entity: Entity) {
        if let Some(children) = self.parent_entity_to_child.get_mut(&ParentWidget(parent)) {
            if let Some(index) = children.iter().position(|em| em.entity == entity) {
                children.remove(index);
            }
        }
    }
}
