use crate::prelude::*;
use bevy::{
    prelude::*,
    window::{CursorIcon, PrimaryWindow, SystemCursorIcon},
};

/// A splitter drag event. `Splitter` only reports movement -- it doesn't own or resize any
/// panes itself (a widget can't reach into sibling entities' styles from its own render
/// pass), so the caller's own observer applies `delta` to whichever panes it placed on
/// either side of the splitter, e.g. by adjusting their `flex_basis`.
#[derive(Reflect, Debug, Clone, PartialEq, Default)]
pub struct SplitterChanged {
    /// Total drag distance along the splitter's drag axis (world/viewport units, not raw
    /// screen pixels) since the drag started: X for a vertical splitter, Y for a horizontal
    /// one. Positive is right/down.
    pub delta: f32,
}

/// Splitter state: hover/drag (for the active-color highlight) and the drag's world-space
/// start position, so `Change<SplitterChanged>::delta` can be measured from drag start
/// rather than accumulated frame-to-frame (which would drift under rounding).
#[derive(Component, Debug, Reflect, Clone, Copy, PartialEq, Default)]
#[reflect(Component, DiffableProp, PartialEq)]
pub struct SplitterState {
    /// Whether the pointer is hovering the splitter.
    pub hovering: bool,
    /// Whether the splitter is currently being dragged.
    pub dragging: bool,
    drag_start_world: Vec2,
}

/// [`Splitter`]'s themed colors, a separate sibling component (rather than plain fields on
/// `Splitter` itself) so they can live-resync on a [`Theme`] swap -- see
/// [`ThemeRegisterExt::register_themed_style`]. A caller wanting custom colors provides its
/// own `SplitterStyles` (plus [`ThemeOverride`] to opt out of future resyncs).
#[derive(Component, Reflect, Clone, Copy, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
pub struct SplitterStyles {
    /// Line color while idle.
    pub color: Color,
    /// Line color while hovered or dragged -- the drag affordance.
    pub active_color: Color,
}

impl Default for SplitterStyles {
    fn default() -> Self {
        Self::from_theme(&Theme::default())
    }
}

impl ThemedStyle for SplitterStyles {
    fn from_theme(theme: &Theme) -> Self {
        Self {
            color: theme.border,
            active_color: theme.primary,
        }
    }
}

/// A thin draggable divider between two flex children -- fires `Change<SplitterChanged>` as
/// it's dragged; the caller resizes its own panes in response. Extends [`Divider`] with drag
/// support and a hover/active color instead of a single static one.
#[derive(Widget, Component, Reflect, Clone, Copy, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(WoodpeckerStyle, WidgetChildren, WidgetRender = WidgetRender::Quad, Pickable, SplitterStyles)]
pub struct Splitter {
    /// Renders as a vertical line (fills height, fixed 4px width) that drags horizontally to
    /// resize left/right panes -- instead of the default horizontal line (fills width, fixed
    /// 4px height) that drags vertically to resize top/bottom panes. Matches `Divider`'s
    /// `vertical` semantic exactly.
    pub vertical: bool,
}

impl Default for Splitter {
    fn default() -> Self {
        Self { vertical: true }
    }
}

fn render(
    mut commands: Commands,
    current_widget: Res<CurrentWidget>,
    mut hooks: ResMut<HookHelper>,
    mut query: Query<(
        &Splitter,
        &SplitterStyles,
        &mut WoodpeckerStyle,
        &mut WidgetChildren,
    )>,
    state_query: Query<&SplitterState>,
) {
    let Ok((splitter, splitter_styles, mut styles, mut children)) = query.get_mut(**current_widget)
    else {
        return;
    };

    let state_entity = hooks.use_state(&mut commands, *current_widget, SplitterState::default());
    let default_state = SplitterState::default();
    let state = state_query.get(state_entity).unwrap_or(&default_state);

    let color = if state.hovering || state.dragging {
        splitter_styles.active_color
    } else {
        splitter_styles.color
    };
    styles.background_color = color;
    if splitter.vertical {
        styles.width = 4.0.into();
        if styles.height == Units::Auto {
            styles.height = Units::Percentage(100.0);
        }
    } else {
        styles.height = 4.0.into();
        if styles.width == Units::Auto {
            styles.width = Units::Percentage(100.0);
        }
    }

    let current_widget = *current_widget;
    let vertical = splitter.vertical;
    let resize_cursor = if vertical {
        SystemCursorIcon::ColResize
    } else {
        SystemCursorIcon::RowResize
    };

    // Deliberately not `hover_cursor`/`hover_state`: those reset the cursor to `Default` on
    // `Pointer<Out>`, but a splitter doesn't move to track the cursor the way a dragged
    // window does, so `Out` fires almost immediately once a drag carries the pointer past
    // this splitter's own thin hitbox -- resetting the cursor mid-drag would flicker it back
    // to the arrow while the user is still actively dragging. Instead the cursor is always
    // derived from the combined `hovering || dragging` state, set fresh after every
    // transition, so leaving the hitbox mid-drag has no effect until the drag itself ends.
    let set_cursor = move |commands: &mut Commands, window: Entity, state: &SplitterState| {
        let icon = if state.hovering || state.dragging {
            resize_cursor
        } else {
            SystemCursorIcon::Default
        };
        commands.entity(window).insert(CursorIcon::from(icon));
    };

    *children = WidgetChildren::default();
    children.observe(
        current_widget,
        move |_trigger: On<Pointer<Over>>,
              mut commands: Commands,
              mut state_query: Query<&mut SplitterState>,
              window: Single<Entity, With<PrimaryWindow>>| {
            let Ok(mut state) = state_query.get_mut(state_entity) else {
                return;
            };
            state.hovering = true;
            set_cursor(&mut commands, *window, &state);
        },
    );
    children.observe(
        current_widget,
        move |_trigger: On<Pointer<Out>>,
              mut commands: Commands,
              mut state_query: Query<&mut SplitterState>,
              window: Single<Entity, With<PrimaryWindow>>| {
            let Ok(mut state) = state_query.get_mut(state_entity) else {
                return;
            };
            state.hovering = false;
            set_cursor(&mut commands, *window, &state);
        },
    );
    children.observe(
        current_widget,
        move |trigger: On<Pointer<DragStart>>,
              mut commands: Commands,
              mut state_query: Query<&mut SplitterState>,
              window: Single<Entity, With<PrimaryWindow>>,
              pointer_world: PointerWorldPosition| {
            let Ok(mut state) = state_query.get_mut(state_entity) else {
                return;
            };
            let Some(cursor_pos_world) = pointer_world.convert(trigger.pointer_location.position)
            else {
                return;
            };
            state.drag_start_world = cursor_pos_world;
            state.dragging = true;
            set_cursor(&mut commands, *window, &state);
        },
    );
    children.observe(
        current_widget,
        move |trigger: On<Pointer<Drag>>,
              mut commands: Commands,
              state_query: Query<&SplitterState>,
              pointer_world: PointerWorldPosition| {
            let Ok(state) = state_query.get(state_entity) else {
                return;
            };
            let Some(cursor_pos_world) = pointer_world.convert(trigger.pointer_location.position)
            else {
                return;
            };
            let drag_delta = cursor_pos_world - state.drag_start_world;
            commands.trigger(Change {
                target: current_widget.0,
                data: SplitterChanged {
                    delta: if vertical { drag_delta.x } else { drag_delta.y },
                },
            });
        },
    );
    children.observe(
        current_widget,
        move |_trigger: On<Pointer<DragEnd>>,
              mut commands: Commands,
              mut state_query: Query<&mut SplitterState>,
              window: Single<Entity, With<PrimaryWindow>>| {
            let Ok(mut state) = state_query.get_mut(state_entity) else {
                return;
            };
            state.dragging = false;
            set_cursor(&mut commands, *window, &state);
        },
    );

    children.apply(current_widget.as_parent());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_theme_tracks_border_and_primary() {
        let dark = SplitterStyles::from_theme(&Theme::dark());
        assert_eq!(dark.color, Theme::dark().border);
        assert_eq!(dark.active_color, Theme::dark().primary);

        let light = SplitterStyles::from_theme(&Theme::light());
        assert_eq!(light.color, Theme::light().border);
        assert_ne!(
            dark.color, light.color,
            "from_theme must track the passed-in theme, not a fixed default"
        );
    }
}
