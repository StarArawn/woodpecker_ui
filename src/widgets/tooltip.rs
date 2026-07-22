use crate::{prelude::*, widgets::popover::PopoverPlacement};
use bevy::prelude::*;

/// A collection of styles for [`Tooltip`]'s floating label.
#[derive(Component, Reflect, Clone, Copy, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
pub struct TooltipStyles {
    /// The label's background color.
    pub background_color: Color,
    /// The label's text color.
    pub text_color: Color,
    /// The label's corner radius.
    pub border_radius: f32,
    /// The label's font size.
    pub font_size: f32,
    /// The label's drop shadow.
    pub box_shadow: WidgetBoxShadow,
}

impl Default for TooltipStyles {
    fn default() -> Self {
        Self::from_theme(&Theme::default())
    }
}

impl ThemedStyle for TooltipStyles {
    fn from_theme(theme: &Theme) -> Self {
        Self {
            background_color: theme.dark_background,
            text_color: theme.text,
            border_radius: 6.0,
            font_size: 12.0,
            box_shadow: theme.elevation.sm,
        }
    }
}

/// Per-widget hover state.
#[derive(Component, Debug, Reflect, Clone, Copy, PartialEq, Default)]
#[reflect(Component, DiffableProp, PartialEq)]
struct TooltipState {
    hovering: bool,
    /// `hovering` as of the last render -- see `DropdownState::previous_open`'s doc comment
    /// for why this is needed (mirrors the identical pattern there).
    previous_hovering: bool,
}

/// The label's open/close `Transition` preset -- quick even by this crate's other overlay
/// panels' standards (tooltips read as "immediate" with a touch of polish, not as a deliberate
/// reveal), not yet started.
fn label_transition() -> Transition {
    Transition {
        easing: TransitionEasing::CubicOut,
        timeout: 80.0,
        looping: false,
        playing: false,
        ..Default::default()
    }
}

/// Shows a short text label near its trigger content while hovered.
///
/// The label is plain text only; for a floating panel with arbitrary content, use
/// [`crate::widgets::Popover`] instead.
#[derive(Widget, Component, Reflect, Clone, PartialEq, Default)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(WoodpeckerStyle, PassedChildren, WidgetChildren, TooltipStyles, Transition = label_transition())]
pub struct Tooltip {
    /// The label text. A tooltip with empty text never shows a label.
    pub text: String,
    /// Which side of the trigger the label appears on.
    pub placement: PopoverPlacement,
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
        &Tooltip,
        &TooltipStyles,
        &WoodpeckerStyle,
        &PassedChildren,
        &mut WidgetChildren,
        &WidgetLayout,
        &mut Transition,
    )>,
    mut state_query: Query<&mut TooltipState>,
    layout_query: Query<(&WidgetLayout, Has<WoodpeckerApp>)>,
) {
    let Ok((tooltip, tooltip_styles, own_style, trigger, mut children, layout, mut transition)) =
        query.get_mut(**current_widget)
    else {
        return;
    };
    // `label` (below) was originally `position: Absolute`, a SIBLING of "trigger" -- both
    // direct children of Tooltip's own root -- so it resolved against Tooltip's own root box,
    // not "trigger"'s own box. They coincide for typical usage (Tooltip's own root has zero
    // padding and Auto-hugs around "trigger" as its only in-flow child), but reading Tooltip's
    // own `WidgetLayout` directly is both simpler and exactly correct, matching what
    // `Absolute` actually resolved against -- see the identical fix (and the concrete case
    // where they diverge) in `popover.rs::render`.
    let (trigger_loc, trigger_size) = (layout.location, layout.size);

    let state_entity = hooks.use_state(&mut commands, *current_widget, TooltipState::default());
    let Ok(mut state) = state_query.get_mut(state_entity) else {
        return;
    };

    *transition = Transition {
        reversing: !state.hovering,
        timeout: 80.0,
        ..*transition
    };
    if state.previous_hovering != state.hovering {
        if transition.reversing {
            transition.start_reverse();
        } else {
            transition.start();
        }
        state.previous_hovering = state.hovering;
    }

    *children = WidgetChildren::default();
    let current_widget_val = *current_widget;

    children.add::<Element>((
        Element,
        // `width`/`height`/`flex_grow`/`flex_basis` copied from Tooltip's *own* declared style
        // (whatever the caller actually set there), not `WoodpeckerStyle::default()`
        // (`Auto`/`Auto`) -- trigger content that itself wants `width: 100%`/`height: 100%`
        // (e.g. a bar chart segment sized to fill its cell) has nothing to resolve that
        // percentage against inside an `Auto`-sized wrapper and collapses to nothing.
        //
        // Deliberately propagates the *declared* sizing intent, not Tooltip's last-resolved
        // pixel size (`WidgetLayout`) -- that was this fix's first attempt, and it introduces a
        // feedback loop for the common `Auto`-sized case (any ordinary inline trigger, e.g. this
        // crate's own chart legend rows): Tooltip's own box is `Auto`-sized *from* trigger's
        // natural content size, so pinning trigger's size *to* Tooltip's last-resolved size
        // means each frame's (possibly-too-small, e.g. on first mount) result becomes the next
        // frame's hard constraint, with nothing ever able to grow back out of an initial
        // undersized measurement -- confirmed via a live repro, garbled/overlapping legend text
        // that never recovered. Propagating the declared value instead of the resolved one
        // sidesteps that entirely: `Auto` copied onto trigger is a no-op (identical to the
        // original, pre-fix behavior), while `Percentage`/`Pixels`/`flex_grow` copied onto
        // trigger resolves against Tooltip's own box the normal way flexbox always has -- and
        // that box's own size came from *its* parent, never circularly from trigger itself.
        WoodpeckerStyle {
            width: own_style.width,
            height: own_style.height,
            flex_grow: own_style.flex_grow,
            flex_shrink: own_style.flex_shrink,
            flex_basis: own_style.flex_basis,
            ..Default::default()
        },
        Pickable::default(),
        trigger.0.clone(),
    ));
    children.observe(
        current_widget_val,
        move |_trigger: On<Pointer<Over>>, mut state_query: Query<&mut TooltipState>| {
            let Ok(mut state) = state_query.get_mut(state_entity) else {
                return;
            };
            state.hovering = true;
        },
    );
    children.observe(
        current_widget_val,
        move |_trigger: On<Pointer<Out>>, mut state_query: Query<&mut TooltipState>| {
            let Ok(mut state) = state_query.get_mut(state_entity) else {
                return;
            };
            state.hovering = false;
        },
    );
    children.add_key("trigger");

    if let Some(overlay_root) = overlay_root {
        if (transition.is_playing() || state.hovering) && !tooltip.text.is_empty() {
            // This widget can't portal itself (only whoever *declares* a child can mark it
            // `.portal()`-ed), so the label is a synthetic child portaled to `OverlayRoot` --
            // meaning it can no longer be `Absolute`-positioned against its own now-portaled-
            // away parent box (Tooltip's own root, per the comment above). It switches to
            // `Fixed` with explicit `left`/`top` computed from that same box's `WidgetLayout`
            // instead. For `Top`/`Left`, which need to back off by the label's own size, that
            // size is read one-frame-lagged via `WidgetMapper::get_child`, since this frame's
            // layout hasn't run yet -- the same lag `devtools/highlight.rs` already accepts
            // elsewhere in this crate. Settles within 1-2 frames of first becoming visible.
            let label_size = widget_mapper
                .get_keyed_child::<Element>(current_widget.as_parent(), "label")
                .and_then(|e| layout_query.get(e).ok())
                .map(|(l, _)| l.size)
                .unwrap_or_default();

            let viewport_height = layout_query
                .iter()
                .find(|(_, is_root)| *is_root)
                .map(|(l, _)| l.size.y)
                .unwrap_or(f32::MAX);

            let effective_placement = match tooltip.placement {
                PopoverPlacement::Bottom
                    if super::popover::should_open_upward(
                        trigger_loc.y,
                        trigger_size.y,
                        label_size.y,
                        viewport_height,
                    ) =>
                {
                    PopoverPlacement::Top
                }
                other => other,
            };

            let mut position = WoodpeckerStyle {
                position: WidgetPosition::Fixed,
                background_color: tooltip_styles.background_color,
                border_radius: Corner::all(tooltip_styles.border_radius),
                padding: Edge::all(0.0).left(8.0).right(8.0).top(4.0).bottom(4.0),
                box_shadow: Some(tooltip_styles.box_shadow),
                z_index: Some(WidgetZ::Global(StackingTier::Tooltip as u32)),
                opacity: 1.0,
                ..Default::default()
            };
            match effective_placement {
                PopoverPlacement::Bottom => {
                    position.left = trigger_loc.x.into();
                    position.top = (trigger_loc.y + trigger_size.y).into();
                }
                PopoverPlacement::Top => {
                    position.left = trigger_loc.x.into();
                    position.top = (trigger_loc.y - label_size.y).into();
                }
                PopoverPlacement::Left => {
                    position.left = (trigger_loc.x - label_size.x).into();
                    position.top = trigger_loc.y.into();
                }
                PopoverPlacement::Right => {
                    position.left = (trigger_loc.x + trigger_size.x).into();
                    position.top = trigger_loc.y.into();
                }
            }

            const SLIDE_OFFSET: f32 = 4.0;
            let mut style_a = position;
            style_a.opacity = 0.0;
            match effective_placement {
                PopoverPlacement::Bottom => {
                    style_a.top = (trigger_loc.y + trigger_size.y - SLIDE_OFFSET).into();
                }
                PopoverPlacement::Top => {
                    style_a.top = (trigger_loc.y - label_size.y + SLIDE_OFFSET).into();
                }
                PopoverPlacement::Left => {
                    style_a.left = (trigger_loc.x - label_size.x + SLIDE_OFFSET).into();
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
                WidgetChildren::default().with_child::<Element>((
                    Element,
                    WoodpeckerStyle {
                        font_size: tooltip_styles.font_size,
                        color: tooltip_styles.text_color,
                        text_wrap: TextWrap::None,
                        ..Default::default()
                    },
                    WidgetRender::Text {
                        content: tooltip.text.clone(),
                    },
                )),
                WidgetRender::Quad,
                content_transition,
            ));
            children.add_key("label");
            children.portal_to(overlay_root.0);
        }
    }

    children.apply(current_widget.as_parent());
}
