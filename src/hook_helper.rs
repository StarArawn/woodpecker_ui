use std::time::Duration;

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

    /// Reports whether `dep` differs from the last time `use_effect` was called for this
    /// widget (`true` on the very first call too, since there's nothing to compare against
    /// yet). A render can be triggered by any number of unrelated prop/context/state changes;
    /// this lets it run a side effect only when *this specific* dependency was part of the
    /// reason, mirroring a dependency-array `useEffect`. The caller needs a
    /// `Query<&EffectDep<T>>` alongside their own state query, the same way `use_state` needs
    /// a `Query<&MyState>` -- `HookHelper` alone has no way to read a component's value back.
    pub fn use_effect<T: Clone + PartialEq + Send + Sync + 'static>(
        &mut self,
        commands: &mut Commands,
        current_widget: CurrentWidget,
        query: &Query<&EffectDep<T>>,
        dep: T,
    ) -> bool {
        let entity = self.use_state(commands, current_widget, EffectDep(dep.clone()));
        match query.get(entity) {
            Ok(EffectDep(previous)) if *previous == dep => false,
            _ => {
                commands.entity(entity).insert(EffectDep(dep));
                true
            }
        }
    }

    /// Returns the value `current` held the *last* time this was called for this widget --
    /// `None` on the first call, when there's nothing to compare against yet. Useful for
    /// detecting a transition (e.g. "just became visible") rather than only the current value.
    pub fn use_previous<T: Clone + Send + Sync + 'static>(
        &mut self,
        commands: &mut Commands,
        current_widget: CurrentWidget,
        query: &Query<&Previous<T>>,
        current: T,
    ) -> Option<T> {
        let entity = self.use_state(commands, current_widget, Previous(current.clone()));
        let previous = query.get(entity).ok().map(|p| p.0.clone());
        commands.entity(entity).insert(Previous(current));
        previous
    }

    /// Returns a cached value, recomputing via `compute` only when `dep` differs from the last
    /// time this was called for this widget (or on the very first call) -- for a derived value
    /// that's expensive to build and shouldn't be rebuilt on every render, only when the input
    /// it's actually derived from changes. The caller needs a `Query<&Memo<D, T>>` alongside
    /// their own state query.
    pub fn use_memo<D: Clone + PartialEq + Send + Sync + 'static, T: Clone + Send + Sync + 'static>(
        &mut self,
        commands: &mut Commands,
        current_widget: CurrentWidget,
        query: &Query<&Memo<D, T>>,
        dep: D,
        compute: impl FnOnce(&D) -> T,
    ) -> T {
        if let Some(entity) = self.get_state::<Memo<D, T>>(current_widget) {
            if let Ok(memo) = query.get(entity) {
                if memo.dep == dep {
                    return memo.value.clone();
                }
            }
        }
        let value = compute(&dep);
        let entity = self.use_state(
            commands,
            current_widget,
            Memo {
                dep: dep.clone(),
                value: value.clone(),
            },
        );
        commands.entity(entity).insert(Memo {
            dep,
            value: value.clone(),
        });
        value
    }

    /// Reports `true` exactly once, on the first render where at least `duration` has passed
    /// since this widget's first call to `use_timer` -- a one-shot `setTimeout`. Reports
    /// `false` on every call before and after that (it never fires twice). While pending, this
    /// widget is kept dirty every frame (see `crate::diffing::diff_pending_timers`) so it
    /// re-renders on its own even if nothing else about it changes -- `now` still needs to
    /// come from somewhere real, usually `Res<Time>::elapsed()`.
    pub fn use_timer(
        &mut self,
        commands: &mut Commands,
        current_widget: CurrentWidget,
        query: &Query<&TimerState>,
        duration: Duration,
        now: Duration,
    ) -> bool {
        let entity = self.use_state(
            commands,
            current_widget,
            TimerState {
                started_at: now,
                fired: false,
            },
        );
        let already_fired = query.get(entity).is_ok_and(|state| state.fired);
        if already_fired {
            commands.entity(entity).remove::<NeedsContinuedRender>();
            return false;
        }
        let started_at = query.get(entity).map(|state| state.started_at).unwrap_or(now);
        if now.saturating_sub(started_at) >= duration {
            commands.entity(entity).insert(TimerState {
                started_at,
                fired: true,
            });
            commands.entity(entity).remove::<NeedsContinuedRender>();
            true
        } else {
            commands.entity(entity).insert(NeedsContinuedRender);
            false
        }
    }

    /// Reports `true` on every render where at least `duration` has passed since the last time
    /// it reported `true` for this widget (`false` on the very first call, which just starts
    /// the countdown) -- a repeating `setInterval`. Like `use_timer`, this keeps the widget
    /// dirty every frame for as long as it's in use, since an interval never truly settles.
    pub fn use_interval(
        &mut self,
        commands: &mut Commands,
        current_widget: CurrentWidget,
        query: &Query<&IntervalState>,
        duration: Duration,
        now: Duration,
    ) -> bool {
        let entity = self.use_state(commands, current_widget, IntervalState { last_tick: now });
        commands.entity(entity).insert(NeedsContinuedRender);
        match query.get(entity) {
            Ok(state) if now.saturating_sub(state.last_tick) >= duration => {
                commands.entity(entity).insert(IntervalState { last_tick: now });
                true
            }
            _ => false,
        }
    }

    /// Returns `Some(value)` once `value` has been unchanged for at least `delay`, `None`
    /// while it's still within that window (including the first time it's ever seen) -- lets a
    /// render system react to a rapidly-changing input (search-as-you-type, a resize in
    /// progress) only once it settles, instead of on every intermediate change. Like
    /// `use_timer`, keeps the widget dirty every frame while a value is still settling.
    pub fn use_debounce<T: Clone + PartialEq + Send + Sync + 'static>(
        &mut self,
        commands: &mut Commands,
        current_widget: CurrentWidget,
        query: &Query<&DebounceState<T>>,
        value: T,
        delay: Duration,
        now: Duration,
    ) -> Option<T> {
        let entity = self.use_state(
            commands,
            current_widget,
            DebounceState {
                value: value.clone(),
                changed_at: now,
            },
        );
        let changed_at = match query.get(entity) {
            Ok(state) if state.value == value => state.changed_at,
            _ => {
                commands.entity(entity).insert(DebounceState {
                    value: value.clone(),
                    changed_at: now,
                });
                now
            }
        };
        if now.saturating_sub(changed_at) >= delay {
            commands.entity(entity).remove::<NeedsContinuedRender>();
            Some(value)
        } else {
            commands.entity(entity).insert(NeedsContinuedRender);
            None
        }
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

/// Returns `true` if any of `entity`'s own `use_state` entities still needs continued
/// rendering (see [`NeedsContinuedRender`]) -- a pending `use_timer`, a `use_interval` (which
/// never truly settles), or a `use_debounce` value still within its settling window. Mirrors
/// the role `diff_spring`/`diff_transition`'s own settled-state checks play in
/// [`crate::diffing::diff_widget_entity`].
pub(crate) fn diff_pending_timers(world: &mut World, entity: Entity) -> bool {
    let Some(hook_helper) = world.get_resource::<HookHelper>() else {
        return false;
    };
    hook_helper
        .own_state_entities(entity)
        .into_iter()
        .any(|state_entity| world.get::<NeedsContinuedRender>(state_entity).is_some())
}

/// A tag component used to mark previous widget entities.
#[derive(Component)]
pub struct PreviousWidget;

/// Bookkeeping component for [`HookHelper::use_effect`] -- the last-seen value of its
/// dependency, stored on the widget's own `use_state` entity purely so `use_effect` can
/// compare against it on the next call. Deliberately doesn't derive `Reflect`/`DiffableProp`:
/// it never needs to trigger a re-render by itself -- the caller's own props/state already do
/// that, and `use_effect` only reports, within a render that's already happening, whether this
/// specific dependency was part of the reason.
#[derive(Component, Clone)]
pub struct EffectDep<T>(pub T);

/// Bookkeeping component for [`HookHelper::use_previous`] -- see [`EffectDep`]'s doc comment
/// for why this doesn't derive `Reflect`/`DiffableProp`.
#[derive(Component, Clone)]
pub struct Previous<T>(pub T);

/// Bookkeeping component for [`HookHelper::use_memo`] -- see [`EffectDep`]'s doc comment for
/// why this doesn't derive `Reflect`/`DiffableProp`.
#[derive(Component, Clone)]
pub struct Memo<D, T> {
    dep: D,
    value: T,
}

/// Present on a state entity for exactly as long as one of `use_timer`/`use_interval`/
/// `use_debounce` still needs its owning widget to keep re-rendering every frame (a pending
/// timer, a repeating interval, or a value still settling) -- checked by
/// [`crate::diffing::diff_pending_timers`], the same role `Spring`/`Transition`'s own
/// settled-state checks play for those primitives.
#[derive(Component)]
pub(crate) struct NeedsContinuedRender;

/// Bookkeeping component for [`HookHelper::use_timer`].
#[derive(Component, Clone, Copy)]
pub struct TimerState {
    started_at: Duration,
    fired: bool,
}

/// Bookkeeping component for [`HookHelper::use_interval`].
#[derive(Component, Clone, Copy)]
pub struct IntervalState {
    last_tick: Duration,
}

/// Bookkeeping component for [`HookHelper::use_debounce`].
#[derive(Component, Clone)]
pub struct DebounceState<T> {
    value: T,
    changed_at: Duration,
}

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

    #[test]
    fn use_effect_reports_changed_on_first_call_then_only_when_the_dependency_differs() {
        let mut app = App::new();
        app.register_widget::<TestWidget>();
        app.init_resource::<HookHelper>();
        let widget = app.world_mut().spawn(TestWidget).id();

        let call = |app: &mut App, dep: u32| {
            app.world_mut()
                .run_system_once(
                    move |mut hooks: ResMut<HookHelper>,
                          mut commands: Commands,
                          query: Query<&EffectDep<u32>>| {
                        hooks.use_effect(&mut commands, CurrentWidget(widget), &query, dep)
                    },
                )
                .unwrap()
        };

        assert!(call(&mut app, 1), "the first call must always report changed");
        assert!(
            !call(&mut app, 1),
            "an unchanged dependency must not report changed again"
        );
        assert!(call(&mut app, 2), "a changed dependency must report changed");
        assert!(
            !call(&mut app, 2),
            "must settle again once the new value has been observed once"
        );
    }

    #[test]
    fn use_previous_returns_none_on_first_call_then_the_prior_value() {
        let mut app = App::new();
        app.register_widget::<TestWidget>();
        app.init_resource::<HookHelper>();
        let widget = app.world_mut().spawn(TestWidget).id();

        let call = |app: &mut App, current: u32| {
            app.world_mut()
                .run_system_once(
                    move |mut hooks: ResMut<HookHelper>,
                          mut commands: Commands,
                          query: Query<&Previous<u32>>| {
                        hooks.use_previous(&mut commands, CurrentWidget(widget), &query, current)
                    },
                )
                .unwrap()
        };

        assert_eq!(
            call(&mut app, 1),
            None,
            "nothing to compare against on the first call"
        );
        assert_eq!(
            call(&mut app, 2),
            Some(1),
            "must return what was passed in last time, not the current value"
        );
        assert_eq!(call(&mut app, 3), Some(2));
    }

    #[test]
    fn use_memo_only_recomputes_when_the_dependency_changes() {
        let mut app = App::new();
        app.register_widget::<TestWidget>();
        app.init_resource::<HookHelper>();
        let widget = app.world_mut().spawn(TestWidget).id();
        let compute_calls = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));

        let call = |app: &mut App, dep: u32| {
            let compute_calls = compute_calls.clone();
            app.world_mut()
                .run_system_once(
                    move |mut hooks: ResMut<HookHelper>,
                          mut commands: Commands,
                          query: Query<&Memo<u32, u32>>| {
                        hooks.use_memo(&mut commands, CurrentWidget(widget), &query, dep, |d| {
                            compute_calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                            d * 10
                        })
                    },
                )
                .unwrap()
        };

        assert_eq!(call(&mut app, 1), 10);
        assert_eq!(call(&mut app, 1), 10);
        assert_eq!(
            compute_calls.load(std::sync::atomic::Ordering::SeqCst),
            1,
            "an unchanged dependency must not recompute"
        );

        assert_eq!(call(&mut app, 2), 20);
        assert_eq!(
            compute_calls.load(std::sync::atomic::Ordering::SeqCst),
            2,
            "a changed dependency must recompute"
        );
    }

    #[test]
    fn use_timer_fires_exactly_once_after_the_duration_elapses() {
        let mut app = App::new();
        app.register_widget::<TestWidget>();
        app.init_resource::<HookHelper>();
        let widget = app.world_mut().spawn(TestWidget).id();

        let call = |app: &mut App, now_secs: u64| {
            app.world_mut()
                .run_system_once(
                    move |mut hooks: ResMut<HookHelper>,
                          mut commands: Commands,
                          query: Query<&TimerState>| {
                        hooks.use_timer(
                            &mut commands,
                            CurrentWidget(widget),
                            &query,
                            Duration::from_secs(3),
                            Duration::from_secs(now_secs),
                        )
                    },
                )
                .unwrap()
        };

        assert!(!call(&mut app, 0), "must not fire immediately");
        assert!(!call(&mut app, 2), "must not fire before the duration elapses");
        assert!(call(&mut app, 3), "must fire once the duration has elapsed");
        assert!(!call(&mut app, 5), "must not fire a second time");
    }

    #[test]
    fn use_interval_ticks_repeatedly() {
        let mut app = App::new();
        app.register_widget::<TestWidget>();
        app.init_resource::<HookHelper>();
        let widget = app.world_mut().spawn(TestWidget).id();

        let call = |app: &mut App, now_secs: u64| {
            app.world_mut()
                .run_system_once(
                    move |mut hooks: ResMut<HookHelper>,
                          mut commands: Commands,
                          query: Query<&IntervalState>| {
                        hooks.use_interval(
                            &mut commands,
                            CurrentWidget(widget),
                            &query,
                            Duration::from_secs(2),
                            Duration::from_secs(now_secs),
                        )
                    },
                )
                .unwrap()
        };

        assert!(!call(&mut app, 0), "must not tick on the frame it starts");
        assert!(!call(&mut app, 1), "must not tick before the interval elapses");
        assert!(call(&mut app, 2), "must tick once the interval elapses");
        assert!(
            !call(&mut app, 3),
            "must not tick again immediately after ticking"
        );
        assert!(call(&mut app, 4), "must tick again after another full interval");
    }

    #[test]
    fn use_debounce_settles_only_after_the_value_stops_changing() {
        let mut app = App::new();
        app.register_widget::<TestWidget>();
        app.init_resource::<HookHelper>();
        let widget = app.world_mut().spawn(TestWidget).id();

        let call = |app: &mut App, value: u32, now_secs: u64| {
            app.world_mut()
                .run_system_once(
                    move |mut hooks: ResMut<HookHelper>,
                          mut commands: Commands,
                          query: Query<&DebounceState<u32>>| {
                        hooks.use_debounce(
                            &mut commands,
                            CurrentWidget(widget),
                            &query,
                            value,
                            Duration::from_secs(1),
                            Duration::from_secs(now_secs),
                        )
                    },
                )
                .unwrap()
        };

        assert_eq!(
            call(&mut app, 1, 0),
            None,
            "not settled yet on the first observation"
        );
        assert_eq!(
            call(&mut app, 2, 0),
            None,
            "changing the value resets the settle window"
        );
        assert_eq!(
            call(&mut app, 2, 1),
            Some(2),
            "settled once unchanged for the full delay"
        );
        assert_eq!(
            call(&mut app, 2, 2),
            Some(2),
            "stays settled while the value keeps not changing"
        );
    }

    #[test]
    fn diff_pending_timers_reports_true_while_pending_and_false_once_fired() {
        let mut app = App::new();
        app.register_widget::<TestWidget>();
        app.init_resource::<HookHelper>();
        let widget = app.world_mut().spawn(TestWidget).id();

        app.world_mut()
            .run_system_once(
                move |mut hooks: ResMut<HookHelper>,
                      mut commands: Commands,
                      query: Query<&TimerState>| {
                    hooks.use_timer(
                        &mut commands,
                        CurrentWidget(widget),
                        &query,
                        Duration::from_secs(3),
                        Duration::from_secs(0),
                    )
                },
            )
            .unwrap();
        assert!(
            diff_pending_timers(app.world_mut(), widget),
            "must report true while the timer is still pending"
        );

        app.world_mut()
            .run_system_once(
                move |mut hooks: ResMut<HookHelper>,
                      mut commands: Commands,
                      query: Query<&TimerState>| {
                    hooks.use_timer(
                        &mut commands,
                        CurrentWidget(widget),
                        &query,
                        Duration::from_secs(3),
                        Duration::from_secs(3),
                    )
                },
            )
            .unwrap();
        assert!(
            !diff_pending_timers(app.world_mut(), widget),
            "must report false once the timer has fired"
        );
    }
}
