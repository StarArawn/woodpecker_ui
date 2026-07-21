use bevy::{
    ecs::{component::Mutable, entity::EntityHashMap},
    prelude::*,
    window::{CursorIcon, PrimaryWindow, SystemCursorIcon},
};

use crate::{
    children::WidgetChildren, picking_backend::PointerWorldPosition, portal::OverlayRoot,
    prelude::WidgetLayout, widgets::Element, CurrentWidget,
};

/// A phase transition reported by [`WidgetChildren::drag_state`] to its `set_drag` closure.
/// `Start`/`Move` carry the position the drag ghost should render at, already anchored to
/// the widget's on-screen position at drag start rather than snapped centered under the
/// cursor (matching the anchor math every hand-written drag in this crate used to repeat).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DragPhase {
    /// The drag just started, at this world-space position.
    Start(Vec2),
    /// The drag moved to this world-space position.
    Move(Vec2),
    /// The drag ended.
    End,
}

/// Per-entity drag anchors: the offset from the pointer to the widget's on-screen position,
/// captured at drag start. A `Resource` rather than a `Component` deliberately -- it's
/// written and read across two *different* observer systems (the `DragStart` and `Drag`
/// handlers below) for the same entity, and bevy_picking dispatches pointer events through
/// the same deferred `Commands` queue observers themselves use. A `Commands`-inserted
/// component can lose a race against an event already queued earlier in the same flush batch
/// (e.g. `DragEnter` on an adjacent drop zone queued right after `DragStart` in one motion);
/// a `ResMut`-backed map is written synchronously within the observer system itself, so
/// there's no flush to race.
#[derive(Resource, Default)]
pub(crate) struct DragAnchors(EntityHashMap<Vec2>);

impl WidgetChildren {
    /// Makes the last-added child (or self) draggable: reports phase transitions through
    /// `set_drag` on a `hooks.use_state`-backed component `S`, and sets the primary window's
    /// cursor to `Grabbing` while dragging (`Default` on drag end).
    ///
    /// Doesn't manage a drag payload -- for a [`WidgetChildren::droppable`] target to
    /// identify what was dropped, attach your own marker component directly to the dragged
    /// widget's bundle (permanently, not conditionally), the same way any other widget prop
    /// is declared; `droppable` reads it straight off `trigger.dragged`/`trigger.dropped`
    /// with a plain query.
    ///
    /// - `state_entity`: the state entity returned by `hooks.use_state(..)`.
    /// - `set_drag`: e.g. `|s: &mut ItemSlotState, phase| match phase { DragPhase::Start(p)
    ///   | DragPhase::Move(p) => { s.dragging = true; s.drag_position = p; } DragPhase::End
    ///   => s.dragging = false }`.
    pub fn drag_state<S: Component<Mutability = Mutable>>(
        &mut self,
        spawn_location: CurrentWidget,
        state_entity: Entity,
        set_drag: impl Fn(&mut S, DragPhase) + Send + Sync + Clone + 'static,
    ) -> &mut Self {
        let set_drag_start = set_drag.clone();
        self.observe(
            spawn_location,
            move |trigger: On<Pointer<DragStart>>,
                  mut commands: Commands,
                  mut state_query: Query<&mut S>,
                  mut anchors: ResMut<DragAnchors>,
                  window: Single<Entity, With<PrimaryWindow>>,
                  pointer_world: PointerWorldPosition,
                  layout_query: Query<&WidgetLayout>| {
                let Ok(mut state) = state_query.get_mut(state_entity) else {
                    return;
                };
                let Some(cursor_pos_world) =
                    pointer_world.convert(trigger.pointer_location.position)
                else {
                    return;
                };

                let widget_position = layout_query
                    .get(trigger.entity)
                    .map(|l| l.position())
                    .unwrap_or(cursor_pos_world);

                anchors
                    .0
                    .insert(trigger.entity, widget_position - cursor_pos_world);
                set_drag_start(&mut state, DragPhase::Start(widget_position));

                commands
                    .entity(*window)
                    .insert(CursorIcon::from(SystemCursorIcon::Grabbing));
            },
        );

        let set_drag_move = set_drag.clone();
        self.observe(
            spawn_location,
            move |trigger: On<Pointer<Drag>>,
                  mut state_query: Query<&mut S>,
                  pointer_world: PointerWorldPosition,
                  anchors: Res<DragAnchors>| {
                let Ok(mut state) = state_query.get_mut(state_entity) else {
                    return;
                };
                let Some(cursor_pos_world) =
                    pointer_world.convert(trigger.pointer_location.position)
                else {
                    return;
                };
                let anchor = anchors
                    .0
                    .get(&trigger.entity)
                    .copied()
                    .unwrap_or(Vec2::ZERO);
                set_drag_move(&mut state, DragPhase::Move(cursor_pos_world + anchor));
            },
        );

        self.observe(
            spawn_location,
            move |trigger: On<Pointer<DragEnd>>,
                  mut commands: Commands,
                  mut state_query: Query<&mut S>,
                  mut anchors: ResMut<DragAnchors>,
                  window: Single<Entity, With<PrimaryWindow>>| {
                let Ok(mut state) = state_query.get_mut(state_entity) else {
                    return;
                };
                anchors.0.remove(&trigger.entity);
                set_drag(&mut state, DragPhase::End);

                commands
                    .entity(*window)
                    .insert(CursorIcon::from(SystemCursorIcon::Default));
            },
        );

        self
    }

    /// Builder form of [`WidgetChildren::drag_state`].
    pub fn with_drag_state<S: Component<Mutability = Mutable>>(
        mut self,
        spawn_location: CurrentWidget,
        state_entity: Entity,
        set_drag: impl Fn(&mut S, DragPhase) + Send + Sync + Clone + 'static,
    ) -> Self {
        self.drag_state(spawn_location, state_entity, set_drag);
        self
    }

    /// Declares a `.portal()`-ed drag-ghost child showing `ghost` (a full bundle -- its own
    /// style, `WidgetRender`, content, `Pickable::IGNORE`, exactly as it should appear while
    /// dragging) whenever `dragging` is `true`. Call every render, right alongside
    /// `.drag_state(...)`, regardless of whether currently dragging -- declaring nothing this
    /// frame (because `dragging` just became `false`) is what despawns a stale ghost from the
    /// previous frame via normal keyed reconciliation.
    ///
    /// Exists because a dragged widget is usually declared deep inside some `Clip`-masked
    /// and/or z-bucketed ancestor (a `WoodpeckerWindow`'s interior, a `ScrollBox`) -- merely
    /// restyling the dragged widget itself to `Fixed` positioning (the obvious first attempt)
    /// still leaves it a physical descendant of that ancestor, so it gets scissor-clipped and
    /// loses to unrelated sibling content in z-order instead of floating freely above
    /// everything the way `Modal`/`Toast`/`Popover`/`WoodpeckerWindow` already do internally.
    /// This portals a *separate* ghost child instead, the same escape hatch every other
    /// built-in floating widget in this crate uses. The widget actually receiving the drag
    /// events must stay in the tree, itself unportaled (see [`Self::drag_state`]'s own doc
    /// comment for why only the *declaring* widget can portal a child, never itself) -- hide it
    /// instead via `display: WidgetDisplay::None` on its own style while `dragging`, so its
    /// observers (and the drag gesture bevy_picking is tracking against its entity id) survive
    /// for the rest of the gesture.
    ///
    /// No-ops if `dragging` is `false`, or if [`OverlayRoot`] hasn't been provisioned yet (a
    /// sibling of `OverlayRootWidget` in the same initial tree can in principle render before
    /// `sync_overlay_root` has observed it -- skipping a frame beats panicking).
    pub fn drag_ghost(
        &mut self,
        overlay_root: Option<OverlayRoot>,
        dragging: bool,
        ghost: impl Bundle + Clone,
    ) -> &mut Self {
        if !dragging {
            return self;
        }
        let Some(overlay_root) = overlay_root else {
            return self;
        };
        self.add::<Element>(ghost);
        self.add_key("drag_ghost");
        self.portal_to(overlay_root.0);
        self
    }

    /// Builder form of [`WidgetChildren::drag_ghost`].
    pub fn with_drag_ghost(
        mut self,
        overlay_root: Option<OverlayRoot>,
        dragging: bool,
        ghost: impl Bundle + Clone,
    ) -> Self {
        self.drag_ghost(overlay_root, dragging, ghost);
        self
    }

    /// Makes the last-added child (or self) a drop target for payload `T`: `accept`
    /// validates the incoming payload (pure -- decide from the payload's own fields, not a
    /// resource lookup, since it runs with no system-param access); `on_hover(state,
    /// Some(valid))` fires on drag-enter and `on_hover(state, None)` fires on drag-leave
    /// *and* immediately after any drop attempt, so a highlight border always clears;
    /// `on_drop(payload, valid, commands)` fires on every drop attempt, valid or not, with a
    /// `Commands` handle so it can queue resource mutations (e.g. equip the item) or push a
    /// toast either way.
    ///
    /// `T` must be a component the dragged widget carries permanently in its own bundle (see
    /// [`WidgetChildren::drag_state`]'s doc comment), not one inserted reactively -- so it's
    /// always present by the time a drag reaches this drop target, however early in the
    /// drag's lifetime that happens.
    ///
    /// - `state_entity`: the state entity returned by `hooks.use_state(..)`.
    pub fn droppable<T: Component, S: Component<Mutability = Mutable>>(
        &mut self,
        spawn_location: CurrentWidget,
        state_entity: Entity,
        accept: impl Fn(&T, &S) -> bool + Send + Sync + Clone + 'static,
        on_hover: impl Fn(&mut S, Option<bool>) + Send + Sync + Clone + 'static,
        on_drop: impl Fn(&T, bool, &S, &mut Commands) + Send + Sync + Clone + 'static,
    ) -> &mut Self {
        let accept_enter = accept.clone();
        let on_hover_enter = on_hover.clone();
        self.observe(
            spawn_location,
            move |trigger: On<Pointer<DragEnter>>,
                  dragged_query: Query<&T>,
                  mut state_query: Query<&mut S>| {
                let Ok(mut state) = state_query.get_mut(state_entity) else {
                    return;
                };
                let Ok(payload) = dragged_query.get(trigger.dragged) else {
                    return;
                };
                let valid = accept_enter(payload, &state);
                on_hover_enter(&mut state, Some(valid));
            },
        );

        let on_hover_leave = on_hover.clone();
        self.observe(
            spawn_location,
            move |_trigger: On<Pointer<DragLeave>>, mut state_query: Query<&mut S>| {
                let Ok(mut state) = state_query.get_mut(state_entity) else {
                    return;
                };
                on_hover_leave(&mut state, None);
            },
        );

        self.observe(
            spawn_location,
            move |trigger: On<Pointer<DragDrop>>,
                  mut commands: Commands,
                  dragged_query: Query<&T>,
                  mut state_query: Query<&mut S>| {
                let Ok(mut state) = state_query.get_mut(state_entity) else {
                    return;
                };

                let Ok(payload) = dragged_query.get(trigger.dropped) else {
                    on_hover(&mut state, None);
                    return;
                };
                let valid = accept(payload, &state);
                on_hover(&mut state, None);
                on_drop(payload, valid, &state, &mut commands);
            },
        );

        self
    }

    /// Builder form of [`WidgetChildren::droppable`].
    pub fn with_droppable<T: Component, S: Component<Mutability = Mutable>>(
        mut self,
        spawn_location: CurrentWidget,
        state_entity: Entity,
        accept: impl Fn(&T, &S) -> bool + Send + Sync + Clone + 'static,
        on_hover: impl Fn(&mut S, Option<bool>) + Send + Sync + Clone + 'static,
        on_drop: impl Fn(&T, bool, &S, &mut Commands) + Send + Sync + Clone + 'static,
    ) -> Self {
        self.droppable(spawn_location, state_entity, accept, on_hover, on_drop);
        self
    }
}
