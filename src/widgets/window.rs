use crate::prelude::*;
use bevy::{
    prelude::*,
    window::{CursorIcon, PrimaryWindow, SystemCursorIcon},
};

/// [`WoodpeckerWindow`]'s themed default colors/radius, used to build
/// `WoodpeckerWindow::default()`'s `window_styles`/`title_styles`. Since those are plain,
/// fully caller-overridable `WoodpeckerStyle` fields (not a queried sibling component like
/// most other widgets' `*Styles`), a `Theme` swap after a window has already spawned doesn't
/// retroactively restyle it -- same caveat as `Avatar`'s default color. This component still
/// exists (and is registered via [`ThemeRegisterExt::register_themed_style`]) so the *default*
/// a newly-spawned window picks up tracks the active `Theme` instead of the static `colors`
/// module, and so a caller building a custom-but-still-themed window has a ready-made source
/// of truth to read from.
#[derive(Component, Reflect, Clone, Copy, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
pub struct WindowStyles {
    /// Title bar background color.
    pub title_bar_background: Color,
    /// Content area background color.
    pub body_background: Color,
    /// Border color for both the title bar and content area.
    pub border_color: Color,
    /// Title text color.
    pub title_text_color: Color,
    /// Corner radius for the window's outer shape.
    pub corner_radius: f32,
    /// Drop shadow for the window's outer shape.
    pub box_shadow: WidgetBoxShadow,
}

impl Default for WindowStyles {
    fn default() -> Self {
        Self::from_theme(&Theme::default())
    }
}

impl ThemedStyle for WindowStyles {
    fn from_theme(theme: &Theme) -> Self {
        Self {
            title_bar_background: theme.dark_background,
            body_background: theme.background,
            // Not `theme.dark_background` -- that's the *darkest* surface in dark mode but
            // becomes the *lightest* (near-white) one in light mode (light theme inverts the
            // surface scale), so a border using it is invisible against light mode's own
            // near-white page background. `theme.border` is the token this crate's other
            // themed styles already use for exactly this purpose, and stays visible in both.
            border_color: theme.border,
            title_text_color: theme.text,
            corner_radius: theme.panel_radius,
            box_shadow: theme.elevation.xl,
        }
    }
}

/// State to keep track of window data.
#[derive(Default, Debug, Component, Reflect, PartialEq, Clone)]
#[reflect(Component, DiffableProp, PartialEq)]
pub struct WindowState {
    /// The position of the window.
    position: Vec2,
    drag_offset: Vec2,
}

/// Last-observed `WidgetLayout` of this window's own portaled `window_wrapper` child --
/// purely so [`diff_window_wrapper_layout`] can force a re-render when it changes. Not meant
/// to be read by anything else.
#[derive(Component, Default)]
struct PreviousWrapperLayout(Option<WidgetLayout>);

/// Forces a `WoodpeckerWindow` to re-render whenever its own portaled `window_wrapper`
/// child's `WidgetLayout` changes -- e.g. once the wrapper's `Auto`-driven width finishes
/// settling after mount. Without this, an `Auto`-width window's title bar (whose own width
/// mirrors the wrapper's, read one-frame-lagged via `WidgetMapper::get_keyed_child` since the
/// wrapper is portaled -- see `render`'s own comment) can permanently freeze at whatever size
/// happened to be known during the last render that occurred *before* the wrapper's size
/// finished settling, if nothing else about the window's own props/state changes afterward to
/// trigger another render (confirmed empirically: `examples/window.rs` gets enough incidental
/// re-renders from its own viewport-clamp settling to mask this, but `examples/viewport.rs`'s
/// single, otherwise-static window does not, and its title bar was observed stuck at the
/// pre-settle size).
///
/// Mirrors `layout::system::diff_watched_layout`'s pattern, but watches a *different* entity
/// (the wrapper) than the one being diffed (the window itself) -- `WatchLayout` only supports
/// watching an entity's own layout, so this is a small, window-specific variant of it instead
/// of a generic crate feature.
pub(crate) fn diff_window_wrapper_layout(world: &mut World, entity: Entity) -> bool {
    if world.get::<WoodpeckerWindow>(entity).is_none() {
        return false;
    }
    let Some(wrapper_entity) = world.get_resource::<WidgetMapper>().and_then(|mapper| {
        mapper.get_keyed_child::<Element>(ParentWidget(entity), "window_wrapper")
    }) else {
        return false;
    };
    let Some(&current) = world.get::<WidgetLayout>(wrapper_entity) else {
        return false;
    };
    if current.size == Vec2::ZERO {
        return false;
    }

    let changed = match world.get::<PreviousWrapperLayout>(entity) {
        Some(previous) => previous.0 != Some(current),
        None => true,
    };

    if changed {
        world
            .entity_mut(entity)
            .insert(PreviousWrapperLayout(Some(current)));
    }

    changed
}

/// Window widget
///
/// Deliberately doesn't require `Pickable`/`Focusable`/`StackingContext` here -- its whole
/// visible/interactive surface is a synthetic `window_wrapper` child `render` portals to
/// `OverlayRoot` (a widget can't portal itself, only whoever *declares* a child can), so those
/// need to live on that wrapper, the entity that's actually hit-tested and Tab-order-walked in
/// the real tree, not on this stable-but-otherwise-inert logical entity.
#[derive(Widget, Component, Reflect, PartialEq, Clone)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(
    WoodpeckerStyle,
    PassedChildren,
    WidgetChildren,
    PreviousWrapperLayout,
    WindowStyles
)]
pub struct WoodpeckerWindow {
    /// The title of the window.
    pub title: String,
    /// Initial Position
    pub initial_position: Vec2,
    /// Whether the window is shown at all. Defaults to `true`.
    ///
    /// Since this widget's whole visible/interactive surface lives on a portaled
    /// `window_wrapper` child (see this struct's own doc comment), it can't be hidden by
    /// putting `display: WidgetDisplay::None`/`visibility: WidgetVisibility::Hidden` on an
    /// *ancestor* the way a non-portaled widget can -- the portaled content is no longer a
    /// physical descendant of that ancestor, so the display-none cascade never reaches it.
    /// This field is the supported way to hide/show a `WoodpeckerWindow` instead: it's a real,
    /// `DiffableProp`-reflected field on the widget itself, so toggling it (even via a direct
    /// `Query<&mut WoodpeckerWindow>` write from outside `render`, not just by re-declaring the
    /// bundle) is correctly picked up by the generic per-widget diff and forces a re-render
    /// -- unlike `WoodpeckerStyle`, which is deliberately excluded from diffing (see its own
    /// doc comment) and so changing it alone would never be noticed.
    pub visible: bool,
    /// Styles for the window widget
    pub window_styles: WoodpeckerStyle,
    /// Styles for the title
    pub title_styles: WoodpeckerStyle,
    /// Styles for the divider under the title
    pub divider_styles: WoodpeckerStyle,
    /// Styles for the children
    pub children_styles: WoodpeckerStyle,
}

impl Default for WoodpeckerWindow {
    fn default() -> Self {
        // Sourced from `WindowStyles::default()` (i.e. the active `Theme` at the moment this
        // runs), not the static `colors` module -- see `WindowStyles`'s own doc comment for
        // why this only affects a *newly-spawned* window's initial styling, not already-live
        // ones, since `window_styles`/`title_styles` are plain caller-overridable fields.
        let window_styles = WindowStyles::default();
        Self {
            title: Default::default(),
            initial_position: Default::default(),
            visible: true,
            window_styles: WoodpeckerStyle {
                background_color: window_styles.body_background,
                border_color: window_styles.border_color,
                border: Edge::all(2.0),
                border_radius: Corner::all(window_styles.corner_radius),
                flex_direction: WidgetFlexDirection::Column,
                ..Default::default()
            },
            title_styles: WoodpeckerStyle {
                background_color: window_styles.title_bar_background,
                height: Units::Pixels(40.0),
                width: Units::Percentage(100.0),
                align_items: Some(WidgetAlignItems::Center),
                padding: Edge::all(0.0).left(10.0).right(10.0),
                // Separates the title bar from the body below it -- without this, the two
                // fills blend together whenever they're close in tone (most noticeable in
                // light mode, where `title_bar_background`/`body_background` sit closer
                // together than in dark mode).
                border: Edge::all(2.0).bottom(0.0),
                border_radius: Corner::all(window_styles.corner_radius),
                border_color: window_styles.border_color,
                ..Default::default()
            },
            divider_styles: WoodpeckerStyle {
                ..Default::default()
            },
            children_styles: WoodpeckerStyle {
                ..Default::default()
            },
        }
    }
}

fn render(
    mut commands: Commands,
    current_widget: Res<CurrentWidget>,
    mut hooks: ResMut<HookHelper>,
    widget_mapper: Res<WidgetMapper>,
    // `Option`, not a hard `Res`: a sibling of `OverlayRootWidget` in the same initial tree can
    // in principle render before `sync_overlay_root` has observed it -- see that system's doc
    // comment. Skipping a frame beats panicking on a legitimate startup-order race.
    overlay_root: Option<Res<OverlayRoot>>,
    mut query: Query<(
        &WoodpeckerWindow,
        &WindowStyles,
        &mut WidgetChildren,
        &PassedChildren,
        Option<&TitleChildren>,
    )>,
    mut state_query: Query<&mut WindowState>,
    mut context_query: Query<&mut WindowingContext>,
    // Merged into one query (rather than a second `Query<&WidgetLayout, With<WoodpeckerApp>>`)
    // to stay under the `#[auto_update]` hot-reload macro's 9-parameter limit. Marks the root
    // entity directly via `Has<WoodpeckerApp>`, deliberately not `Res<WoodpeckerContext>` --
    // see the long comment on this same pattern in `examples/game_ui/settings_panel.rs`.
    layout_query: Query<(&WidgetLayout, Has<WoodpeckerApp>)>,
) {
    let Ok((window, window_styles, mut children, passed_children, title_children)) =
        query.get_mut(**current_widget)
    else {
        return;
    };

    let state_entity = hooks.use_state(
        &mut commands,
        *current_widget,
        WindowState {
            position: window.initial_position,
            drag_offset: Vec2::new(0.0, 0.0),
        },
    );

    let Ok(mut state) = state_query.get_mut(state_entity) else {
        return;
    };

    // This widget can't portal itself (only whoever *declares* a child can mark it
    // `.portal()`-ed), so the window's real visible/interactive surface is a synthetic
    // `window_wrapper` child (below) portaled to `OverlayRoot` -- meaning `current_widget`'s
    // own `WidgetLayout` no longer reflects the window's actual on-screen size/position.
    // `WidgetMapper::get_child` recovers the wrapper's stable entity id from the *last*
    // completed reconciliation pass (one-frame-lagged, since this frame's own layout hasn't
    // run yet -- the same lag `devtools/highlight.rs` already accepts elsewhere).
    let wrapper_entity =
        widget_mapper.get_keyed_child::<Element>(current_widget.as_parent(), "window_wrapper");
    let wrapper_size = wrapper_entity
        .and_then(|e| layout_query.get(e).ok())
        .map(|(l, _)| l.size)
        .unwrap_or(Vec2::ZERO);

    // Clamp on every render, not just while dragging (see the `Pointer<Drag>` observer
    // below), so a window can't open or end up off-screen after a viewport resize.
    if let Some((viewport, _)) = layout_query.iter().find(|(_, is_root)| *is_root) {
        let clamped = Vec2::new(
            state
                .position
                .x
                .clamp(0.0, (viewport.size.x - wrapper_size.x).max(0.0)),
            state
                .position
                .y
                .clamp(0.0, (viewport.size.y - wrapper_size.y).max(0.0)),
        );
        if clamped != state.position {
            state.position = clamped;
        }
    }

    // Setup windowing context.
    let context_entity =
        hooks.use_context(&mut commands, *current_widget, WindowingContext::default());

    let Ok(mut context) = context_query.get_mut(context_entity) else {
        return;
    };

    let z_index = context.get_or_add(current_widget.entity());

    *children = WidgetChildren::default();
    let current_widget = *current_widget;

    let Some(overlay_root) = overlay_root else {
        children.apply(current_widget.as_parent());
        return;
    };

    // Declaring zero children (both here and above) is what actually hides the window --
    // normal keyed reconciliation despawns the already-portaled `window_wrapper` (if any) the
    // moment it's no longer declared, exactly like `Modal`'s own `should_render` gate. Drag
    // position/etc. survive, since those live on `state_entity` (a `hooks.use_state` entity
    // tied to `current_widget`, not to the wrapper), which stays untouched either way.
    if !window.visible {
        children.apply(current_widget.as_parent());
        return;
    }

    let wrapper_style = WoodpeckerStyle {
        position: WidgetPosition::Fixed,
        left: state.position.x.into(),
        top: state.position.y.into(),
        // Offset from `StackingTier::Window` so a window's ordinal can never reach the next
        // tier (`Modal`), however many windows are open -- see `StackingTier`.
        z_index: Some(WidgetZ::Global(StackingTier::Window as u32 + z_index)),
        // The wrapper renders nothing visible itself -- `window_styles`'s background/border/
        // radius are instead applied split across the title bar and content wrapper below, so
        // there's only ever one border drawn in the scene.
        background_color: Color::NONE,
        border: Edge::all(0.0),
        box_shadow: Some(window_styles.box_shadow),
        ..window.window_styles
    };

    // The window's whole interior (title + content) is wrapped in a `Clip` masked to the
    // window's own rounded shape, so the title bar itself can stay a plain square-cornered box.
    let mut interior = WidgetChildren::default();
    interior
        // Title
        .add::<Element>((
            Element,
            WoodpeckerStyle {
                // Use the last-computed wrapper width only for `Auto`-sized windows, where
                // there's no definite parent width for `Percentage(100.0)` to resolve against.
                width: if window.window_styles.width == Units::Auto {
                    wrapper_size.x.into()
                } else {
                    Units::Percentage(100.0)
                },
                // Matches the window's own top corners directly, rather than relying solely on
                // the `Clip` wrapper below to mask a square-cornered title bar to the rounded
                // outer shape -- a border *stroke* (unlike a plain fill) can still visibly peek
                // past a clip mask at a sharp corner, since the stroke geometry extends outward
                // from the title bar's own (square) corner point, not the clip's curve.
                border_radius: Corner::all(0.0)
                    .top_left(window.window_styles.border_radius.top_left.value_or(0.0))
                    .top_right(window.window_styles.border_radius.top_right.value_or(0.0)),
                ..window.title_styles
            },
            if let Some(title_children) = title_children.as_ref() {
                title_children.0.clone()
            } else {
                WidgetChildren::default().with_child::<Element>((
                    Element,
                    WoodpeckerStyle {
                        font_size: Theme::default().font_size,
                        color: window_styles.title_text_color,
                        text_wrap: TextWrap::None,
                        ..Default::default()
                    },
                    WidgetRender::Text {
                        content: window.title.clone(),
                    },
                ))
            },
            WidgetRender::Quad,
            Pickable::default(),
        ));
    interior.add_key("title");
    interior
        .observe(
            current_widget,
            move |_trigger: On<Pointer<Press>>, mut context_query: Query<&mut WindowingContext>| {
                let Ok(mut context) = context_query.get_mut(context_entity) else {
                    return;
                };

                context.shift_to_top(current_widget.entity());
            },
        )
        .observe(
            current_widget,
            move |_trigger: On<Pointer<Over>>,
                  mut commands: Commands,
                  entity: Single<Entity, With<PrimaryWindow>>| {
                commands
                    .entity(*entity)
                    .insert(CursorIcon::from(SystemCursorIcon::Grab));
            },
        )
        .observe(
            current_widget,
            move |_trigger: On<Pointer<Out>>,
                  mut commands: Commands,
                  entity: Single<Entity, With<PrimaryWindow>>| {
                commands
                    .entity(*entity)
                    .insert(CursorIcon::from(SystemCursorIcon::Default));
            },
        )
        .observe(
            current_widget,
            move |_trigger: On<Pointer<DragEnd>>,
                  mut commands: Commands,
                  entity: Single<Entity, With<PrimaryWindow>>| {
                commands
                    .entity(*entity)
                    .insert(CursorIcon::from(SystemCursorIcon::Grab));
            },
        )
        .observe(
            current_widget,
            move |trigger: On<Pointer<DragStart>>,
                  mut commands: Commands,
                  mut state_query: Query<&mut WindowState>,
                  window: Single<Entity, With<PrimaryWindow>>,
                  pointer_world: PointerWorldPosition,
                  mut context_query: Query<&mut WindowingContext>| {
                let Ok(mut state) = state_query.get_mut(state_entity) else {
                    return;
                };

                let Some(cursor_pos_world) =
                    pointer_world.convert(trigger.pointer_location.position)
                else {
                    return;
                };

                state.drag_offset = state.position - cursor_pos_world;

                let Ok(mut context) = context_query.get_mut(context_entity) else {
                    return;
                };

                commands
                    .entity(*window)
                    .insert(CursorIcon::from(SystemCursorIcon::Grabbing));

                context.shift_to_top(current_widget.entity());
            },
        )
        .observe(
            current_widget,
            move |trigger: On<Pointer<Drag>>,
                  mut state_query: Query<&mut WindowState>,
                  pointer_world: PointerWorldPosition,
                  mut context_query: Query<&mut WindowingContext>,
                  layout_query: Query<&WidgetLayout>| {
                let Ok(mut state) = state_query.get_mut(state_entity) else {
                    return;
                };

                let Some(cursor_pos_world) =
                    pointer_world.convert(trigger.pointer_location.position)
                else {
                    return;
                };
                let Some(viewport_size) = pointer_world.target_size() else {
                    return;
                };

                let mut new_position = cursor_pos_world + state.drag_offset;

                // Keep the whole window within the viewport so it can't be dragged fully
                // off-screen with nothing left to grab. `wrapper_entity` (not
                // `current_widget.entity()`) since the window's real on-screen size lives on
                // its portaled wrapper now -- see the matching comment above this render fn.
                let window_size = wrapper_entity
                    .and_then(|e| layout_query.get(e).ok())
                    .map(|l| l.size)
                    .unwrap_or(Vec2::ZERO);
                new_position.x = new_position
                    .x
                    .clamp(0.0, (viewport_size.x - window_size.x).max(0.0));
                new_position.y = new_position
                    .y
                    .clamp(0.0, (viewport_size.y - window_size.y).max(0.0));

                state.position = new_position;

                let Ok(mut context) = context_query.get_mut(context_entity) else {
                    return;
                };

                context.shift_to_top(current_widget.entity());
            },
        )
        // Children -- carries the window's own background/border, bordered on the
        // left/right/bottom only (no top, since it butts directly against the title bar)
        // and rounded only at the bottom corners.
        .add::<Element>((
            Element,
            WoodpeckerStyle {
                background_color: window.window_styles.background_color,
                border_color: window.window_styles.border_color,
                border: Edge::all(0.0)
                    .left(window.window_styles.border.left.value_or(0.0))
                    .right(window.window_styles.border.right.value_or(0.0))
                    .bottom(window.window_styles.border.bottom.value_or(0.0)),
                border_radius: Corner::all(0.0)
                    .bottom_left(window.window_styles.border_radius.bottom_left.value_or(0.0))
                    .bottom_right(
                        window
                            .window_styles
                            .border_radius
                            .bottom_right
                            .value_or(0.0),
                    ),
                ..window.children_styles
            },
            WidgetRender::Quad,
            passed_children.0.clone(),
        ));
    interior.add_key("children");

    let mut wrapper_children = WidgetChildren::default();
    wrapper_children.add::<Clip>((
        Clip,
        WoodpeckerStyle {
            width: Units::Percentage(100.0),
            height: Units::Percentage(100.0),
            flex_direction: WidgetFlexDirection::Column,
            border_radius: window.window_styles.border_radius,
            ..Default::default()
        },
        interior,
    ));
    wrapper_children.add_key("interior");

    children.add::<Element>((
        Element,
        wrapper_style,
        WidgetRender::Quad,
        Pickable::default(),
        Focusable,
        // Ensures picking priority follows z-order across overlapping windows, not just
        // render order. See `Modal`/`Dropdown`/`Popover`, which require this for the same
        // reason.
        StackingContext,
        wrapper_children,
    ));
    children.add_key("window_wrapper");
    children.observe(
        current_widget,
        move |_trigger: On<WidgetFocus>, mut context_query: Query<&mut WindowingContext>| {
            let Ok(mut context) = context_query.get_mut(context_entity) else {
                return;
            };

            // Deliberately `current_widget.entity()`, not `trigger.target` (the wrapper,
            // since `Focusable` lives there now) -- `WindowingContext`'s z-order map is keyed
            // by this widget's own stable entity (see `get_or_add` above), the same identity
            // every other `shift_to_top` call in this file already uses.
            context.shift_to_top(current_widget.entity());
        },
    );
    children.portal_to(overlay_root.0);

    children.apply(current_widget.as_parent());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_theme_tracks_the_passed_in_theme() {
        let theme = Theme::dark();
        let styles = WindowStyles::from_theme(&theme);
        assert_eq!(styles.title_bar_background, theme.dark_background);
        assert_eq!(styles.body_background, theme.background);
        assert_eq!(styles.border_color, theme.border);
        assert_eq!(styles.title_text_color, theme.text);
        assert_eq!(styles.corner_radius, theme.panel_radius);
        assert_eq!(styles.box_shadow, theme.elevation.xl);

        let dark = WindowStyles::from_theme(&Theme::dark());
        let light = WindowStyles::from_theme(&Theme::light());
        assert_ne!(
            dark.body_background, light.body_background,
            "from_theme must track the passed-in theme, not a fixed default"
        );
    }

    #[test]
    fn default_woodpecker_window_style_fields_are_sourced_from_window_styles() {
        let window = WoodpeckerWindow::default();
        let window_styles = WindowStyles::default();
        assert_eq!(
            window.window_styles.background_color,
            window_styles.body_background
        );
        assert_eq!(
            window.window_styles.border_color,
            window_styles.border_color
        );
        assert_eq!(
            window.title_styles.background_color,
            window_styles.title_bar_background
        );
    }
}
