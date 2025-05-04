use std::sync::{Arc, RwLock};

use bevy::{ecs::system::IntoObserverSystem, prelude::*};

use crate::{context::Widget, prelude::WidgetMapper, CurrentWidget, ObserverCache, ParentWidget};

/// A component to pass children down the tree
/// while also having children of its own.
#[derive(Component, Default, Clone, Deref, DerefMut, PartialEq)]
pub struct PassedChildren(pub WidgetChildren);

/// A commponent to pass children down the tree
/// while also having children of its own.
#[derive(Component, Default, Clone)]
pub struct Mounted;

type ObserverList = Vec<(
    CurrentWidget,
    Arc<dyn Fn(&mut World, Entity, Entity) -> Option<Entity> + Sync + Send>,
)>;

/// A bevy component that keeps track of Woodpecker UI widget children.
///
/// This is very similar to bevy commands as in it lets you spawn bundles
/// but it does not create an entity until
/// WidgetChildren::process_world is called.
#[derive(Component, Default, Clone)]
pub struct WidgetChildren {
    // Strings here are widget type names.
    // First children are stored in a queue.
    children_queue: Vec<(
        String,
        Arc<
            dyn Fn(
                    &mut World,
                    &mut WidgetMapper,
                    &mut ObserverCache,
                    ParentWidget,
                    usize,
                    String,
                    ObserverList,
                ) + Sync
                + Send,
        >,
        ObserverList,
    )>,
    // When a widget is processed onto a parent they get stored here and removed from the queue.
    children: Vec<(
        String,
        Arc<
            dyn Fn(
                    &mut World,
                    &mut WidgetMapper,
                    &mut ObserverCache,
                    ParentWidget,
                    usize,
                    String,
                    ObserverList,
                ) + Sync
                + Send,
        >,
        ObserverList,
    )>,
    /// Stores a list of previous children.
    prev_children: Vec<String>,
    /// Lets the system know who the parent is.
    /// We need this because childen can be passed around until they
    /// are committed to a parent.
    parent_widget: Option<ParentWidget>,
}

impl PartialEq for WidgetChildren {
    fn eq(&self, other: &Self) -> bool {
        let queue = self
            .children_queue
            .iter()
            .map(|(wn, _, _)| wn.clone())
            .collect::<Vec<_>>();
        let other_queue = other
            .children_queue
            .iter()
            .map(|(wn, _, _)| wn.clone())
            .collect::<Vec<_>>();
        let children: Vec<String> = self
            .children
            .iter()
            .map(|(wn, _, _)| wn.clone())
            .collect::<Vec<_>>();
        let other_children = other
            .children
            .iter()
            .map(|(wn, _, _)| wn.clone())
            .collect::<Vec<_>>();
        queue == other_queue && children == other_children
    }
}

impl WidgetChildren {
    /// Builder pattern for adding children when you initially create the component.
    pub fn with_child<T: Widget>(mut self, bundle: impl Bundle + Clone) -> Self {
        self.add::<T>(bundle);
        self
    }

    /// Builder pattern for adding observers when you initially create a child.
    /// - spawn_location: Widget entity where the observer was created.
    pub fn with_observe<E: Event, B: Bundle, M>(
        mut self,
        spawn_location: CurrentWidget,
        observer: impl IntoObserverSystem<E, B, M>,
    ) -> Self {
        self.observe(spawn_location, observer);
        self
    }

    /// Clears out all children.
    pub fn clear(&mut self) {
        self.children.clear();
        self.parent_widget = None;
    }

    /// Add a new widget to the list of children. The widget should be a bevy bundle and T should implement widget.
    ///
    /// Note: Make sure to call [`WidgetChildren::apply`] in the render system of the parent
    /// otherwise the entities will not be spawned! This will NOT spawn the bundles.
    pub fn add<T: Widget>(&mut self, bundle: impl Bundle + Clone) -> &mut Self {
        let widget_type = T::get_name();
        self.children_queue.push((
            widget_type,
            Arc::new(
                move |world: &mut World,
                      widget_mapper: &mut WidgetMapper,
                      observer_cache: &mut ObserverCache,
                      parent: ParentWidget,
                      index: usize,
                      widget_type: String,
                      observer_list: ObserverList| {
                    let type_name_without_path =
                        widget_type.clone().split("::").last().unwrap().to_string();
                    let child_widget = widget_mapper.get_or_insert_entity_world(
                        world,
                        widget_type,
                        parent,
                        None,
                        index,
                    );
                    world
                        .entity_mut(child_widget)
                        .insert(bundle.clone())
                        .insert(Mounted)
                        .insert(Name::new(type_name_without_path.clone()));
                    for (id, (spawn_entity, ob)) in observer_list.iter().enumerate() {
                        if !observer_cache.contains(spawn_entity.0, id, child_widget) {
                            if let Some(observer_entity) = (ob)(world, **spawn_entity, child_widget)
                            {
                                observer_cache.add(
                                    spawn_entity.0,
                                    id,
                                    child_widget,
                                    observer_entity,
                                );
                            } else {
                                panic!("Attempted to add an observer when its already been used. This is considered a bug please open a ticket.");
                            }
                        }
                    }
                },
            ),
            vec![],
        ));

        self
    }

    /// Add a bevy observer system to the last added entity.
    /// - spawn_location: Widget entity where the observer was created.
    pub fn observe<E: Event, B: Bundle, M>(
        &mut self,
        spawn_location: CurrentWidget,
        observer: impl IntoObserverSystem<E, B, M>,
    ) -> &mut Self {
        if let Some((_, _, observers)) = self.children_queue.last_mut() {
            let o = Arc::new(RwLock::new(Some(Observer::new(observer))));
            observers.push((
                spawn_location,
                Arc::new(move |world, parent, target_entity| {
                    // Last we attempt to spawn the observer.
                    // We need to do this funkyness to get around observer not being cloneable.
                    // Instead we can just reuse it!
                    if let Some(ob) = o.write().unwrap().take() {
                        trace!("Adding new observer for {}", target_entity);
                        let observer_entity = world
                            .spawn((ob.with_entity(target_entity), ChildOf(parent)))
                            .id();
                        return Some(observer_entity);
                    } else {
                        return None;
                    }
                }),
            ));
        } else {
            warn!("Can't attach observer!");
        }

        self
    }

    /// Lets you know if the children have changed between now and when they were last rendered.
    pub fn children_changed(&self) -> bool {
        self.children.iter().map(|(n, _, _)| n).collect::<Vec<_>>()
            != self.prev_children.iter().collect::<Vec<_>>()
    }

    /// Attaches the children to a parent widget.
    /// Note: This doesn't actually spawn the children
    /// that occurs when the parent widget finishes rendering.
    pub fn apply(&mut self, parent_widget: ParentWidget) {
        self.parent_widget = Some(parent_widget);
    }

    pub(crate) fn process_world(&mut self, world: &mut World) {
        let Some(parent_widget) = self.parent_widget else {
            return;
        };

        // If our queue isn't empty drain it into the children
        // This ensures we remove previous children.
        if !self.children_queue.is_empty() {
            self.children = self.children_queue.drain(..).collect::<Vec<_>>();
        }

        world.resource_scope(|world: &mut World, mut widget_mapper: Mut<WidgetMapper>| {
            world.resource_scope(
                |world: &mut World, mut observer_cache: Mut<ObserverCache>| {
                    // Loop through each child and spawn the bundles.
                    // The widget mapper helps keep track of which entities go with which child.
                    // They are ensured to have the same entity id for a given child index and
                    // widget type name. The type name is passed in here from the children vec.
                    for (i, (widget_type, child, observers)) in self.children.iter().enumerate() {
                        trace!("Adding as child: {}", widget_type);
                        child(
                            world,
                            &mut widget_mapper,
                            &mut observer_cache,
                            parent_widget,
                            i,
                            widget_type.clone(),
                            observers.clone(),
                        );
                    }
                },
            );
        });

        // Remove the parent widget.
        // [`Self::apply`] should be called if this parent re-renders.
        // If its not then we assume no children.
        self.parent_widget = None;

        // Throw the children type names into the previous children list.
        self.prev_children = self
            .children
            .iter()
            .map(|(n, _, _)| n.clone())
            .collect::<Vec<_>>()
    }
}
