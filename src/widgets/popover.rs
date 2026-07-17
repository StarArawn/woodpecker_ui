use crate::prelude::*;
use bevy::prelude::*;

/// Which side of the trigger a [`Popover`]'s floating content appears on.
#[derive(Reflect, Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum PopoverPlacement {
    /// Directly below the trigger, left-edge aligned.
    #[default]
    Bottom,
    /// Directly above the trigger, left-edge aligned.
    Top,
    /// Directly to the left of the trigger, top-edge aligned.
    Left,
    /// Directly to the right of the trigger, top-edge aligned.
    Right,
}

/// A collection of styles for [`Popover`]'s floating content panel.
#[derive(Component, Reflect, Clone, Copy, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
pub struct PopoverStyles {
    /// The floating panel's background color.
    pub background_color: Color,
    /// The floating panel's border color.
    pub border_color: Color,
    /// The floating panel's corner radius.
    pub border_radius: f32,
    /// The floating panel's padding.
    pub padding: f32,
    /// The floating panel's drop shadow.
    pub box_shadow: WidgetBoxShadow,
}

impl Default for PopoverStyles {
    fn default() -> Self {
        Self::from_theme(&Theme::default())
    }
}

impl ThemedStyle for PopoverStyles {
    fn from_theme(theme: &Theme) -> Self {
        Self {
            background_color: theme.background,
            border_color: theme.border,
            border_radius: theme.control_radius,
            padding: 12.0,
            box_shadow: theme.elevation.md,
        }
    }
}

/// The floating content shown when a [`Popover`] is open.
///
/// Can't derive `Reflect` (it holds a [`WidgetChildren`], which in turn holds `Arc<dyn Fn>`
/// closures) so, like [`PassedChildren`], it gets its own special-cased diffing --
/// see [`diff_popover_content`], wired into [`crate::diffing::diff_widget_entity`].
#[derive(Component, Default, Clone, PartialEq)]
pub struct PopoverContent(pub WidgetChildren);

/// Snapshot of the last-diffed [`PopoverContent`] value for a widget entity -- mirrors
/// `children::PreviousPassedChildren`, but for the conditionally-visible floating content
/// rather than the always-visible trigger.
#[derive(Component, Default)]
struct PreviousPopoverContent(Option<PopoverContent>);

/// Compares `entity`'s current [`PopoverContent`] against its last snapshot, returning
/// `true` (and updating the snapshot) if it changed.
pub(crate) fn diff_popover_content(world: &mut World, entity: Entity) -> bool {
    let Some(current) = world.get::<PopoverContent>(entity) else {
        return false;
    };
    let current = current.clone();

    let changed = match world.get::<PreviousPopoverContent>(entity) {
        Some(previous) => previous.0.as_ref() != Some(&current),
        None => true,
    };

    if changed {
        world
            .entity_mut(entity)
            .insert(PreviousPopoverContent(Some(current)));
    }

    changed
}

/// `Popover`'s own default `Transition` -- opacity-only, not yet started (`playing: false`).
/// `render` overwrites `style_a`/`style_b` every call with the freshly anchored position --
/// unlike `Modal`'s fixed/centered box, `Popover`'s floating content is positioned relative to
/// a trigger that can itself move.
fn popover_transition() -> Transition {
    Transition {
        easing: TransitionEasing::CubicOut,
        timeout: 150.0,
        looping: false,
        playing: false,
        ..Default::default()
    }
}

/// Tracks whether this popover was visible last render, so a `visible` flip can kick off
/// `transition.start()`/`start_reverse()` -- mirrors `ModalState`/`DrawerState`'s identical
/// "own state drives the transition" pattern.
#[derive(Component, Reflect, PartialEq, Clone, Copy, Default)]
#[reflect(Component, DiffableProp, PartialEq)]
struct PopoverState {
    previous_visible: bool,
}

/// A generic floating-content primitive: renders `trigger` (always) plus `content`
/// (absolutely positioned next to the trigger, only while `visible`). A controlled
/// component -- the caller owns `visible` and toggles it themselves.
#[derive(Widget, Component, Reflect, Clone, PartialEq, Default)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(
    WoodpeckerStyle,
    PassedChildren,
    PopoverContent,
    WidgetChildren,
    PopoverStyles,
    Transition = popover_transition()
)]
pub struct Popover {
    /// Whether the floating content is currently shown.
    pub visible: bool,
    /// Which side of the trigger the content appears on.
    pub placement: PopoverPlacement,
}

/// A bundle for creating a [`Popover`].
#[derive(Bundle, Clone, Default)]
pub struct PopoverBundle {
    /// The popover itself.
    pub popover: Popover,
    /// The always-visible trigger content.
    pub trigger: PassedChildren,
    /// The floating content, shown only while `popover.visible`.
    pub content: PopoverContent,
    /// Styles for the floating content panel.
    pub styles: PopoverStyles,
    /// Internal styles for the popover's own (trigger-sized) box.
    pub internal_styles: WoodpeckerStyle,
    /// Internal children (populated by `render`).
    pub internal_children: WidgetChildren,
}

fn render(
    mut commands: Commands,
    mut hooks: ResMut<HookHelper>,
    current_widget: Res<CurrentWidget>,
    widget_mapper: Res<WidgetMapper>,
    // `Option`, not a hard `Res`: a sibling of `OverlayRootWidget` in the same initial tree can
    // in principle render before `sync_overlay_root` has observed it -- see that system's doc
    // comment. Skipping a frame beats panicking on a legitimate startup-order race.
    overlay_root: Option<Res<OverlayRoot>>,
    mut query: Query<(
        &Popover,
        &PopoverStyles,
        &PassedChildren,
        &PopoverContent,
        &mut WidgetChildren,
        &WidgetLayout,
        &mut Transition,
    )>,
    layout_query: Query<&WidgetLayout>,
    mut popover_state: Query<&mut PopoverState>,
) {
    let Ok((popover, popover_styles, trigger, content, mut children, layout, mut transition)) =
        query.get_mut(**current_widget)
    else {
        return;
    };
    // `content` (below) was originally `position: Absolute`, a SIBLING of "trigger" -- both
    // direct children of Popover's own root -- so it resolved against Popover's own root box,
    // not "trigger"'s own box (they only coincide when "trigger" is Popover's only in-flow
    // child *and* happens to hug-content to the same size, which `DatePicker` breaks: it
    // leaves `PopoverBundle::trigger` empty and instead sizes Popover's own root directly via
    // `internal_styles`). So this reads Popover's own `WidgetLayout` directly -- synchronously
    // available, no `WidgetMapper` lag needed -- exactly like `Dropdown`/`ComboBox` already do
    // for their own trigger.
    let (trigger_loc, trigger_size) = (layout.location, layout.size);

    let state_entity = hooks.use_state(&mut commands, *current_widget, PopoverState::default());
    let Ok(mut state) = popover_state.get_mut(state_entity) else {
        return;
    };

    *transition = Transition {
        reversing: !popover.visible,
        timeout: 150.0,
        ..*transition
    };
    if state.previous_visible != popover.visible {
        if transition.reversing {
            transition.start_reverse();
        } else {
            transition.start();
        }
        state.previous_visible = popover.visible;
    }

    *children = WidgetChildren::default();

    children.add::<Element>((Element, WoodpeckerStyle::default(), trigger.0.clone()));
    children.add_key("trigger");

    let should_render = (transition.is_playing() || popover.visible) && overlay_root.is_some();
    if should_render {
        let overlay_root = overlay_root.unwrap();
        // This widget can't portal itself (only whoever *declares* a child can mark it
        // `.portal()`-ed), so `content` is a synthetic child portaled to `OverlayRoot` --
        // meaning it can no longer be `Absolute`-positioned against its own now-portaled-
        // away parent box (Popover's own root, per the comment above). It switches to
        // `Fixed` with explicit `left`/`top` computed from that same box's `WidgetLayout`
        // instead. For `Top`/`Left`, which need to back off by the content's own size, that
        // size is read one-frame-lagged via `WidgetMapper::get_child`, since this frame's
        // layout hasn't run yet -- the same lag `devtools/highlight.rs` already accepts
        // elsewhere in this crate. Settles within 1-2 frames of first becoming visible.
        let content_size = widget_mapper
            .get_keyed_child::<Element>(current_widget.as_parent(), "content")
            .and_then(|e| layout_query.get(e).ok())
            .map(|l| l.size)
            .unwrap_or_default();

        let mut position = WoodpeckerStyle {
            position: WidgetPosition::Fixed,
            background_color: popover_styles.background_color,
            border_color: popover_styles.border_color,
            border: Edge::all(1.0),
            border_radius: Corner::all(popover_styles.border_radius),
            padding: Edge::all(popover_styles.padding),
            box_shadow: Some(popover_styles.box_shadow),
            z_index: Some(WidgetZ::Global(StackingTier::Popover as u32)),
            opacity: 1.0,
            ..Default::default()
        };
        match popover.placement {
            PopoverPlacement::Bottom => {
                position.left = trigger_loc.x.into();
                position.top = (trigger_loc.y + trigger_size.y).into();
            }
            PopoverPlacement::Top => {
                position.left = trigger_loc.x.into();
                position.top = (trigger_loc.y - content_size.y).into();
            }
            PopoverPlacement::Left => {
                position.left = (trigger_loc.x - content_size.x).into();
                position.top = trigger_loc.y.into();
            }
            PopoverPlacement::Right => {
                position.left = (trigger_loc.x + trigger_size.x).into();
                position.top = trigger_loc.y.into();
            }
        }

        const SLIDE_OFFSET: f32 = 8.0;
        let mut style_a = position;
        style_a.opacity = 0.0;
        match popover.placement {
            PopoverPlacement::Bottom => {
                style_a.top = (trigger_loc.y + trigger_size.y - SLIDE_OFFSET).into();
            }
            PopoverPlacement::Top => {
                style_a.top = (trigger_loc.y - content_size.y + SLIDE_OFFSET).into();
            }
            PopoverPlacement::Left => {
                style_a.left = (trigger_loc.x - content_size.x + SLIDE_OFFSET).into();
            }
            PopoverPlacement::Right => {
                style_a.left = (trigger_loc.x + trigger_size.x - SLIDE_OFFSET).into();
            }
        }
        let content_transition = Transition {
            style_a,
            style_b: position,
            ..*transition
        };

        children.add::<Element>((
            Element,
            position,
            content.0.clone(),
            WidgetRender::Quad,
            content_transition,
            // Without this, the floating content's high `z_index` renders it on top but
            // interactive elements inside it still compete for clicks at the ambient
            // picking depth. See `Dropdown`.
            StackingContext,
        ));
        children.add_key("content");
        children.portal_to(overlay_root.0);
    }

    children.apply(current_widget.as_parent());
}
