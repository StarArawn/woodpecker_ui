use bevy::prelude::*;

use crate::prelude::*;

use super::floating::attach_revert_on_unconsumed_drag;
use super::panels::DockPanelRegistry;
use super::tree::{DockTree, NodePath, PanelId};
use super::DockStyles;

/// Permanent marker on each [`DockTabHeader`] identifying what's being dragged and where it
/// came from -- read by [`super::drop_zone`]'s drop targets via `WidgetChildren::droppable`,
/// which requires the payload live permanently on the dragged entity's own bundle.
///
/// `drag_state` is the header's own [`DockDragState`] entity (the one `header_render` creates
/// via `hooks.use_state`) -- carried here so a drop target's `on_drop`, which only knows about
/// the *payload*, can still reach back and mark that specific drag `consumed`. See
/// `DockDragState::consumed`'s own doc comment for why this mutual-exclusion flag exists at all.
#[derive(Component, Clone, PartialEq, Debug)]
pub(super) struct DockPanelPayload {
    pub panel: PanelId,
    pub source: NodePath,
    pub drag_state: Entity,
}

/// Per-header drag state.
#[derive(Component, Reflect, Clone, Copy, PartialEq, Default, Debug)]
#[reflect(Component, DiffableProp, PartialEq)]
pub(super) struct DockDragState {
    pub dragging: bool,
    pub position: Vec2,
    /// Set synchronously, inside the same deferred `World` closure, by whichever drop target's
    /// `on_drop` (`tab_bar`'s or `drop_zone`'s) successfully redocks this drag; reset on the
    /// next `DragStart`. No longer load-bearing for correctness -- back when an unconsumed drag
    /// floated the panel into a new window instead of reverting (see
    /// [`super::floating::attach_revert_on_unconsumed_drag`]'s own doc comment for why that
    /// changed), this is what stopped that fallback from undoing an `on_drop` that had *already*
    /// succeeded a moment earlier (confirmed via a live repro: `on_drop` redocked the panel,
    /// which put it back in the tree, so the old float fallback's own `remove_panel` call found
    /// it there and "successfully" floated a panel that had already landed correctly). Kept
    /// purely so that fallback's diagnostic log can still distinguish "this drag was consumed by
    /// a drop handler" from "it reverted" -- see that function's own doc comment.
    pub consumed: bool,
}

/// Whether *any* tab is currently being dragged within this `DockArea` -- a shared context
/// (seeded via `hooks.use_own_context` on `DockArea`, alongside `DockTree`/`DockFloating`, and
/// `DiffableProp` so reading it drives a re-render the same way reading `DockTree` does) so
/// [`super::drop_zone::DockDropSurface`] only builds its drop-target hit-regions while a drag is
/// actually in progress, rather than permanently overlaying -- and stealing pointer events from
/// -- every panel's own content underneath.
#[derive(Component, Reflect, Clone, Copy, PartialEq, Default, Debug)]
#[reflect(Component, DiffableProp, PartialEq)]
pub(crate) struct DockDragActive {
    pub dragging: bool,
}

/// Styles for [`DockTabBar`] and its [`DockTabHeader`] children.
#[derive(Component, Reflect, Clone, Copy, PartialEq)]
#[reflect(Component, DiffableProp, PartialEq)]
pub struct DockTabBarStyles {
    /// The strip's height.
    pub height: f32,
    /// Background of an inactive header.
    pub background: Color,
    /// Background of the active header.
    pub active_background: Color,
    /// Bottom-border color of the active header -- the "you are here" affordance.
    pub active_border: Color,
    /// Text color of an inactive header.
    pub text_color: Color,
    /// Text color of the active header.
    pub active_text_color: Color,
    /// Font size of header labels.
    pub font_size: f32,
}

impl Default for DockTabBarStyles {
    fn default() -> Self {
        Self::from_theme(&Theme::default())
    }
}

impl ThemedStyle for DockTabBarStyles {
    fn from_theme(theme: &Theme) -> Self {
        Self {
            height: 32.0,
            background: theme.dark_background,
            active_background: theme.background,
            active_border: theme.primary,
            text_color: theme.text.with_alpha(0.7),
            active_text_color: theme.text,
            font_size: 12.0,
        }
    }
}

fn default_tab_bar_style() -> WoodpeckerStyle {
    WoodpeckerStyle {
        width: Units::Percentage(100.0),
        flex_direction: WidgetFlexDirection::Row,
        flex_shrink: 0.0,
        overflow: WidgetOverflow::Hidden,
        ..Default::default()
    }
}

/// The tab strip for one [`super::DockNode::Tabs`] group -- select/close/drag-to-redock
/// headers, one per panel. Deliberately not built on `crate::widgets::tab`'s `Tab`/`TabButton`:
/// that pair is index-based and single-level, with no reorder/close/drag support, the wrong
/// shape for a redockable header row.
/// Whether the tab bar itself (as opposed to any specific header) is currently a valid drop
/// target -- dropping anywhere on the bar (including directly on another tab's own header,
/// since a header that doesn't handle `Pointer<DragDrop>` itself lets it bubble up to its
/// parent) joins this tab group, the same as dropping on the content area's `Center` zone.
#[derive(Component, Reflect, Clone, PartialEq, Default, Debug)]
#[reflect(Component, DiffableProp, PartialEq)]
pub(super) struct DockTabBarDropState {
    hovering: bool,
    /// This tab group's own current location, and the panel to redock onto -- written fresh
    /// by `render` every frame (the state entity is a real `Component`, not a closure
    /// capture) rather than captured into the `droppable` closures below. Those closures are
    /// only ever *spawned* once (see `ObserverCache`'s dedup-by-target-entity), so anything
    /// `move`-captured into them at declaration time is frozen at whatever it was on this
    /// widget's first-ever render, even though this same tab-bar entity gets reused (by key)
    /// across tree restructuring -- reading it back off `&S` at call time instead is what
    /// keeps the drop target correct after the tree has changed shape.
    node_path: NodePath,
    target_panel: Option<PanelId>,
}

#[derive(Widget, Component, Reflect, Clone, PartialEq, Default)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(render)]
#[require(WoodpeckerStyle = default_tab_bar_style(), WidgetChildren, WidgetRender = WidgetRender::Quad, Pickable, DockTabBarStyles, DockStyles)]
pub struct DockTabBar {
    /// The panels in this group, in tab order.
    pub panels: Vec<PanelId>,
    /// The index into `panels` currently shown.
    pub active: usize,
    /// This tab group's own location in the owning [`super::DockTree`].
    pub node_path: NodePath,
}

fn render(
    mut commands: Commands,
    mut hooks: ResMut<HookHelper>,
    current_widget: Res<CurrentWidget>,
    mut query: Query<(
        &DockTabBar,
        &DockTabBarStyles,
        &DockStyles,
        &mut WoodpeckerStyle,
        &mut WidgetChildren,
    )>,
    registry_query: Query<&DockPanelRegistry>,
    drop_state_query: Query<&DockTabBarDropState>,
) {
    let Ok((tab_bar, styles, dock_styles, mut widget_style, mut children)) =
        query.get_mut(**current_widget)
    else {
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
    let drop_state_entity = hooks.use_state(
        &mut commands,
        *current_widget,
        DockTabBarDropState::default(),
    );

    let target_panel = tab_bar
        .panels
        .get(tab_bar.active)
        .or_else(|| tab_bar.panels.first())
        .cloned();
    let hovering = drop_state_query
        .get(drop_state_entity)
        .map(|s| s.hovering)
        .unwrap_or(false);
    info!(
        "dock: tab_bar render node_path={:?} panels={:?} active={} target_panel={:?}",
        tab_bar.node_path.0,
        tab_bar.panels.iter().map(|p| &p.0).collect::<Vec<_>>(),
        tab_bar.active,
        target_panel.as_ref().map(|p| &p.0)
    );
    commands
        .entity(drop_state_entity)
        .insert(DockTabBarDropState {
            hovering,
            node_path: tab_bar.node_path.clone(),
            target_panel,
        });

    widget_style.background_color = if hovering {
        styles.active_border.with_alpha(0.25)
    } else {
        styles.background
    };
    widget_style.border_radius =
        Corner::vertical(dock_styles.panel_border_radius.into(), 0.0.into());

    *children = WidgetChildren::default();

    children.droppable::<DockPanelPayload, DockTabBarDropState>(
        *current_widget,
        drop_state_entity,
        |payload: &DockPanelPayload, state: &DockTabBarDropState| payload.source != state.node_path,
        |state: &mut DockTabBarDropState, valid: Option<bool>| {
            info!("dock: tab_bar on_hover valid={valid:?}");
            state.hovering = valid == Some(true);
        },
        move |payload: &DockPanelPayload,
              valid: bool,
              state: &DockTabBarDropState,
              commands: &mut Commands| {
            info!(
                "dock: tab_bar on_drop payload={:?} valid={valid}",
                payload.panel.0
            );
            if !valid {
                return;
            }
            let payload = payload.clone();
            info!(
                "dock: tab_bar on_drop state.node_path={:?} state.target_panel={:?}",
                state.node_path.0,
                state.target_panel.as_ref().map(|p| &p.0)
            );
            let Some(target_panel) = state.target_panel.clone() else {
                warn!("dock: tab_bar on_drop bailing -- state.target_panel is None");
                return;
            };
            commands.queue(move |world: &mut World| {
                let Some(mut tree) = world.get_mut::<DockTree>(tree_entity) else {
                    warn!("dock: tab_bar on_drop queued closure -- DockTree query failed");
                    return;
                };
                if !tree.remove_panel(&payload.panel) {
                    warn!(
                        "dock: tab_bar on_drop queued closure -- remove_panel({:?}) returned \
                         false (already gone -- another handler beat us to it)",
                        payload.panel.0
                    );
                    return;
                }
                let Some(target) = tree.find_panel_group(&target_panel) else {
                    warn!(
                        "Woodpecker UI: dock drop target group vanished before the drop could \
                         land -- panel {:?} was removed from its old spot but not re-placed.",
                        payload.panel.0
                    );
                    return;
                };
                info!(
                    "dock: tab_bar on_drop queued closure -- redocking {:?} at {:?}",
                    payload.panel.0, target.0
                );
                tree.insert_center(&target, payload.panel.clone());
                if let Some(mut drag_state) = world.get_mut::<DockDragState>(payload.drag_state) {
                    drag_state.consumed = true;
                }
            });
        },
    );

    for (index, panel_id) in tab_bar.panels.iter().enumerate() {
        let def = registry.and_then(|r| r.get(panel_id));
        let title = def
            .map(|d| d.title.clone())
            .unwrap_or_else(|| panel_id.0.clone());
        let closable = def.map(|d| d.closable).unwrap_or(true);

        children.add::<DockTabHeader>((DockTabHeader {
            panel: panel_id.clone(),
            title,
            closable,
            active: index == tab_bar.active,
            node_path: tab_bar.node_path.clone(),
        },));
        children.add_key(panel_id.0.clone());
    }

    children.apply(current_widget.as_parent());
}

fn default_header_style() -> WoodpeckerStyle {
    WoodpeckerStyle {
        height: Units::Percentage(100.0),
        flex_direction: WidgetFlexDirection::Row,
        align_items: Some(WidgetAlignItems::Center),
        padding: Edge::all(0.0).left(10.0).right(8.0),
        flex_shrink: 0.0,
        ..Default::default()
    }
}

/// One clickable, closable, draggable tab header within a [`DockTabBar`].
#[derive(Widget, Component, Reflect, Clone, PartialEq, Default)]
#[reflect(Component, DiffableProp, PartialEq)]
#[auto_update(header_render)]
#[require(WoodpeckerStyle = default_header_style(), WidgetChildren, WidgetRender = WidgetRender::Quad, Pickable, DockTabBarStyles)]
pub(crate) struct DockTabHeader {
    pub panel: PanelId,
    pub title: String,
    pub closable: bool,
    pub active: bool,
    pub node_path: NodePath,
}

fn header_render(
    mut commands: Commands,
    mut hooks: ResMut<HookHelper>,
    current_widget: Res<CurrentWidget>,
    overlay_root: Option<Res<OverlayRoot>>,
    icon_font: Res<IconFont>,
    mut query: Query<(
        &DockTabHeader,
        &DockTabBarStyles,
        &mut WoodpeckerStyle,
        &mut WidgetChildren,
    )>,
    drag_state_query: Query<&DockDragState>,
) {
    let Ok((header, styles, mut widget_style, mut children)) = query.get_mut(**current_widget)
    else {
        return;
    };

    let tree_entity =
        hooks.use_context::<DockTree>(&mut commands, *current_widget, DockTree::default());
    let drag_active_entity = hooks.use_context::<DockDragActive>(
        &mut commands,
        *current_widget,
        DockDragActive::default(),
    );
    let drag_state_entity =
        hooks.use_state(&mut commands, *current_widget, DockDragState::default());
    let dragging = drag_state_query
        .get(drag_state_entity)
        .map(|s| s.dragging)
        .unwrap_or(false);

    commands.entity(**current_widget).insert(DockPanelPayload {
        panel: header.panel.clone(),
        source: header.node_path.clone(),
        drag_state: drag_state_entity,
    });

    widget_style.height = styles.height.into();
    widget_style.background_color = if header.active {
        styles.active_background
    } else {
        styles.background
    };
    widget_style.border_color = styles.active_border;
    widget_style.border = if header.active {
        Edge::all(0.0).bottom(2.0)
    } else {
        Edge::all(0.0)
    };
    widget_style.display = if dragging {
        WidgetDisplay::None
    } else {
        WidgetDisplay::Flex
    };

    let current_widget_val = *current_widget;
    let overlay_root_val = overlay_root.as_deref().copied();

    *children = WidgetChildren::default();

    let panel_for_activate = header.panel.clone();
    let node_path_for_activate = header.node_path.clone();
    children.self_observe(
        current_widget_val,
        move |_trigger: On<Pointer<Click>>, mut tree_query: Query<&mut DockTree>| {
            if let Ok(mut tree) = tree_query.get_mut(tree_entity) {
                tree.set_active(&node_path_for_activate, &panel_for_activate);
            }
        },
    );

    let panel_for_log = header.panel.clone();
    children.drag_state::<DockDragState>(
        current_widget_val,
        drag_state_entity,
        move |state, phase| match phase {
            DragPhase::Start(p) => {
                info!(
                    "dock: drag_state START panel={:?} pos={p:?}",
                    panel_for_log.0
                );
                state.dragging = true;
                state.position = p;
                state.consumed = false;
            }
            DragPhase::Move(p) => {
                state.dragging = true;
                state.position = p;
            }
            DragPhase::End => {
                info!("dock: drag_state END panel={:?}", panel_for_log.0);
                state.dragging = false;
            }
        },
    );

    let panel_for_active_log = header.panel.clone();
    children.self_observe(
        current_widget_val,
        move |_trigger: On<Pointer<DragStart>>, mut active_query: Query<&mut DockDragActive>| {
            info!(
                "dock: DockDragActive -> true (panel={:?})",
                panel_for_active_log.0
            );
            if let Ok(mut active) = active_query.get_mut(drag_active_entity) {
                active.dragging = true;
            }
        },
    );
    children.self_observe(
        current_widget_val,
        move |_trigger: On<Pointer<DragEnd>>, mut active_query: Query<&mut DockDragActive>| {
            if let Ok(mut active) = active_query.get_mut(drag_active_entity) {
                active.dragging = false;
            }
        },
    );
    attach_revert_on_unconsumed_drag(
        &mut children,
        current_widget_val,
        drag_state_entity,
        header.panel.clone(),
    );

    let ghost_title = header.title.clone();
    let ghost_position = drag_state_query
        .get(drag_state_entity)
        .map(|s| s.position)
        .unwrap_or_default();
    children.drag_ghost(
        overlay_root_val,
        dragging,
        (
            WoodpeckerStyle {
                position: WidgetPosition::Fixed,
                left: ghost_position.x.into(),
                top: ghost_position.y.into(),
                padding: Edge::all(0.0).left(10.0).right(10.0).top(6.0).bottom(6.0),
                background_color: styles.active_background,
                border_color: styles.active_border,
                border: Edge::all(1.0),
                border_radius: Corner::all(4.0),
                z_index: Some(WidgetZ::Global(u32::MAX)),
                ..Default::default()
            },
            WidgetRender::Quad,
            Pickable::IGNORE,
            WidgetChildren::default().with_child::<Element>((
                Element,
                WoodpeckerStyle {
                    font_size: styles.font_size,
                    color: styles.active_text_color,
                    text_wrap: TextWrap::None,
                    ..Default::default()
                },
                WidgetRender::Text {
                    content: ghost_title,
                },
            )),
        ),
    );

    children.add::<Element>((
        Element,
        WoodpeckerStyle {
            font_size: styles.font_size,
            color: if header.active {
                styles.active_text_color
            } else {
                styles.text_color
            },
            text_wrap: TextWrap::None,
            flex_grow: 1.0,
            ..Default::default()
        },
        WidgetRender::Text {
            content: header.title.clone(),
        },
    ));
    children.add_key("title");

    if header.closable {
        let panel_for_close = header.panel.clone();
        children.add::<Element>((
            Element,
            WoodpeckerStyle {
                margin: Edge::all(0.0).left(8.0),
                font_size: styles.font_size,
                color: styles.text_color,
                font: Some(icon_font.0.id()),
                ..Default::default()
            },
            Pickable::default(),
            WidgetRender::Text {
                content: icons::X.into(),
            },
        ));
        children.observe(
            current_widget_val,
            move |mut trigger: On<Pointer<Click>>, mut tree_query: Query<&mut DockTree>| {
                // Closing a tab must not also activate it.
                trigger.propagate(false);
                if let Ok(mut tree) = tree_query.get_mut(tree_entity) {
                    tree.remove_panel(&panel_for_close);
                }
            },
        );
        children.add_key("close");
    }

    children.apply(current_widget.as_parent());
}
