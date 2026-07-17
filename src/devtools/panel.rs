use crate::{prelude::*, widgets::colors};
use bevy::prelude::*;

use super::{
    metrics_readout::MetricsTextMarker, style_editor, DevtoolsHover, DevtoolsSnapshot,
    DevtoolsState,
};

const PANEL_WIDTH: f32 = 460.0;
const TREE_HEIGHT: f32 = 220.0;

fn default_style() -> WoodpeckerStyle {
    WoodpeckerStyle {
        position: WidgetPosition::Fixed,
        top: 0.0.into(),
        right: 0.0.into(),
        height: Units::Percentage(100.0),
        // `StackingTier::Devtools` sits above every app-content tier including `Toast`, so
        // the panel stays usable regardless of what the inspected app itself has open. This
        // panel used to sit at a deliberately low z (100) instead, as a workaround for a
        // since-fixed f32 precision bug in `picking_backend.rs`'s depth encoding (pick depth
        // is now a monotonic integer rank, immune to z magnitude -- see
        // `picking_backend::rank_hits`), so the workaround is no longer needed.
        z_index: Some(WidgetZ::Global(StackingTier::Devtools as u32)),
        ..Default::default()
    }
}

/// The devtools panel's own root widget. **Must be declared explicitly by the app**, exactly
/// once, alongside its real content, e.g.:
///
/// ```rust,ignore
/// WidgetChildren::default()
///     .with_child::<MyAppRoot>(MyAppRoot)
///     .with_child::<DevtoolsRoot>(DevtoolsRoot)
/// ```
///
/// `WoodpeckerDevtoolsPlugin` cannot add this child for you: `WoodpeckerApp::render` calls
/// `children.apply()` on every window resize, which reconciles the app root's *entire*
/// declared child set -- a child injected any other way (e.g. a raw
/// `commands.spawn(..).insert(ChildOf(root))`) would be silently despawned the next time the
/// window resizes, since it was never part of that declared set. This is the same reason
/// `ToastViewport`/`WindowingContextProvider` are opt-in children too, not auto-injected by
/// their own plugins.
///
/// Renders nothing while closed (`DevtoolsState::open == false`), and is completely inert
/// until [`super::WoodpeckerDevtoolsPlugin`] is added.
///
/// Deliberately **not** `.portal()`-ed, unlike `Modal`/`Toast`/`Dropdown`/etc. Those widgets
/// need portaling because they're typically declared deep inside some clipped/scrolled/z-
/// ambient part of the app; `DevtoolsRoot` isn't -- its own doc comment above already requires
/// it be declared as a direct sibling of the app's own root, and its `StackingTier::Devtools`
/// tier is the highest of any built-in, so it already wins the root-level sibling bucket sort
/// (see `layout::system::order_children_for_paint`) against everything else declared there,
/// portaled or not -- confirmed empirically: opening it over a live `Toast` still ranks it
/// above the toast's own (portaled, `OverlayRoot`-parented) content. Portaling it anyway would
/// only add a mandatory `OverlayRootWidget` dependency for apps that use devtools but none of
/// the portaled built-ins, without fixing anything. `SkipPortalLint` (below) tells the dev-build
/// lint this is an intentional, understood exception rather than a leftover pre-portal-migration
/// escape hatch.
#[derive(Widget, Component, Reflect, Clone, PartialEq, Default)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(
    WoodpeckerStyle = default_style(),
    WidgetChildren,
    // Establishes a new stacking tier for its own descendants -- see `default_style`'s
    // z_index. Without this, children with no explicit `z_index` of their own would inherit
    // the *ambient* tier from wherever DevtoolsRoot happens to sit in the tree, not
    // StackingTier::Devtools + 1.
    StackingContext,
    SkipPortalLint,
    WatchedResource<DevtoolsState>,
    WatchedResource<DevtoolsSnapshot>
)]
pub struct DevtoolsRoot;

fn button_styles(active: bool) -> ButtonStyles {
    let normal = WoodpeckerStyle {
        height: 24.0.into(),
        padding: Edge::all(0.0).left(10.0).right(10.0),
        align_items: Some(WidgetAlignItems::Center),
        border_radius: Corner::all(colors::CONTROL_RADIUS * 0.7),
        background_color: if active {
            colors::PRIMARY
        } else {
            colors::BACKGROUND_MID
        },
        ..Default::default()
    };
    ButtonStyles {
        normal,
        hovered: WoodpeckerStyle {
            background_color: if active {
                colors::PRIMARY_LIGHT
            } else {
                colors::BACKGROUND_LIGHT
            },
            ..normal
        },
    }
}

fn label(text: impl Into<String>) -> (Element, WoodpeckerStyle, WidgetRender) {
    (
        Element,
        WoodpeckerStyle {
            font_size: colors::FONT_SIZE,
            color: colors::TEXT,
            text_wrap: TextWrap::None,
            ..Default::default()
        },
        WidgetRender::Text {
            content: text.into(),
        },
    )
}

fn render(
    current_widget: Res<CurrentWidget>,
    mut query: Query<(
        &WatchedResource<DevtoolsState>,
        &WatchedResource<DevtoolsSnapshot>,
        &mut WidgetChildren,
    )>,
    // The selected entity's *live* style, read directly rather than through the throttled
    // `DevtoolsSnapshot` -- the editable form below must always reflect (and let you edit)
    // the current frame's real value, not a value that can be up to `DevtoolsRefreshTimer`
    // stale.
    style_query: Query<&WoodpeckerStyle>,
) {
    let Ok((state, snapshot, mut children)) = query.get_mut(**current_widget) else {
        return;
    };

    *children = WidgetChildren::default();

    if !state.open {
        children.apply(current_widget.as_parent());
        return;
    }

    let current_widget_val = *current_widget;

    // Self-observer: `Change<TreeNodeSelected>` bubbles up through `ChildOf` ancestors (see
    // `on_change.rs`'s `ChangeTraversal`), so attaching here -- before any `.add()`, while
    // `children_queue` is still empty -- catches it from the `TreeView` nested several levels
    // down inside the tree `ScrollBox` below, without needing a handle to that specific entity.
    children.observe(
        current_widget_val,
        |trigger: On<Change<TreeNodeSelected>>, mut state: ResMut<DevtoolsState>| {
            if let Ok(bits) = trigger.data.key.parse::<u64>() {
                state.selected = Some(Entity::from_bits(bits));
            }
        },
    );

    // ---- Full-screen click-catcher, present only while pick mode is armed. Declared BEFORE
    // "panel" below so the panel's own buttons win ties over it (later siblings win ties at
    // the same inherited z-tier) -- clicks inside the panel keep working, clicks anywhere else
    // land on the catcher and are blocked from reaching the inspected app underneath.
    // `pick::click_to_select_system` does the actual selection independently, off the raw
    // cursor position -- this entity's only job is to block the inspected app's own `Pointer`
    // observers from also firing on the same click.
    if state.pick_mode {
        children.add::<Element>((
            Element,
            WoodpeckerStyle {
                position: WidgetPosition::Fixed,
                width: Units::Percentage(100.0),
                height: Units::Percentage(100.0),
                ..Default::default()
            },
            Pickable::default(),
        ));
        children.add_key("pick_catcher");
    }

    // ---- Title bar ----
    let mut title_bar = WidgetChildren::default();
    title_bar.add::<Element>((
        Element,
        WoodpeckerStyle {
            flex_grow: 1.0,
            font_size: colors::FONT_SIZE,
            color: colors::TEXT,
            text_wrap: TextWrap::None,
            ..Default::default()
        },
        WidgetRender::Text {
            content: "Woodpecker Devtools".into(),
        },
    ));
    title_bar.add_key("title");

    title_bar
        .add::<WButton>((
            WButton,
            WidgetChildren::default().with_child::<Element>(label(if state.pick_mode {
                "Cancel"
            } else {
                "Pick"
            })),
            button_styles(state.pick_mode),
        ))
        .observe(
            current_widget_val,
            |_: On<Pointer<Click>>,
             mut state: ResMut<DevtoolsState>,
             mut hover: ResMut<DevtoolsHover>| {
                state.pick_mode = !state.pick_mode;
                hover.0 = None;
            },
        );
    title_bar.add_key("pick_toggle");

    title_bar
        .add::<WButton>((
            WButton,
            WidgetChildren::default().with_child::<Element>(label("Close")),
            button_styles(false),
        ))
        .observe(
            current_widget_val,
            |_: On<Pointer<Click>>,
             mut state: ResMut<DevtoolsState>,
             mut hover: ResMut<DevtoolsHover>| {
                state.open = false;
                state.pick_mode = false;
                hover.0 = None;
            },
        );
    title_bar.add_key("close");

    // ---- Tree (scrollable) ----
    // No explicit `WoodpeckerStyle` override here -- `TreeView` already requires one
    // (`flex_direction: Column`, see `tree_view.rs`'s `default_style`), and passing a bare
    // `WoodpeckerStyle::default()` alongside it would win over that require (explicit bundle
    // components take precedence), silently reverting `flex_direction` back to `Row` and
    // laying every row out side by side instead of stacked.
    let selected_key = state.selected.map(|e| e.to_bits().to_string());
    let tree_scroll = WidgetChildren::default().with_child::<TreeView>(TreeView {
        nodes: snapshot.tree.clone(),
        selected_key: selected_key.clone(),
        indent: 12.0,
        // Selecting via "Pick" (or the tree itself, harmlessly a no-op there since a clicked
        // row is already visible) expands every ancestor of the newly-selected node so its
        // row comes into view instead of staying hidden in a collapsed subtree.
        force_expand_ancestors_of: selected_key,
    });

    // ---- Detail: an editable form for the selected entity's `WoodpeckerStyle` (if it has
    // one), plus a read-only field-list dump of everything else (state, layout, ...).
    let mut detail = WidgetChildren::default();
    let selected_style = state
        .selected
        .and_then(|e| style_query.get(e).ok().map(|s| (e, s)));
    match selected_style {
        Some((target, style)) => {
            style_editor::build(
                current_widget_val,
                target,
                style,
                state.open_color_field.as_deref(),
                &mut detail,
            );
        }
        None if state.selected.is_some() => {
            detail.add::<Element>(label("(no WoodpeckerStyle on this entity)"));
            detail.add_key("no_style");
        }
        None => {}
    }
    if !snapshot.detail_text.is_empty() {
        detail.add::<Element>((
            Element,
            WoodpeckerStyle {
                width: Units::Percentage(100.0),
                padding: Edge::all(0.0).left(8.0).top(10.0).bottom(2.0),
                font_size: 11.0,
                color: colors::PRIMARY_LIGHT,
                text_wrap: TextWrap::None,
                ..Default::default()
            },
            WidgetRender::Text {
                content: "Other components".into(),
            },
        ));
        detail.add_key("other_header");
        detail.add::<Element>((
            Element,
            WoodpeckerStyle {
                width: Units::Percentage(100.0),
                padding: Edge::all(8.0),
                font_size: 12.0,
                color: colors::TEXT.with_alpha(0.8),
                text_wrap: TextWrap::WordOrGlyph,
                ..Default::default()
            },
            WidgetRender::Text {
                content: snapshot.detail_text.clone(),
            },
        ));
        detail.add_key("other_text");
    }
    if selected_style.is_none() && snapshot.detail_text.is_empty() {
        detail.add::<Element>(label(
            "<nothing selected -- use \"Pick\" or click a row in the tree>",
        ));
        detail.add_key("placeholder");
    }
    let detail_scroll = WidgetChildren::default().with_child::<Element>((
        Element,
        WoodpeckerStyle {
            width: Units::Percentage(100.0),
            flex_direction: WidgetFlexDirection::Column,
            ..Default::default()
        },
        detail,
    ));

    let mut panel_content = WidgetChildren::default();
    panel_content.add::<Element>((
        Element,
        WoodpeckerStyle {
            width: Units::Percentage(100.0),
            height: 40.0.into(),
            padding: Edge::all(0.0).left(12.0).right(12.0),
            align_items: Some(WidgetAlignItems::Center),
            background_color: colors::DARK_BACKGROUND,
            gap: (8.0.into(), 0.0.into()),
            ..Default::default()
        },
        WidgetRender::Quad,
        title_bar,
    ));
    panel_content.add_key("title_bar");

    panel_content.add::<ScrollBox>((
        ScrollBox {
            hide_horizontal: true,
            ..Default::default()
        },
        WoodpeckerStyle {
            width: Units::Percentage(100.0),
            height: TREE_HEIGHT.into(),
            border: Edge::all(0.0).bottom(1.0),
            border_color: colors::BORDER,
            ..Default::default()
        },
        PassedChildren(tree_scroll),
    ));
    panel_content.add_key("tree");

    panel_content.add::<Element>((
        Element,
        WoodpeckerStyle {
            width: Units::Percentage(100.0),
            padding: Edge::all(8.0),
            font_size: colors::FONT_SIZE,
            color: colors::TEXT,
            ..Default::default()
        },
        WidgetRender::Text {
            content: "Detail".into(),
        },
    ));
    panel_content.add_key("detail_label");

    panel_content.add::<ScrollBox>((
        ScrollBox::default(),
        WoodpeckerStyle {
            width: Units::Percentage(100.0),
            flex_grow: 1.0,
            border: Edge::all(0.0).top(1.0),
            border_color: colors::BORDER,
            ..Default::default()
        },
        PassedChildren(detail_scroll),
    ));
    panel_content.add_key("detail");

    // Content is left blank here and kept updated afterwards by
    // `metrics_readout::refresh_metrics_text_system`, which finds this element via
    // `MetricsTextMarker` and writes its `WidgetRender::Text` directly -- deliberately NOT
    // driven by `DevtoolsSnapshot`/this reactive rebuild, since render metrics change on
    // essentially every frame and would otherwise force the whole panel (including the style
    // editor's input rows) to reconcile that often.
    panel_content.add::<Element>((
        Element,
        WoodpeckerStyle {
            width: Units::Percentage(100.0),
            height: 28.0.into(),
            padding: Edge::all(0.0).left(12.0).right(12.0),
            align_items: Some(WidgetAlignItems::Center),
            background_color: colors::DARK_BACKGROUND,
            font_size: 11.0,
            color: colors::TEXT.with_alpha(0.7),
            ..Default::default()
        },
        WidgetRender::Text {
            content: String::new(),
        },
        MetricsTextMarker,
    ));
    panel_content.add_key("metrics");

    children.add::<Element>((
        Element,
        WoodpeckerStyle {
            width: PANEL_WIDTH.into(),
            height: Units::Percentage(100.0),
            flex_direction: WidgetFlexDirection::Column,
            background_color: colors::BACKGROUND,
            border: Edge::all(0.0).left(1.0),
            border_color: colors::BORDER,
            ..Default::default()
        },
        WidgetRender::Quad,
        panel_content,
    ));
    children.add_key("panel");

    // ---- Selection/hover highlight, drawn last so it visually sits above the panel too.
    // No `Pickable` at all -- it must never intercept a click.
    if let Some((position, size)) = snapshot.highlight {
        children.add::<Element>((
            Element,
            WoodpeckerStyle {
                position: WidgetPosition::Fixed,
                left: position.x.into(),
                top: position.y.into(),
                width: size.x.into(),
                height: size.y.into(),
                border: Edge::all(2.0),
                border_color: colors::PRIMARY,
                background_color: Color::NONE,
                ..Default::default()
            },
            WidgetRender::Quad,
        ));
        children.add_key("highlight");
    }

    children.apply(current_widget_val.as_parent());
}
