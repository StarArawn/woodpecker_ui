use bevy::prelude::*;

use crate::prelude::*;

use super::drop_zone::DockDropSurface;
use super::tab_bar::DockTabBar;
use super::tree::{DockAxis, DockNode, DockTree, NodePath};
use super::DockStyles;

/// Everything [`render_dock_node`] needs that isn't already implied by the node/path it's
/// currently walking -- threaded through the recursion instead of re-fetched at every level.
#[derive(Clone, Copy)]
pub(super) struct DockRenderCtx {
    pub current_widget: CurrentWidget,
    pub tree_entity: Entity,
    pub styles: DockStyles,
}

/// The ratio pair a dock `Splitter` is resizing, captured once at `Pointer<DragStart>` and read
/// back on every `Change<SplitterChanged>` for the rest of that drag gesture -- see
/// `DockTree::set_split_ratio_from_base`'s own doc comment for why this can't just be "the
/// current ratio plus this event's delta". A single shared `Resource` rather than a per-splitter
/// entity -- only one splitter can be dragged at a time in a single-pointer desktop UI.
#[derive(Resource, Default, Clone, Copy)]
pub(crate) struct DockSplitDragBase {
    ratios: Option<(f32, f32)>,
}

/// Recursively builds the widget tree for `node` (at `path` within the owning [`DockTree`]).
/// Every node -- `Split` or `Tabs` -- returns exactly one wrapping `Element`, so a caller (this
/// function's own `Split` branch, or [`super::DockArea`]'s root) never needs to special-case
/// which kind of node it just asked for.
pub(super) fn render_dock_node(
    node: &DockNode,
    path: NodePath,
    ctx: DockRenderCtx,
) -> WidgetChildren {
    let mut wrapper = WidgetChildren::default();
    match node {
        DockNode::Split {
            axis,
            children: split_children,
        } => {
            let axis = *axis;
            let mut inner = WidgetChildren::default();

            for (index, (child_node, ratio)) in split_children.iter().enumerate() {
                let child_path = path.child(index);
                let (width, height) = match axis {
                    DockAxis::Row => (Units::Auto, Units::Percentage(100.0)),
                    DockAxis::Column => (Units::Percentage(100.0), Units::Auto),
                };
                inner.add::<Element>((
                    Element,
                    WoodpeckerStyle {
                        flex_grow: ratio.max(0.0001),
                        flex_shrink: 1.0,
                        flex_basis: Units::Pixels(0.0),
                        width,
                        height,
                        margin: Edge::all(ctx.styles.panel_gap / 2.0),
                        overflow: WidgetOverflow::Hidden,
                        ..Default::default()
                    },
                    render_dock_node(child_node, child_path, ctx),
                ));
                inner.add_key(format!("node_{index}"));

                if index + 1 < split_children.len() {
                    inner.add::<Splitter>((Splitter {
                        vertical: axis == DockAxis::Row,
                    },));
                    inner.add_key(format!(
                        "split_{index}_{}",
                        if axis == DockAxis::Row { "row" } else { "col" }
                    ));

                    let tree_entity = ctx.tree_entity;
                    let split_path = path.clone();
                    let child_index = index;
                    let split_path_for_start = split_path.clone();
                    inner.observe(
                        ctx.current_widget,
                        move |_trigger: On<Pointer<DragStart>>,
                              mut base: ResMut<DockSplitDragBase>,
                              tree_query: Query<&DockTree>| {
                            base.ratios = tree_query
                                .get(tree_entity)
                                .ok()
                                .and_then(|tree| tree.node_at(&split_path_for_start))
                                .and_then(|node| match node {
                                    DockNode::Split { children, .. }
                                        if child_index + 1 < children.len() =>
                                    {
                                        Some((children[child_index].1, children[child_index + 1].1))
                                    }
                                    _ => None,
                                });
                        },
                    );

                    let tree_entity_for_change = tree_entity;
                    let split_path_for_change = split_path.clone();
                    inner.observe(
                        ctx.current_widget,
                        move |trigger: On<Change<SplitterChanged>>,
                              mut tree_query: Query<&mut DockTree>,
                              parent_query: Query<&ChildOf>,
                              layout_query: Query<&WidgetLayout>,
                              base: Res<DockSplitDragBase>| {
                            let Some((base_a, base_b)) = base.ratios else {
                                return;
                            };
                            let Ok(mut tree) = tree_query.get_mut(tree_entity_for_change) else {
                                return;
                            };
                            let extent = parent_query
                                .get(trigger.target)
                                .ok()
                                .and_then(|child_of| layout_query.get(child_of.parent()).ok())
                                .map(|layout| {
                                    if axis == DockAxis::Row {
                                        layout.size.x
                                    } else {
                                        layout.size.y
                                    }
                                })
                                .unwrap_or(1.0);
                            tree.set_split_ratio_from_base(
                                &split_path_for_change,
                                child_index,
                                base_a,
                                base_b,
                                trigger.data.delta,
                                extent,
                            );
                        },
                    );
                }
            }

            wrapper.add::<Element>((
                Element,
                WoodpeckerStyle {
                    width: Units::Percentage(100.0),
                    height: Units::Percentage(100.0),
                    flex_direction: match axis {
                        DockAxis::Row => WidgetFlexDirection::Row,
                        DockAxis::Column => WidgetFlexDirection::Column,
                    },
                    ..Default::default()
                },
                inner,
            ));
            wrapper.add_key("split");
        }
        DockNode::Tabs { panels, active } => {
            let mut inner = WidgetChildren::default();

            inner.add::<DockTabBar>((DockTabBar {
                panels: panels.clone(),
                active: *active,
                node_path: path.clone(),
            },));
            inner.add_key("tab_bar");

            inner.add::<DockDropSurface>((
                DockDropSurface {
                    node_path: path.clone(),
                    active_panel: panels.get(*active).cloned(),
                },
                WoodpeckerStyle {
                    width: Units::Percentage(100.0),
                    flex_grow: 1.0,
                    ..Default::default()
                },
            ));
            inner.add_key("content");

            wrapper.add::<Element>((
                Element,
                WoodpeckerStyle {
                    width: Units::Percentage(100.0),
                    height: Units::Percentage(100.0),
                    flex_direction: WidgetFlexDirection::Column,
                    border: Edge::all(1.0),
                    border_color: ctx.styles.panel_border,
                    border_radius: Corner::all(ctx.styles.panel_border_radius),
                    overflow: WidgetOverflow::Hidden,
                    ..Default::default()
                },
                WidgetRender::Quad,
                inner,
            ));
            wrapper.add_key("tabs");
        }
    }
    wrapper
}
