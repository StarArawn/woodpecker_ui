use std::sync::{Arc, RwLock};

use bevy::{ecs::system::IntoObserverSystem, prelude::*};

use crate::{
    context::Widget, portal::Portal, prelude::WidgetMapper, CurrentWidget, ObserverCache,
    ParentWidget,
};

/// A component to pass children down the tree
/// while also having children of its own.
#[derive(Component, Default, Clone, Deref, DerefMut, PartialEq)]
pub struct PassedChildren(pub WidgetChildren);

/// A commponent to pass children down the tree
/// while also having children of its own.
#[derive(Component, Default, Clone)]
pub struct Mounted;

/// Snapshot of the last-diffed [`PassedChildren`] value for a widget entity.
///
/// `PassedChildren` holds `Arc<dyn Fn>` closures and can't derive `Reflect`, so it's invisible
/// to the generic [`crate::diffing::diff_widget_entity`] walk; this is a hand-written
/// equivalent just for this one type.
#[derive(Component, Default)]
pub(crate) struct PreviousPassedChildren(Option<PassedChildren>);

/// Compares `entity`'s current [`PassedChildren`] value (if it has one) against the last
/// snapshot taken of it, returning `true` (and updating the snapshot) if it changed. A
/// widget that doesn't use `PassedChildren` at all is simply never affected by this check.
pub(crate) fn diff_passed_children(world: &mut World, entity: Entity) -> bool {
    let Some(current) = world.get::<PassedChildren>(entity) else {
        return false;
    };
    let current = current.clone();

    let changed = match world.get::<PreviousPassedChildren>(entity) {
        Some(previous) => previous.0.as_ref() != Some(&current),
        None => true,
    };

    if changed {
        world
            .entity_mut(entity)
            .insert(PreviousPassedChildren(Some(current)));
    }

    changed
}

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
                    String,
                    ObserverList,
                    Option<String>, // Child key
                    Option<Portal>,
                ) + Sync
                + Send,
        >,
        ObserverList,
        Option<String>, // Child key
        Option<Portal>,
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
                    String,
                    ObserverList,
                    Option<String>, // Child key
                    Option<Portal>,
                ) + Sync
                + Send,
        >,
        ObserverList,
        Option<String>, // Child key
        Option<Portal>,
    )>,
    /// A collection of observers attached to the parent widget not the children
    self_observers: ObserverList,
    /// Stores a list of previous children.
    prev_children: Vec<(String, Option<String>)>,
    /// Lets the system know who the parent is.
    /// We need this because childen can be passed around until they
    /// are committed to a parent.
    parent_widget: Option<ParentWidget>,
}

impl std::fmt::Debug for WidgetChildren {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WidgetChildren").finish()
    }
}

impl PartialEq for WidgetChildren {
    fn eq(&self, other: &Self) -> bool {
        let queue = self
            .children_queue
            .iter()
            .map(|(wn, _, _, child_key, portal)| (wn.clone(), child_key.clone(), *portal))
            .collect::<Vec<_>>();
        let other_queue = other
            .children_queue
            .iter()
            .map(|(wn, _, _, child_key, portal)| (wn.clone(), child_key.clone(), *portal))
            .collect::<Vec<_>>();
        let children = self
            .children
            .iter()
            .map(|(wn, _, _, child_key, portal)| (wn.clone(), child_key.clone(), *portal))
            .collect::<Vec<_>>();
        let other_children = other
            .children
            .iter()
            .map(|(wn, _, _, child_key, portal)| (wn.clone(), child_key.clone(), *portal))
            .collect::<Vec<_>>();
        queue == other_queue && children == other_children
    }
}

impl WidgetChildren {
    /// Builder pattern for adding children when you initially create the component.
    pub fn with_child<T: Widget + Component + Default>(
        mut self,
        bundle: impl Bundle + Clone,
    ) -> Self {
        self.add::<T>(bundle);
        self
    }

    /// Adds a key to the last child widget entity added
    pub fn with_key(mut self, key: impl Into<String>) -> Self {
        self.add_key(key);
        self
    }

    /// Adds a key to the last child widget entity added
    pub fn add_key(&mut self, key: impl Into<String>) {
        if let Some((_, _, _, child_key, _)) = self.children_queue.last_mut() {
            *child_key = Some(key.into());
        }
    }

    /// Builder pattern equivalent of [`Self::portal`] -- see its doc comment.
    pub fn with_portal(mut self) -> Self {
        self.portal();
        self
    }

    /// Marks the last child widget added as portaled: instead of being physically parented
    /// (`ChildOf`) to the widget declaring it, it's parented to the crate's shared
    /// [`crate::portal::OverlayRoot`] -- e.g. so a `Modal` deeply nested in the declared tree
    /// still renders/picks as if it were a root-level sibling. The widget's *logical* position
    /// (context/theme inheritance, focus-trap scoping) is unaffected -- see
    /// [`crate::portal::LogicalParent`]. Only takes effect the first time this keyed entity is
    /// spawned; call right after `.add::<T>()`, the same convention as `.add_key(...)`.
    pub fn portal(&mut self) -> &mut Self {
        self.portal_to_inner(None);
        self
    }

    /// Builder pattern equivalent of [`Self::portal_to`] -- see its doc comment.
    pub fn with_portal_to(mut self, target: Entity) -> Self {
        self.portal_to(target);
        self
    }

    /// Same as [`Self::portal`], but targets `target` directly instead of defaulting to
    /// [`crate::portal::OverlayRoot`] -- e.g. portaling into a specific nested overlay rather
    /// than the app-wide root.
    pub fn portal_to(&mut self, target: Entity) -> &mut Self {
        self.portal_to_inner(Some(target));
        self
    }

    fn portal_to_inner(&mut self, target: Option<Entity>) {
        if let Some((_, _, _, _, portal)) = self.children_queue.last_mut() {
            *portal = Some(Portal(target));
        }
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
    pub fn add<T: Widget + Component + Default>(
        &mut self,
        bundle: impl Bundle + Clone,
    ) -> &mut Self {
        let widget_type = T::get_name();
        self.children_queue.push((
            widget_type,
            Arc::new(
                move |world: &mut World,
                      widget_mapper: &mut WidgetMapper,
                      observer_cache: &mut ObserverCache,
                      parent: ParentWidget,
                      widget_type: String,
                      observer_list: ObserverList,
                      child_key: Option<String>,
                      portal: Option<Portal>| {
                    let type_name_without_path =
                        widget_type.clone().split("::").last().unwrap().to_string();
                    let child_widget = widget_mapper.get_or_insert_entity_world(
                        world,
                        widget_type,
                        parent,
                        child_key,
                        portal,
                    );
                    world
                        .entity_mut(child_widget)
                        .insert(T::default())
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
            None,
            None,
        ));

        self
    }

    /// Builds a single `(spawn_location, spawner)` `ObserverList` entry shared by
    /// [`Self::observe`] and [`Self::self_observe`].
    fn build_observer_entry<E: Event, B: Bundle, M>(
        spawn_location: CurrentWidget,
        observer: impl IntoObserverSystem<E, B, M>,
    ) -> (
        CurrentWidget,
        Arc<dyn Fn(&mut World, Entity, Entity) -> Option<Entity> + Sync + Send>,
    ) {
        let o = Arc::new(RwLock::new(Some(Observer::new(observer))));
        (
            spawn_location,
            Arc::new(move |world, _parent, target_entity| {
                // Last we attempt to spawn the observer.
                // We need to do this funkyness to get around observer not being cloneable.
                // Instead we can just reuse it!
                if let Some(ob) = o.write().unwrap().take() {
                    trace!("Adding new observer for {}", target_entity);
                    let observer_entity = world
                        .spawn((ob.with_entity(target_entity), ChildOf(target_entity)))
                        .id();
                    Some(observer_entity)
                } else {
                    None
                }
            }),
        )
    }

    /// Add a bevy observer system to the last added widget if no widget was added the observer
    /// is added to the widget entity who has ownership of the `WidgetChildren` component.
    ///
    /// Note this "no widget was added yet" check only looks at whether `add`/`with_child` has
    /// been called on *this* `WidgetChildren` value so far -- a widget whose children arrive
    /// pre-populated via its own spawn bundle (e.g. `WButton`, whose `render` never calls
    /// `add` itself) will see a non-empty queue here even though it wants a self-observer; use
    /// [`Self::self_observe`] instead in that case.
    /// - spawn_location: Widget entity where the observer was created.
    pub fn observe<E: Event, B: Bundle, M>(
        &mut self,
        spawn_location: CurrentWidget,
        observer: impl IntoObserverSystem<E, B, M>,
    ) -> &mut Self {
        let entry = Self::build_observer_entry(spawn_location, observer);
        if let Some((_, _, observers, _, _)) = self.children_queue.last_mut() {
            observers.push(entry);
        } else {
            // Treat like parent observer
            self.self_observers.push(entry);
        }

        self
    }

    /// Adds a bevy observer system to the widget entity that owns this `WidgetChildren`,
    /// unconditionally -- unlike [`Self::observe`], this never attaches to a queued child
    /// instead, regardless of whether `add`/`with_child` has already been called this frame.
    ///
    /// Needed by any widget whose `render` wants a self-observer (e.g. `hover_state`/
    /// `hover_cursor` in `cursor.rs`, used for hover-driven style/cursor changes) but never
    /// calls `add` itself -- its own children arrive pre-populated via the caller's spawn
    /// bundle, so `children_queue` is never actually empty by the time `render` runs, and
    /// `observe` would otherwise silently misattach to whatever child happens to be queued.
    /// - spawn_location: Widget entity where the observer was created.
    pub fn self_observe<E: Event, B: Bundle, M>(
        &mut self,
        spawn_location: CurrentWidget,
        observer: impl IntoObserverSystem<E, B, M>,
    ) -> &mut Self {
        let entry = Self::build_observer_entry(spawn_location, observer);
        self.self_observers.push(entry);
        self
    }

    /// Lets you know if the children have changed between now and when they were last rendered.
    pub fn children_changed(&self) -> bool {
        self.children
            .iter()
            .map(|(n, _, _, child_key, _)| (n, child_key))
            .collect::<Vec<_>>()
            != self
                .prev_children
                .iter()
                .map(|t| (&t.0, &t.1))
                .collect::<Vec<_>>()
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

        // Process self observers
        world.resource_scope(
            |world: &mut World, mut observer_cache: Mut<ObserverCache>| {
            for (id, (spawn_entity, ob)) in self.self_observers.iter().enumerate() {
                if !observer_cache.contains(spawn_entity.0, id, parent_widget.entity()) {
                    if let Some(observer_entity) = (ob)(world, **spawn_entity, parent_widget.entity()) {
                        observer_cache.add(spawn_entity.0, id, parent_widget.entity(), observer_entity);
                    } else {
                        panic!("Attempted to add an observer when its already been used. This is considered a bug please open a ticket.");
                    }
                }
            }
        });

        world.resource_scope(|world: &mut World, mut widget_mapper: Mut<WidgetMapper>| {
            world.resource_scope(
                |world: &mut World, mut observer_cache: Mut<ObserverCache>| {
                    // Keyed reconciliation: entities survive reorders, only keys that
                    // disappear get despawned. Started unconditionally so N -> 0 still cleans up.
                    widget_mapper.start_reconciliation(parent_widget);

                    // Loop through each child and spawn the bundles.
                    for (widget_type, child, observers, child_key, portal) in self.children.iter() {
                        trace!("Adding as child: {}", widget_type);
                        child(
                            world,
                            &mut widget_mapper,
                            &mut observer_cache,
                            parent_widget,
                            widget_type.clone(),
                            observers.clone(),
                            child_key.clone(),
                            *portal,
                        );
                    }

                    widget_mapper.finish_reconciliation(world, &mut observer_cache, parent_widget);
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
            .map(|(n, _, _, child_key, _)| (n.clone(), child_key.clone()))
            .collect::<Vec<_>>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Component, Default, Clone)]
    struct TestWidget;
    impl Widget for TestWidget {}

    #[derive(Event, Clone)]
    struct TestClick;

    /// Finds the entity (if any) that is both a child of `target` and has an `Observer`
    /// component -- i.e. an observer spawned to watch `target`.
    fn find_observer_entity(world: &World, target: Entity) -> Option<Entity> {
        world
            .entity(target)
            .get::<Children>()?
            .iter()
            .find(|e| world.get_entity(*e).is_ok_and(|e| e.contains::<Observer>()))
    }

    fn render_one_child_with_observer(world: &mut World, parent: ParentWidget) {
        let mut children = WidgetChildren::default();
        children.add::<TestWidget>(TestWidget);
        children.observe(parent.as_current(), |_trigger: On<TestClick>| {});
        children.apply(parent);
        children.process_world(world);
    }

    /// A child re-declared identically across two renders must keep the same entity
    /// identity and must not have its observer despawned and recreated.
    #[test]
    fn observer_survives_unchanged_re_render() {
        let mut world = World::new();
        world.insert_resource(WidgetMapper::new());
        world.insert_resource(ObserverCache::default());

        let parent_entity = world.spawn(WidgetChildren::default()).id();
        let parent = ParentWidget(parent_entity);

        render_one_child_with_observer(&mut world, parent);
        let child_entity = world
            .entity(parent_entity)
            .get::<Children>()
            .expect("parent should have a child after first render")
            .iter()
            .next()
            .expect("parent should have exactly one child");
        let observer_entity_1 = find_observer_entity(&world, child_entity)
            .expect("child should have an observer after first render");

        // Second render: same declared child, same observer, nothing conceptually changed.
        render_one_child_with_observer(&mut world, parent);
        let child_entity_2 = world
            .entity(parent_entity)
            .get::<Children>()
            .expect("parent should have a child after second render")
            .iter()
            .next()
            .expect("parent should have exactly one child");
        assert_eq!(child_entity, child_entity_2, "child entity must be reused");

        let observer_entity_2 = find_observer_entity(&world, child_entity_2)
            .expect("child should still have an observer after second render");
        assert_eq!(
            observer_entity_1, observer_entity_2,
            "observer entity must survive an unchanged re-render, not be despawned and recreated"
        );
    }

    /// `clear()` empties `children` without touching `prev_children`, which is the one way
    /// they can diverge -- callers use it to force `children_changed()` to report true.
    #[test]
    fn clear_causes_children_changed_to_report_true() {
        let mut world = World::new();
        world.insert_resource(WidgetMapper::new());
        world.insert_resource(ObserverCache::default());

        let parent_entity = world.spawn(WidgetChildren::default()).id();
        let parent = ParentWidget(parent_entity);

        let mut children = WidgetChildren::default();
        children.add::<TestWidget>(TestWidget);
        children.apply(parent);
        children.process_world(&mut world);
        assert!(
            !children.children_changed(),
            "immediately after processing, children and prev_children must match"
        );

        children.clear();
        assert!(
            children.children_changed(),
            "clear() must cause children_changed() to report true until the next process_world"
        );
    }

    /// End-to-end check of the `.portal()` builder API (as opposed to `entity_mapping.rs`'s
    /// lower-level `get_or_insert_entity_world` tests): a child added via `.add::<T>()` then
    /// marked `.portal()` must end up physically parented to `OverlayRoot`, excluded from the
    /// declaring parent's own `Children`, and must keep its entity identity (not despawn and
    /// respawn) across a second, otherwise-identical render pass.
    #[test]
    fn portal_builder_api_parents_to_overlay_root_and_survives_a_second_render() {
        use crate::portal::OverlayRoot;

        let mut world = World::new();
        world.insert_resource(WidgetMapper::new());
        world.insert_resource(ObserverCache::default());
        let overlay_root = world.spawn_empty().id();
        world.insert_resource(OverlayRoot(overlay_root));

        let parent_entity = world.spawn(WidgetChildren::default()).id();
        let parent = ParentWidget(parent_entity);

        let render = |world: &mut World| {
            let mut children = WidgetChildren::default();
            children.add::<TestWidget>(TestWidget);
            children.portal();
            children.add_key("portaled");
            children.apply(parent);
            children.process_world(world);
        };

        render(&mut world);
        let portaled_entity = world
            .entity(overlay_root)
            .get::<Children>()
            .expect("OverlayRoot should have gained the portaled entity as a real child")
            .iter()
            .next()
            .expect("OverlayRoot should have exactly one child");
        assert!(
            world.entity(parent_entity).get::<Children>().is_none(),
            "the declaring parent must have no Children of its own -- its only declared \
             child was portaled away"
        );

        render(&mut world);
        let portaled_entity_2 = world
            .entity(overlay_root)
            .get::<Children>()
            .unwrap()
            .iter()
            .next()
            .unwrap();
        assert_eq!(
            portaled_entity, portaled_entity_2,
            "the portaled entity's identity must survive an unchanged second render, not be \
             despawned and respawned"
        );
    }

    /// Finds every live `Observer` entity in `world` and returns the set of entities any of
    /// them watches. `Observer` targeting (`.with_entity(...)`/`watch_entity`) is independent
    /// of `ChildOf` -- an observer fires based on its own watched-entities list regardless of
    /// its hierarchy position -- so this is the correct way to check *what* an observer
    /// watches, unlike walking `Children` (which `WidgetMapper::finish_reconciliation`'s
    /// `replace_related::<ChildOf>` can sever after the observer was spawned, a separate,
    /// pre-existing characteristic of self-observers in this crate that predates
    /// `self_observe` -- the observer keeps firing correctly either way).
    fn all_watched_entities(world: &World) -> std::collections::HashSet<Entity> {
        let mut watched = std::collections::HashSet::new();
        for entity in world.iter_entities() {
            if let Some(observer) = world.get::<Observer>(entity.id()) {
                watched.extend(observer.descriptor().entities().iter().copied());
            }
        }
        watched
    }

    /// Regression test for the `WButton`-class bug: a widget whose children arrive
    /// pre-populated via the caller's spawn bundle (so `children_queue` is *never* empty by
    /// the time that widget's own `render` runs) still needs a real self-observer for hover
    /// tracking. `observe()`'s "attach to the last queued child, or self if the queue is
    /// empty" heuristic silently misattaches to that pre-populated child in this case --
    /// `self_observe()` must attach to the owning widget regardless of queue state.
    #[test]
    fn self_observe_targets_the_owning_widget_even_with_a_nonempty_queue() {
        let mut world = World::new();
        world.insert_resource(WidgetMapper::new());
        world.insert_resource(ObserverCache::default());

        let parent_entity = world.spawn(WidgetChildren::default()).id();
        let parent = ParentWidget(parent_entity);

        let mut children = WidgetChildren::default();
        // Simulates a caller-supplied spawn bundle populating this widget's own children
        // *before* its own render ever runs -- exactly `WButton`'s situation.
        children.add::<TestWidget>(TestWidget);
        assert!(
            !children.children_queue.is_empty(),
            "sanity check: the queue must be non-empty for this test to actually exercise the \
             bug this regression test targets"
        );
        // What `WButton::render()` (etc.) does: call `self_observe` with no preceding `.add()`
        // of its *own*, wanting to observe itself, not the pre-populated child above.
        children.self_observe(parent.as_current(), |_trigger: On<TestClick>| {});
        children.apply(parent);
        children.process_world(&mut world);

        let widget_child = world
            .entity(parent_entity)
            .get::<Children>()
            .expect("parent should have the one queued TestWidget child")
            .iter()
            .next()
            .expect("parent should have exactly one non-observer child");

        let watched = all_watched_entities(&world);
        assert!(
            watched.contains(&parent_entity),
            "the self-observer must watch the widget entity itself"
        );
        assert!(
            !watched.contains(&widget_child),
            "the self-observer must NOT watch the pre-populated queued child"
        );
    }

    /// Companion to the above: `observe()`'s existing "attach to the last queued child"
    /// behavior must be unchanged by the `self_observe()` addition -- this is the correct,
    /// intentional behavior for widgets that observe a child *they* just added (e.g. a list
    /// row's own click handler), not a bug.
    #[test]
    fn observe_still_targets_the_last_queued_child_when_the_queue_is_nonempty() {
        let mut world = World::new();
        world.insert_resource(WidgetMapper::new());
        world.insert_resource(ObserverCache::default());

        let parent_entity = world.spawn(WidgetChildren::default()).id();
        let parent = ParentWidget(parent_entity);

        let mut children = WidgetChildren::default();
        children.add::<TestWidget>(TestWidget);
        children.observe(parent.as_current(), |_trigger: On<TestClick>| {});
        children.apply(parent);
        children.process_world(&mut world);

        let widget_child = world
            .entity(parent_entity)
            .get::<Children>()
            .expect("parent should have the one queued TestWidget child")
            .iter()
            .next()
            .expect("parent should have exactly one non-observer child");

        let watched = all_watched_entities(&world);
        assert!(
            watched.contains(&widget_child),
            "observe() must still attach to the last queued child when the queue is non-empty"
        );
        assert!(
            !watched.contains(&parent_entity),
            "observe() must not also attach a self-observer in this case"
        );
    }

    /// Regression test for `WidgetMapper::finish_reconciliation`'s `replace_related::<ChildOf>`
    /// severing a self-observer's `ChildOf` the moment its owning widget reconciles its
    /// *declared* children -- a self-observer is spawned as `ChildOf(parent)` outside the
    /// `get_or_insert_entity_world`/`pending` bookkeeping entirely, so without preserving it
    /// explicitly, `replace_related` (which sets `parent`'s full `Children` list to exactly
    /// its declared children) would silently detach it. A detached observer still fires
    /// correctly (targeting doesn't depend on `ChildOf`), but never gets cascade-despawned
    /// when `parent` itself is later removed -- a real, if narrow, entity leak. This test
    /// exercises the exact WButton-shaped sequence: a self-observer *and* a declared child
    /// coexist, across two render passes, so the self-observer's `ChildOf` must survive being
    /// there when `finish_reconciliation` also reconciles the declared child.
    #[test]
    fn self_observer_child_of_survives_reconciling_declared_children_across_two_renders() {
        let mut world = World::new();
        world.insert_resource(WidgetMapper::new());
        world.insert_resource(ObserverCache::default());

        let parent_entity = world.spawn(WidgetChildren::default()).id();
        let parent = ParentWidget(parent_entity);

        let render = |world: &mut World| {
            let mut children = WidgetChildren::default();
            children.self_observe(parent.as_current(), |_trigger: On<TestClick>| {});
            children.add::<TestWidget>(TestWidget);
            children.add_key("label");
            children.apply(parent);
            children.process_world(world);
        };

        render(&mut world);
        render(&mut world);

        let self_observer_child_of_parent = world
            .entity(parent_entity)
            .get::<Children>()
            .expect("parent should have children after two renders")
            .iter()
            .any(|child| {
                world
                    .get_entity(child)
                    .is_ok_and(|e| e.contains::<Observer>())
            });
        assert!(
            self_observer_child_of_parent,
            "the self-observer's ChildOf(parent) must survive a second render that also \
             reconciles a declared child -- finish_reconciliation's replace_related must not \
             sever it"
        );

        let watched = all_watched_entities(&world);
        assert!(
            watched.contains(&parent_entity),
            "the self-observer must still watch the widget entity after two renders"
        );
    }
}
