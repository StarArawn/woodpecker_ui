use bevy::{
    ecs::system::SystemParam,
    prelude::*,
    window::{CursorIcon, PrimaryWindow, SystemCursorIcon},
};

use crate::{children::WidgetChildren, CurrentWidget};

/// A `SystemParam` for setting the primary window's cursor icon from a widget's `Pointer<Over>`/
/// `Pointer<Out>` observers, without each one repeating the `Commands` + `Single<Entity, With<PrimaryWindow>>`
/// boilerplate.
#[derive(SystemParam)]
pub struct CursorSetter<'w, 's> {
    commands: Commands<'w, 's>,
    window: Single<'w, 's, Entity, With<PrimaryWindow>>,
}

impl CursorSetter<'_, '_> {
    /// Sets the primary window's cursor to `icon`.
    pub fn set(&mut self, icon: SystemCursorIcon) {
        self.commands
            .entity(*self.window)
            .insert(CursorIcon::from(icon));
    }
}

impl WidgetChildren {
    /// Adds `Pointer<Over>`/`Pointer<Out>` observers that set the primary window's cursor to
    /// `hover_icon` on hover and back to `SystemCursorIcon::Default` on exit. For widgets that
    /// only need a hover cursor with no accompanying style/state change (e.g. table header
    /// cells, radio options, a slider track).
    ///
    /// `spawn_location`: same meaning as [`WidgetChildren::observe`]'s `spawn_location` -- pass
    /// `current_widget` from the render system. Like `observe`, if called before any child has
    /// been queued via `add`/`with_child`, the observers attach to the widget's own entity;
    /// otherwise they attach to the most recently added child. If you want a hover cursor on
    /// the widget *that owns this `WidgetChildren`* regardless of queue state (e.g. from that
    /// widget's own `render`, whose children arrive pre-populated via its spawn bundle rather
    /// than its own `add` calls), use [`Self::self_hover_cursor`] instead.
    pub fn hover_cursor(
        &mut self,
        spawn_location: CurrentWidget,
        hover_icon: SystemCursorIcon,
    ) -> &mut Self {
        self.observe(
            spawn_location,
            move |_: On<Pointer<Over>>, mut cursor: CursorSetter| {
                cursor.set(hover_icon);
            },
        );
        self.observe(
            spawn_location,
            |_: On<Pointer<Out>>, mut cursor: CursorSetter| {
                cursor.set(SystemCursorIcon::Default);
            },
        );
        self
    }

    /// Builder form of [`WidgetChildren::hover_cursor`].
    pub fn with_hover_cursor(
        mut self,
        spawn_location: CurrentWidget,
        hover_icon: SystemCursorIcon,
    ) -> Self {
        self.hover_cursor(spawn_location, hover_icon);
        self
    }

    /// Same as [`Self::hover_cursor`], but always targets the widget entity that owns this
    /// `WidgetChildren`, regardless of whether any children have already been queued via
    /// `add`/`with_child` -- see [`WidgetChildren::self_observe`] for why this distinction
    /// matters. Use this from a widget's own `render`, e.g. `WButton`/`Slider`/`Toggle`, whose
    /// children arrive pre-populated via the caller's spawn bundle rather than that widget's
    /// own `add` calls (so `children_queue` is never actually empty by the time `render` runs).
    pub fn self_hover_cursor(
        &mut self,
        spawn_location: CurrentWidget,
        hover_icon: SystemCursorIcon,
    ) -> &mut Self {
        self.self_observe(
            spawn_location,
            move |_: On<Pointer<Over>>, mut cursor: CursorSetter| {
                cursor.set(hover_icon);
            },
        );
        self.self_observe(
            spawn_location,
            |_: On<Pointer<Out>>, mut cursor: CursorSetter| {
                cursor.set(SystemCursorIcon::Default);
            },
        );
        self
    }

    /// Builder form of [`WidgetChildren::self_hover_cursor`].
    pub fn with_self_hover_cursor(
        mut self,
        spawn_location: CurrentWidget,
        hover_icon: SystemCursorIcon,
    ) -> Self {
        self.self_hover_cursor(spawn_location, hover_icon);
        self
    }

    /// Adds `Pointer<Over>`/`Pointer<Out>` observers that set the cursor to `hover_icon` on
    /// hover (`SystemCursorIcon::Default` on exit) *and* flip a boolean hover flag on a
    /// `hooks.use_state`-backed component `S`, e.g. `WButtonState`, `CheckboxState`,
    /// `ToggleState`. Use this when hover also drives a normal/hovered style variant.
    ///
    /// - `state_entity`: the state entity returned by `hooks.use_state(..)`.
    /// - `set_hovering`: a small setter, e.g. `|s: &mut WButtonState, hovering| s.hovering = hovering`.
    ///
    /// Like [`Self::hover_cursor`], this attaches to the most recently queued child if one
    /// exists, or the owning widget otherwise -- use [`Self::self_hover_state`] to always
    /// target the owning widget regardless of queue state.
    pub fn hover_state<S: Component<Mutability = bevy::ecs::component::Mutable>>(
        &mut self,
        spawn_location: CurrentWidget,
        state_entity: Entity,
        hover_icon: SystemCursorIcon,
        set_hovering: impl Fn(&mut S, bool) + Send + Sync + Clone + 'static,
    ) -> &mut Self {
        let on_over = set_hovering.clone();
        self.observe(
            spawn_location,
            move |_: On<Pointer<Over>>,
                  mut state_query: Query<&mut S>,
                  mut cursor: CursorSetter| {
                if let Ok(mut state) = state_query.get_mut(state_entity) {
                    on_over(&mut state, true);
                }
                cursor.set(hover_icon);
            },
        );
        self.observe(
            spawn_location,
            move |_: On<Pointer<Out>>, mut state_query: Query<&mut S>, mut cursor: CursorSetter| {
                if let Ok(mut state) = state_query.get_mut(state_entity) {
                    set_hovering(&mut state, false);
                }
                cursor.set(SystemCursorIcon::Default);
            },
        );
        self
    }

    /// Builder form of [`WidgetChildren::hover_state`].
    pub fn with_hover_state<S: Component<Mutability = bevy::ecs::component::Mutable>>(
        mut self,
        spawn_location: CurrentWidget,
        state_entity: Entity,
        hover_icon: SystemCursorIcon,
        set_hovering: impl Fn(&mut S, bool) + Send + Sync + Clone + 'static,
    ) -> Self {
        self.hover_state(spawn_location, state_entity, hover_icon, set_hovering);
        self
    }

    /// Same as [`Self::hover_state`], but always targets the widget entity that owns this
    /// `WidgetChildren` -- see [`Self::self_hover_cursor`]/[`WidgetChildren::self_observe`]
    /// for why this distinction matters.
    pub fn self_hover_state<S: Component<Mutability = bevy::ecs::component::Mutable>>(
        &mut self,
        spawn_location: CurrentWidget,
        state_entity: Entity,
        hover_icon: SystemCursorIcon,
        set_hovering: impl Fn(&mut S, bool) + Send + Sync + Clone + 'static,
    ) -> &mut Self {
        let on_over = set_hovering.clone();
        self.self_observe(
            spawn_location,
            move |_: On<Pointer<Over>>,
                  mut state_query: Query<&mut S>,
                  mut cursor: CursorSetter| {
                if let Ok(mut state) = state_query.get_mut(state_entity) {
                    on_over(&mut state, true);
                }
                cursor.set(hover_icon);
            },
        );
        self.self_observe(
            spawn_location,
            move |_: On<Pointer<Out>>, mut state_query: Query<&mut S>, mut cursor: CursorSetter| {
                if let Ok(mut state) = state_query.get_mut(state_entity) {
                    set_hovering(&mut state, false);
                }
                cursor.set(SystemCursorIcon::Default);
            },
        );
        self
    }

    /// Builder form of [`WidgetChildren::self_hover_state`].
    pub fn with_self_hover_state<S: Component<Mutability = bevy::ecs::component::Mutable>>(
        mut self,
        spawn_location: CurrentWidget,
        state_entity: Entity,
        hover_icon: SystemCursorIcon,
        set_hovering: impl Fn(&mut S, bool) + Send + Sync + Clone + 'static,
    ) -> Self {
        self.self_hover_state(spawn_location, state_entity, hover_icon, set_hovering);
        self
    }
}
