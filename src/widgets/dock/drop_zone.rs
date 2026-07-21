use bevy::prelude::*;

use crate::prelude::*;

use super::panels::DockPanelRegistry;
use super::tab_bar::{DockDragActive, DockDragState, DockPanelPayload};
use super::tree::{DockEdge, DockTree, NodePath, PanelId};
use super::DockStyles;

/// Which edge (if any) is currently highlighted while a drag hovers this surface. A single
/// shared state per [`DockDropSurface`] rather than one per edge zone -- only one edge can be
/// hovered at a time, and `HookHelper::use_state` only scopes one state entity per (widget,
/// type) pair, so a single combined state is both simpler and the only option that fits.
#[derive(Component, Reflect, Clone, PartialEq, Default, Debug)]
#[reflect(Component, DiffableProp, PartialEq)]
pub(super) struct DockDropZoneState {
    hovering: Option<DockEdge>,
    /// This surface's own current group location, and the panel to redock onto -- written
    /// fresh by `render` every frame rather than `move`-captured into the `droppable`
    /// closures below, for the exact reason `DockTabBarDropState`'s own fields exist: see
    /// `WidgetChildren::droppable`'s doc comment.
    node_path: NodePath,
    target_panel: Option<PanelId>,
}

fn default_surface_style() -> WoodpeckerStyle {
    WoodpeckerStyle {
        width: Units::Percentage(100.0),
        height: Units::Percentage(100.0),
        position: WidgetPosition::Relative,
        ..Default::default()
    }
}

/// The content area of one [`super::DockNode::Tabs`] group: the active panel's own content,
/// overlaid with five drop zones (edges + center) a dragged [`DockPanelPayload`] can land on to
/// redock there. Its own widget (rather than a plain `Element` [`super::tree_render`] builds
/// inline) purely so [`HookHelper::use_state`] scopes [`DockDropZoneState`] correctly -- state
/// is keyed by `(widget entity, type)`, so every `Tabs` group in the tree needs to be its own
/// entity to get its own independent drop-zone hover state, not one shared across the whole
/// `DockArea`.
#[derive(Widget, Component, Reflect, Clone, PartialEq, Default)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(WoodpeckerStyle = default_surface_style(), WidgetChildren, DockStyles)]
pub(crate) struct DockDropSurface {
    pub node_path: NodePath,
    pub active_panel: Option<PanelId>,
}

fn render(
    mut commands: Commands,
    mut hooks: ResMut<HookHelper>,
    current_widget: Res<CurrentWidget>,
    mut query: Query<(&DockDropSurface, &DockStyles, &mut WidgetChildren)>,
    registry_query: Query<&DockPanelRegistry>,
    active_query: Query<&DockDragActive>,
    state_query: Query<&DockDropZoneState>,
) {
    let Ok((surface, styles, mut children)) = query.get_mut(**current_widget) else {
        return;
    };

    let tree_entity =
        hooks.use_context::<DockTree>(&mut commands, *current_widget, DockTree::default());
    let registry_entity = hooks.use_context::<DockPanelRegistry>(
        &mut commands,
        *current_widget,
        DockPanelRegistry::default(),
    );
    let registry = registry_query.get(registry_entity).ok();
    let drag_active_entity = hooks.use_context::<DockDragActive>(
        &mut commands,
        *current_widget,
        DockDragActive::default(),
    );
    let dragging = active_query
        .get(drag_active_entity)
        .map(|a| a.dragging)
        .unwrap_or(false);
    let state_entity =
        hooks.use_state(&mut commands, *current_widget, DockDropZoneState::default());
    let hovering = state_query.get(state_entity).ok().and_then(|s| s.hovering);
    commands.entity(state_entity).insert(DockDropZoneState {
        hovering,
        node_path: surface.node_path.clone(),
        target_panel: surface.active_panel.clone(),
    });
    info!(
        "dock: DockDropSurface render node_path={:?} dragging={dragging} hovering={hovering:?}",
        surface.node_path.0
    );

    *children = WidgetChildren::default();

    if let Some(panel_id) = &surface.active_panel {
        if let Some(def) = registry.and_then(|r| r.get(panel_id)) {
            children.add::<Element>((
                Element,
                WoodpeckerStyle {
                    width: Units::Percentage(100.0),
                    height: Units::Percentage(100.0),
                    padding: Edge::all(styles.content_padding),
                    ..Default::default()
                },
                (def.factory)(panel_id.clone()),
            ));
            children.add_key(panel_id.0.clone());
        }
    }

    if dragging {
        for edge in [
            DockEdge::Left,
            DockEdge::Right,
            DockEdge::Top,
            DockEdge::Bottom,
            DockEdge::Center,
        ] {
            children.add::<Element>((Element, hit_region_style(edge), Pickable::default()));
            children.add_key(format!("zone_{edge:?}"));

            let hover_edge = edge;
            children.droppable::<DockPanelPayload, DockDropZoneState>(
                *current_widget,
                state_entity,
                move |payload: &DockPanelPayload, state: &DockDropZoneState| {
                    payload.source != state.node_path
                },
                move |state: &mut DockDropZoneState, valid: Option<bool>| {
                    info!("dock: zone {hover_edge:?} on_hover valid={valid:?}");
                    state.hovering = match valid {
                        Some(true) => Some(hover_edge),
                        _ => None,
                    };
                },
                move |payload: &DockPanelPayload,
                      valid: bool,
                      state: &DockDropZoneState,
                      commands: &mut Commands| {
                    info!(
                        "dock: zone {edge:?} on_drop payload={:?} valid={valid}",
                        payload.panel.0
                    );
                    if !valid {
                        return;
                    }
                    let payload = payload.clone();
                    let Some(target_panel) = state.target_panel.clone() else {
                        return;
                    };
                    commands.queue(move |world: &mut World| {
                        let Some(mut tree) = world.get_mut::<DockTree>(tree_entity) else {
                            return;
                        };
                        if !tree.remove_panel(&payload.panel) {
                            return;
                        }
                        let Some(target) = tree.find_panel_group(&target_panel) else {
                            warn!(
                                "Woodpecker UI: dock drop target group vanished before the drop \
                                 could land -- panel {:?} was removed from its old spot but not \
                                 re-placed.",
                                payload.panel.0
                            );
                            return;
                        };
                        if edge == DockEdge::Center {
                            tree.insert_center(&target, payload.panel.clone());
                        } else {
                            tree.insert_edge(&target, payload.panel.clone(), edge, 0.5);
                        }
                        if let Some(mut drag_state) =
                            world.get_mut::<DockDragState>(payload.drag_state)
                        {
                            drag_state.consumed = true;
                        }
                    });
                },
            );
        }

        if let Some(edge) = hovering {
            children.add::<Element>((
                Element,
                indicator_style(edge, styles.drop_highlight),
                WidgetRender::Quad,
            ));
            children.add_key("indicator");
        }
    }

    children.apply(current_widget.as_parent());
}

/// Always-available outer-edge drop targets for the whole [`super::DockArea`], independent of
/// whatever [`super::DockNode::Tabs`] groups do or don't currently exist near an edge.
///
/// Every other drop target in this module ([`DockDropSurface`]'s own edges, [`super::DockTabBar`])
/// is scoped to *one specific* `Tabs` group's own content area -- which means once the only
/// group that ever occupied, say, the leftmost slot is closed (its `Split` branch collapses away
/// via [`DockTree::simplify`]), there is no longer anything left at that edge to drop onto at
/// all. Confirmed via user report: closing the sole left-hand panel made it impossible to redock
/// anything to the left ever again, even though the tree still has plenty of room to grow there.
/// This widget renders four thin strips around the *entire* `DockArea`'s own outer boundary,
/// always reachable regardless of the tree's current shape, that split the tree's whole `root`
/// the same way any other edge drop would split a single leaf.
///
/// Deliberately thin ([`ROOT_EDGE_PIXELS`], not a `hit_region_style`-style percentage) and layered
/// on *top* of the rest of the tree (declared last in [`super::DockArea::render`]'s own children,
/// after `render_dock_node`'s whole subtree, giving it the highest paint order) -- a generous
/// percentage-of-whole-area strip would swallow up a large fraction of every outermost leaf's own
/// [`DockDropSurface`] zones; a thin sliver right at the true outer edge instead coexists with
/// them, since a leaf's own 25%-of-*its own bounds* zone still wins everywhere but the outermost
/// few pixels.
///
/// Doesn't need [`DockDropZoneState`]'s `node_path`/`target_panel` staleness workaround at all --
/// unlike a specific `Tabs` group's path, [`NodePath::root`] never goes stale (it's always the
/// empty path), so there's nothing to re-resolve after `remove_panel`.
#[derive(Widget, Component, Reflect, Clone, PartialEq, Default)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(root_zone_render)]
#[require(WoodpeckerStyle = root_zone_style(), WidgetChildren, DockStyles)]
pub(crate) struct DockRootDropZone;

const ROOT_EDGE_PIXELS: f32 = 14.0;

fn root_zone_style() -> WoodpeckerStyle {
    WoodpeckerStyle {
        position: WidgetPosition::Absolute,
        width: Units::Percentage(100.0),
        height: Units::Percentage(100.0),
        top: 0.0.into(),
        left: 0.0.into(),
        ..Default::default()
    }
}

fn root_zone_render(
    mut commands: Commands,
    mut hooks: ResMut<HookHelper>,
    current_widget: Res<CurrentWidget>,
    mut query: Query<(&DockStyles, &mut WidgetChildren), With<DockRootDropZone>>,
    tree_query: Query<&DockTree>,
    active_query: Query<&DockDragActive>,
    state_query: Query<&DockDropZoneState>,
) {
    let Ok((styles, mut children)) = query.get_mut(**current_widget) else {
        return;
    };

    let tree_entity =
        hooks.use_context::<DockTree>(&mut commands, *current_widget, DockTree::default());
    let drag_active_entity = hooks.use_context::<DockDragActive>(
        &mut commands,
        *current_widget,
        DockDragActive::default(),
    );
    let dragging = active_query
        .get(drag_active_entity)
        .map(|a| a.dragging)
        .unwrap_or(false);
    let tree_is_empty = tree_query
        .get(tree_entity)
        .is_ok_and(|tree| tree.root.is_none());
    let state_entity =
        hooks.use_state(&mut commands, *current_widget, DockDropZoneState::default());
    let hovering = state_query.get(state_entity).ok().and_then(|s| s.hovering);
    commands.entity(state_entity).insert(DockDropZoneState {
        hovering,
        node_path: NodePath::root(),
        target_panel: None,
    });

    *children = WidgetChildren::default();

    if !dragging {
        children.apply(current_widget.as_parent());
        return;
    }

    let edges: &[DockEdge] = if tree_is_empty {
        &[
            DockEdge::Left,
            DockEdge::Right,
            DockEdge::Top,
            DockEdge::Bottom,
            DockEdge::Center,
        ]
    } else {
        &[
            DockEdge::Left,
            DockEdge::Right,
            DockEdge::Top,
            DockEdge::Bottom,
        ]
    };

    for &edge in edges {
        children.add::<Element>((
            Element,
            root_hit_region_style(edge, tree_is_empty),
            Pickable::default(),
        ));
        children.add_key(format!("root_zone_{edge:?}"));

        let hover_edge = edge;
        children.droppable::<DockPanelPayload, DockDropZoneState>(
            *current_widget,
            state_entity,
            |payload: &DockPanelPayload, _state: &DockDropZoneState| {
                payload.source != NodePath::root()
            },
            move |state: &mut DockDropZoneState, valid: Option<bool>| {
                info!("dock: root zone {hover_edge:?} on_hover valid={valid:?}");
                state.hovering = match valid {
                    Some(true) => Some(hover_edge),
                    _ => None,
                };
            },
            move |payload: &DockPanelPayload,
                  valid: bool,
                  _state: &DockDropZoneState,
                  commands: &mut Commands| {
                info!(
                    "dock: root zone {edge:?} on_drop payload={:?} valid={valid}",
                    payload.panel.0
                );
                if !valid {
                    return;
                }
                let payload = payload.clone();
                commands.queue(move |world: &mut World| {
                    let Some(mut tree) = world.get_mut::<DockTree>(tree_entity) else {
                        return;
                    };
                    if !tree.remove_panel(&payload.panel) {
                        return;
                    }
                    if edge == DockEdge::Center {
                        tree.insert_center(&NodePath::root(), payload.panel.clone());
                    } else {
                        tree.insert_edge(&NodePath::root(), payload.panel.clone(), edge, 0.25);
                    }
                    if let Some(mut drag_state) = world.get_mut::<DockDragState>(payload.drag_state)
                    {
                        drag_state.consumed = true;
                    }
                });
            },
        );
    }

    if let Some(edge) = hovering {
        children.add::<Element>((
            Element,
            if tree_is_empty && edge == DockEdge::Center {
                indicator_style(edge, styles.drop_highlight)
            } else {
                root_indicator_style(edge, styles.drop_highlight)
            },
            WidgetRender::Quad,
        ));
        children.add_key("root_indicator");
    }

    children.apply(current_widget.as_parent());
}

/// [`hit_region_style`]'s counterpart for [`DockRootDropZone`] -- a thin, fixed-width outer
/// strip instead of a generous percentage of the surface, so it coexists with each outermost
/// leaf's own (much larger) per-group zones instead of swallowing them. `Center`, when present,
/// still covers the whole area -- there's no leaf underneath it to coexist with on an empty tree.
fn root_hit_region_style(edge: DockEdge, tree_is_empty: bool) -> WoodpeckerStyle {
    if edge == DockEdge::Center && tree_is_empty {
        return hit_region_style(DockEdge::Center);
    }
    let base = WoodpeckerStyle {
        position: WidgetPosition::Absolute,
        background_color: Color::NONE,
        ..Default::default()
    };
    match edge {
        DockEdge::Left => WoodpeckerStyle {
            left: 0.0.into(),
            top: 0.0.into(),
            width: ROOT_EDGE_PIXELS.into(),
            height: Units::Percentage(100.0),
            ..base
        },
        DockEdge::Right => WoodpeckerStyle {
            right: 0.0.into(),
            top: 0.0.into(),
            width: ROOT_EDGE_PIXELS.into(),
            height: Units::Percentage(100.0),
            ..base
        },
        DockEdge::Top => WoodpeckerStyle {
            left: 0.0.into(),
            top: 0.0.into(),
            width: Units::Percentage(100.0),
            height: ROOT_EDGE_PIXELS.into(),
            ..base
        },
        DockEdge::Bottom => WoodpeckerStyle {
            left: 0.0.into(),
            bottom: 0.0.into(),
            width: Units::Percentage(100.0),
            height: ROOT_EDGE_PIXELS.into(),
            ..base
        },
        DockEdge::Center => hit_region_style(DockEdge::Center),
    }
}

/// [`indicator_style`]'s counterpart for [`DockRootDropZone`]'s own thin strips.
fn root_indicator_style(edge: DockEdge, highlight: Color) -> WoodpeckerStyle {
    const BORDER_THICKNESS: f32 = 3.0;
    let base = WoodpeckerStyle {
        position: WidgetPosition::Absolute,
        background_color: highlight.with_alpha(0.45),
        border_color: highlight,
        ..Default::default()
    };
    match edge {
        DockEdge::Left => WoodpeckerStyle {
            left: 0.0.into(),
            top: 0.0.into(),
            width: ROOT_EDGE_PIXELS.into(),
            height: Units::Percentage(100.0),
            border: Edge::all(0.0).right(BORDER_THICKNESS),
            ..base
        },
        DockEdge::Right => WoodpeckerStyle {
            right: 0.0.into(),
            top: 0.0.into(),
            width: ROOT_EDGE_PIXELS.into(),
            height: Units::Percentage(100.0),
            border: Edge::all(0.0).left(BORDER_THICKNESS),
            ..base
        },
        DockEdge::Top => WoodpeckerStyle {
            left: 0.0.into(),
            top: 0.0.into(),
            width: Units::Percentage(100.0),
            height: ROOT_EDGE_PIXELS.into(),
            border: Edge::all(0.0).bottom(BORDER_THICKNESS),
            ..base
        },
        DockEdge::Bottom => WoodpeckerStyle {
            left: 0.0.into(),
            bottom: 0.0.into(),
            width: Units::Percentage(100.0),
            height: ROOT_EDGE_PIXELS.into(),
            border: Edge::all(0.0).top(BORDER_THICKNESS),
            ..base
        },
        DockEdge::Center => indicator_style(DockEdge::Center, highlight),
    }
}

/// The invisible, `Pickable` region a drag can land in for `edge` -- generous (25% of the
/// surface, or the whole thing for `Center`) so a drop doesn't need pixel-precision, independent
/// of [`indicator_style`]'s much thinner visual bar.
fn hit_region_style(edge: DockEdge) -> WoodpeckerStyle {
    const EDGE_FRACTION: f32 = 25.0;
    let base = WoodpeckerStyle {
        position: WidgetPosition::Absolute,
        background_color: Color::NONE,
        ..Default::default()
    };
    match edge {
        DockEdge::Left => WoodpeckerStyle {
            left: 0.0.into(),
            top: 0.0.into(),
            width: Units::Percentage(EDGE_FRACTION),
            height: Units::Percentage(100.0),
            ..base
        },
        DockEdge::Right => WoodpeckerStyle {
            left: Units::Percentage(100.0 - EDGE_FRACTION),
            top: 0.0.into(),
            width: Units::Percentage(EDGE_FRACTION),
            height: Units::Percentage(100.0),
            ..base
        },
        DockEdge::Top => WoodpeckerStyle {
            left: 0.0.into(),
            top: 0.0.into(),
            width: Units::Percentage(100.0),
            height: Units::Percentage(EDGE_FRACTION),
            ..base
        },
        DockEdge::Bottom => WoodpeckerStyle {
            left: 0.0.into(),
            top: Units::Percentage(100.0 - EDGE_FRACTION),
            width: Units::Percentage(100.0),
            height: Units::Percentage(EDGE_FRACTION),
            ..base
        },
        DockEdge::Center => WoodpeckerStyle {
            left: Units::Percentage(EDGE_FRACTION),
            top: Units::Percentage(EDGE_FRACTION),
            width: Units::Percentage(100.0 - EDGE_FRACTION * 2.0),
            height: Units::Percentage(100.0 - EDGE_FRACTION * 2.0),
            ..base
        },
    }
}

/// The "drop bar" itself -- a bold, unmissable preview of exactly where a panel will land: for
/// an edge, a full-height (or full-width) band the same size as the space the new split will
/// actually occupy, with a bright accent border on its inner edge for crispness; for `Center`,
/// an inset outline framing the whole surface to read as "joins this tab group" rather than
/// "splits it". Deliberately sized to match [`hit_region_style`]'s own `EDGE_FRACTION`, rather
/// than a thin cosmetic line, so the preview is an honest "the new panel will fill this much
/// space", not just a pointer at an edge.
fn indicator_style(edge: DockEdge, highlight: Color) -> WoodpeckerStyle {
    const EDGE_FRACTION: f32 = 25.0;
    const BORDER_THICKNESS: f32 = 3.0;
    const CENTER_INSET: f32 = 10.0;
    let base = WoodpeckerStyle {
        position: WidgetPosition::Absolute,
        background_color: highlight.with_alpha(0.35),
        border_color: highlight,
        ..Default::default()
    };
    match edge {
        DockEdge::Left => WoodpeckerStyle {
            left: 0.0.into(),
            top: 0.0.into(),
            width: Units::Percentage(EDGE_FRACTION),
            height: Units::Percentage(100.0),
            border: Edge::all(0.0).right(BORDER_THICKNESS),
            ..base
        },
        DockEdge::Right => WoodpeckerStyle {
            right: 0.0.into(),
            top: 0.0.into(),
            width: Units::Percentage(EDGE_FRACTION),
            height: Units::Percentage(100.0),
            border: Edge::all(0.0).left(BORDER_THICKNESS),
            ..base
        },
        DockEdge::Top => WoodpeckerStyle {
            left: 0.0.into(),
            top: 0.0.into(),
            width: Units::Percentage(100.0),
            height: Units::Percentage(EDGE_FRACTION),
            border: Edge::all(0.0).bottom(BORDER_THICKNESS),
            ..base
        },
        DockEdge::Bottom => WoodpeckerStyle {
            left: 0.0.into(),
            bottom: 0.0.into(),
            width: Units::Percentage(100.0),
            height: Units::Percentage(EDGE_FRACTION),
            border: Edge::all(0.0).top(BORDER_THICKNESS),
            ..base
        },
        DockEdge::Center => WoodpeckerStyle {
            left: Units::Percentage(0.0),
            top: Units::Percentage(0.0),
            width: Units::Percentage(100.0),
            height: Units::Percentage(100.0),
            padding: Edge::all(CENTER_INSET),
            border: Edge::all(BORDER_THICKNESS),
            background_color: highlight.with_alpha(0.12),
            ..base
        },
    }
}
