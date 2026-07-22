use crate::prelude::*;
use bevy::prelude::*;

/// Which edge of the screen a [`Drawer`] slides in from.
#[derive(Reflect, Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum DrawerPosition {
    /// Slides in from the left edge.
    #[default]
    Left,
    /// Slides in from the right edge.
    Right,
    /// Slides in from the top edge.
    Top,
    /// Slides in from the bottom edge.
    Bottom,
}

/// [`Drawer`]'s two behavior modes.
#[derive(Reflect, Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum DrawerVariant {
    /// Portaled overlay with a scrim, opened/closed via `Drawer::open`, sliding in with a
    /// `Spring`. Fires [`Change<DrawerCloseRequested>`] when the scrim is clicked -- like
    /// [`Modal`](super::Modal), `Drawer` doesn't own `open` itself, so it can't close on its
    /// own; the caller's own observer sets `open: false` on the next render in response.
    #[default]
    Temporary,
    /// Renders inline, in normal layout flow, always visible -- no scrim, no portal, no
    /// transition, and `Drawer::open` is ignored. For an always-present navigation column
    /// (generalizes `examples/dashboard/sidebar.rs`'s previously hand-built nav pattern).
    Persistent,
}

/// Fired when a `Temporary` [`Drawer`]'s scrim is clicked.
#[derive(Reflect, Clone, PartialEq, Default, Debug)]
pub struct DrawerCloseRequested;

/// [`Drawer`]'s themed colors -- a separate sibling component so it live-resyncs on a
/// [`Theme`] swap.
#[derive(Component, Reflect, Clone, Copy, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
pub struct DrawerStyles {
    /// Drawer surface background color.
    pub background_color: Color,
    /// Drawer surface border color (the edge facing the rest of the app).
    pub border_color: Color,
    /// `Temporary` variant's scrim color.
    pub scrim_color: Color,
    /// `Temporary` variant's panel drop shadow, cast toward the content it slides over --
    /// `render` flips its offset to match `Drawer::position`.
    pub box_shadow: WidgetBoxShadow,
}

impl Default for DrawerStyles {
    fn default() -> Self {
        Self::from_theme(&Theme::default())
    }
}

impl ThemedStyle for DrawerStyles {
    fn from_theme(theme: &Theme) -> Self {
        Self {
            background_color: theme.background,
            border_color: theme.border,
            scrim_color: Srgba::new(0.0, 0.0, 0.0, 0.6).into(),
            box_shadow: theme.elevation.lg,
        }
    }
}

/// A [`Paper`](super::Paper)-adjacent surface that slides in from an edge of the screen
/// (`Temporary`) or sits permanently in normal layout flow (`Persistent`) -- see
/// [`DrawerVariant`]. Generalizes `examples/dashboard/sidebar.rs`'s hand-built nav column
/// (`Persistent`) and adds the toggleable overlay case MUI's `Drawer` also covers
/// (`Temporary`), which nothing in this crate had before.
///
/// The slide is driven by a [`Spring`] rather than a [`Transition`]: retargeting mid-slide
/// (rapidly toggling `open`) continues smoothly from wherever the panel currently is instead
/// of snapping to an endpoint first -- see [`SpringStyle`]'s doc comment for why.
#[derive(Widget, Component, Reflect, Clone, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(WoodpeckerStyle, PassedChildren, WidgetChildren, DrawerStyles, Spring)]
pub struct Drawer {
    /// Whether the drawer is open. Ignored by the `Persistent` variant.
    pub open: bool,
    /// Which edge the drawer slides in from (or sits against, for `Persistent`).
    pub position: DrawerPosition,
    /// `Temporary` (overlay, scrim, animated) or `Persistent` (inline, always visible).
    pub variant: DrawerVariant,
    /// Width (for `Left`/`Right`) or height (for `Top`/`Bottom`), in pixels.
    pub size: f32,
    /// The driving spring's stiffness -- see [`Spring::stiffness`]. Defaults to a snappier,
    /// critically damped ~100ms settle rather than [`Spring::default`]'s own pointer-follow
    /// tuning. Ignored by the `Persistent` variant.
    pub stiffness: f32,
    /// The driving spring's damping -- see [`Spring::damping`]. Ignored by the `Persistent`
    /// variant.
    pub damping: f32,
}

impl Default for Drawer {
    fn default() -> Self {
        Self {
            open: false,
            position: DrawerPosition::default(),
            variant: DrawerVariant::default(),
            size: 280.0,
            stiffness: 2000.0,
            damping: 89.44,
        }
    }
}

/// The offscreen/onscreen `left`/`right`/`top`/`bottom` pair for sliding a `size`-wide/tall
/// panel in from `position`.
fn slide_styles(position: DrawerPosition, size: f32) -> (WoodpeckerStyle, WoodpeckerStyle) {
    let base = WoodpeckerStyle {
        position: WidgetPosition::Fixed,
        ..Default::default()
    };
    match position {
        DrawerPosition::Left => (
            WoodpeckerStyle {
                left: (-size).into(),
                top: 0.0.into(),
                width: size.into(),
                height: Units::Percentage(100.0),
                ..base
            },
            WoodpeckerStyle {
                left: 0.0.into(),
                top: 0.0.into(),
                width: size.into(),
                height: Units::Percentage(100.0),
                ..base
            },
        ),
        DrawerPosition::Right => (
            WoodpeckerStyle {
                right: (-size).into(),
                top: 0.0.into(),
                width: size.into(),
                height: Units::Percentage(100.0),
                ..base
            },
            WoodpeckerStyle {
                right: 0.0.into(),
                top: 0.0.into(),
                width: size.into(),
                height: Units::Percentage(100.0),
                ..base
            },
        ),
        DrawerPosition::Top => (
            WoodpeckerStyle {
                top: (-size).into(),
                left: 0.0.into(),
                height: size.into(),
                width: Units::Percentage(100.0),
                ..base
            },
            WoodpeckerStyle {
                top: 0.0.into(),
                left: 0.0.into(),
                height: size.into(),
                width: Units::Percentage(100.0),
                ..base
            },
        ),
        DrawerPosition::Bottom => (
            WoodpeckerStyle {
                bottom: (-size).into(),
                left: 0.0.into(),
                height: size.into(),
                width: Units::Percentage(100.0),
                ..base
            },
            WoodpeckerStyle {
                bottom: 0.0.into(),
                left: 0.0.into(),
                height: size.into(),
                width: Units::Percentage(100.0),
                ..base
            },
        ),
    }
}

/// `base` recast toward the content `position`'s panel sits against -- e.g. a `Left` drawer's
/// content sits to its right, so the shadow's offset needs to point right (+x), not down
/// (`base`'s own fixed offset axis, which only matters for its magnitude here).
fn directional_shadow(base: WidgetBoxShadow, position: DrawerPosition) -> WidgetBoxShadow {
    let magnitude = base.y_offset.abs().max(base.x_offset.abs());
    WidgetBoxShadow {
        x_offset: match position {
            DrawerPosition::Left => magnitude,
            DrawerPosition::Right => -magnitude,
            DrawerPosition::Top | DrawerPosition::Bottom => 0.0,
        },
        y_offset: match position {
            DrawerPosition::Top => magnitude,
            DrawerPosition::Bottom => -magnitude,
            DrawerPosition::Left | DrawerPosition::Right => 0.0,
        },
        ..base
    }
}

fn render(
    current_widget: Res<CurrentWidget>,
    overlay_root: Option<Res<OverlayRoot>>,
    mut query: Query<(
        &Drawer,
        &DrawerStyles,
        &mut WidgetChildren,
        &PassedChildren,
        &mut Spring,
    )>,
) {
    let Ok((drawer, drawer_styles, mut children, passed_children, mut spring)) =
        query.get_mut(**current_widget)
    else {
        return;
    };

    let current_widget = *current_widget;

    if drawer.variant == DrawerVariant::Persistent {
        *children = WidgetChildren::default().with_child::<Element>((
            Element,
            WoodpeckerStyle {
                width: Units::Percentage(100.0),
                height: Units::Percentage(100.0),
                background_color: drawer_styles.background_color,
                border_color: drawer_styles.border_color,
                border: (match drawer.position {
                    DrawerPosition::Left => Edge::all(0.0).right(1.0),
                    DrawerPosition::Right => Edge::all(0.0).left(1.0),
                    DrawerPosition::Top => Edge::all(0.0).bottom(1.0),
                    DrawerPosition::Bottom => Edge::all(0.0).top(1.0),
                }),
                flex_direction: WidgetFlexDirection::Column,
                box_shadow: Some(directional_shadow(drawer_styles.box_shadow, drawer.position)),
                ..Default::default()
            },
            WidgetRender::Quad,
            passed_children.0.clone(),
        ));
        children.add_key("panel");
        children.apply(current_widget.as_parent());
        return;
    }

    spring.stiffness = drawer.stiffness;
    spring.damping = drawer.damping;
    spring.set_target(Vec2::splat(if drawer.open { 1.0 } else { 0.0 }));

    *children = WidgetChildren::default();

    let should_render = (!spring.is_settled() || drawer.open) && overlay_root.is_some();
    if !should_render {
        children.apply(current_widget.as_parent());
        return;
    }
    let overlay_root = overlay_root.unwrap();

    let scrim_style_b = WoodpeckerStyle {
        background_color: drawer_styles.scrim_color,
        width: Units::Percentage(100.0),
        height: Units::Percentage(100.0),
        position: WidgetPosition::Absolute,
        opacity: 1.0,
        ..Default::default()
    };
    let scrim_style_a = WoodpeckerStyle {
        opacity: 0.0,
        ..scrim_style_b
    };
    let scrim_baseline = scrim_style_a.lerp(&scrim_style_b, spring.value.x.clamp(0.0, 1.0));

    let mut portal_children = WidgetChildren::default();
    portal_children.add::<Element>((
        Element,
        scrim_baseline,
        *spring,
        SpringStyle {
            style_a: scrim_style_a,
            style_b: scrim_style_b,
        },
        Pickable::default(),
        WidgetRender::Quad,
    ));
    portal_children.add_key("scrim");
    portal_children
        .observe(current_widget, |mut trigger: On<Pointer<Over>>| {
            trigger.propagate(false);
        })
        .observe(current_widget, |mut trigger: On<Pointer<Out>>| {
            trigger.propagate(false);
        })
        .observe(
            current_widget,
            move |mut trigger: On<Pointer<Click>>, mut commands: Commands| {
                trigger.propagate(false);
                commands.trigger(Change {
                    target: current_widget.entity(),
                    data: DrawerCloseRequested,
                });
            },
        );

    let (mut style_a, mut style_b) = slide_styles(drawer.position, drawer.size);
    let panel_border = match drawer.position {
        DrawerPosition::Left => Edge::all(0.0).right(1.0),
        DrawerPosition::Right => Edge::all(0.0).left(1.0),
        DrawerPosition::Top => Edge::all(0.0).bottom(1.0),
        DrawerPosition::Bottom => Edge::all(0.0).top(1.0),
    };
    let panel_shadow = directional_shadow(drawer_styles.box_shadow, drawer.position);
    // Colors deliberately identical on both ends -- only position slides, the panel doesn't
    // also fade (`SpringStyle` would otherwise lerp `background_color`/`border_color` too,
    // since neither end sets them and both default to `WoodpeckerStyle::DEFAULT`'s
    // transparent black).
    for style in [&mut style_a, &mut style_b] {
        style.background_color = drawer_styles.background_color;
        style.border_color = drawer_styles.border_color;
        style.border = panel_border;
        style.flex_direction = WidgetFlexDirection::Column;
        style.box_shadow = Some(panel_shadow);
    }
    let baseline = style_a.lerp(&style_b, spring.value.x.clamp(0.0, 1.0));
    let panel_spring_style = SpringStyle { style_a, style_b };
    portal_children.add::<Element>((
        Element,
        baseline,
        *spring,
        panel_spring_style,
        FocusTrapRoot,
        WidgetRender::Quad,
        passed_children.0.clone(),
    ));
    portal_children.add_key("panel");

    children.add::<Element>((
        Element,
        WoodpeckerStyle {
            width: Units::Percentage(100.0),
            height: Units::Percentage(100.0),
            position: WidgetPosition::Fixed,
            z_index: Some(WidgetZ::Global(StackingTier::Drawer as u32)),
            ..Default::default()
        },
        WidgetRender::Layer,
        StackingContext,
        portal_children,
    ));
    children.add_key("portal_wrapper");
    children.portal_to(overlay_root.0);

    children.apply(current_widget.as_parent());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_theme_tracks_the_passed_in_theme() {
        let theme = Theme::dark();
        let styles = DrawerStyles::from_theme(&theme);
        assert_eq!(styles.background_color, theme.background);
        assert_eq!(styles.border_color, theme.border);
        assert_eq!(styles.box_shadow, theme.elevation.lg);

        let dark = DrawerStyles::from_theme(&Theme::dark());
        let light = DrawerStyles::from_theme(&Theme::light());
        assert_ne!(
            dark.background_color, light.background_color,
            "from_theme must track the passed-in theme, not a fixed default"
        );
    }

    #[test]
    fn directional_shadow_points_toward_the_content_each_position_sits_against() {
        let base = Theme::dark().elevation.lg;
        let magnitude = base.y_offset.abs();

        let left = directional_shadow(base, DrawerPosition::Left);
        assert_eq!(left.x_offset, magnitude);
        assert_eq!(left.y_offset, 0.0);

        let right = directional_shadow(base, DrawerPosition::Right);
        assert_eq!(right.x_offset, -magnitude);
        assert_eq!(right.y_offset, 0.0);

        let top = directional_shadow(base, DrawerPosition::Top);
        assert_eq!(top.x_offset, 0.0);
        assert_eq!(top.y_offset, magnitude);

        let bottom = directional_shadow(base, DrawerPosition::Bottom);
        assert_eq!(bottom.x_offset, 0.0);
        assert_eq!(bottom.y_offset, -magnitude);
    }

    #[test]
    fn slide_styles_start_offscreen_by_exactly_size_and_end_flush() {
        let (a, b) = slide_styles(DrawerPosition::Left, 280.0);
        assert_eq!(a.left, Units::Pixels(-280.0));
        assert_eq!(b.left, Units::Pixels(0.0));

        let (a, b) = slide_styles(DrawerPosition::Right, 280.0);
        assert_eq!(a.right, Units::Pixels(-280.0));
        assert_eq!(b.right, Units::Pixels(0.0));

        let (a, b) = slide_styles(DrawerPosition::Top, 200.0);
        assert_eq!(a.top, Units::Pixels(-200.0));
        assert_eq!(b.top, Units::Pixels(0.0));

        let (a, b) = slide_styles(DrawerPosition::Bottom, 200.0);
        assert_eq!(a.bottom, Units::Pixels(-200.0));
        assert_eq!(b.bottom, Units::Pixels(0.0));
    }
}
