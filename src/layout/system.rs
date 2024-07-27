#![allow(dead_code)]

use bevy::{prelude::*, utils::HashMap};
use bevy_trait_query::One;
use bevy_vello::{text::VelloFont, VelloScene};
use taffy::Layout;

use crate::{
    context::{Widget, WoodpeckerContext},
    font::FontManager,
    hook_helper::StateMarker,
    prelude::{PreviousWidget, WidgetPosition, WidgetRender},
    styles::Edge,
    DefaultFont,
};

use super::{measure::LayoutMeasure, UiLayout, WoodpeckerStyle};

#[derive(Debug, Copy, Clone, Reflect)]
pub struct ReflectedLayout {
    /// The relative ordering of the node
    ///
    /// Nodes with a higher order should be rendered on top of those with a lower order.
    /// This is effectively a topological sort of each tree.
    pub order: u32,
    /// The top-left corner of the node
    pub location: Vec2,
    /// The width and height of the node
    pub size: Vec2,
    /// The width and height of the content inside the node. This may be larger than the size of the node in the case of
    /// overflowing content and is useful for computing a "scroll width/height" for scrollable nodes
    pub content_size: Vec2,
    /// The size of the scrollbars in each dimension. If there is no scrollbar then the size will be zero.
    pub scrollbar_size: Vec2,
    /// The size of the borders of the node
    pub border: Edge,
    /// The size of the padding of the node
    pub padding: Edge,
}

impl From<&Layout> for ReflectedLayout {
    fn from(value: &Layout) -> Self {
        Self {
            order: value.order,
            location: Vec2::new(value.location.x, value.location.y),
            size: Vec2::new(value.size.width, value.size.height),
            content_size: Vec2::new(value.content_size.width, value.content_size.height),
            scrollbar_size: Vec2::new(value.scrollbar_size.width, value.scrollbar_size.height),
            border: Edge::new(
                value.border.top,
                value.border.right,
                value.border.bottom,
                value.border.left,
            ),
            padding: Edge::new(
                value.padding.top,
                value.padding.right,
                value.padding.bottom,
                value.padding.left,
            ),
        }
    }
}

/// A widget's layout
/// This is built by taffy and included as a component on
/// your widgets automatically when taffy computes layout logic.
#[derive(Component, Debug, Clone, Copy, Deref, DerefMut, Reflect)]
pub struct WidgetLayout(ReflectedLayout);

impl WidgetLayout {
    /// The position of the widget in pixels
    pub fn position(&self) -> Vec2 {
        self.location
    }

    /// The width of the layout in pixels
    pub fn width(&self) -> f32 {
        self.0.size.x
    }

    /// The height of the layout in pixels
    pub fn height(&self) -> f32 {
        self.0.size.y
    }

    /// The content width of the layout in pixels
    ///
    /// Not to be confused with width or height this measurement is the amount of space
    /// the children take up.
    pub fn content_width(&self) -> f32 {
        self.0.content_size.x
    }

    /// The content height of the layout in pixels
    ///
    /// Not to be confused with width or height this measurement is the amount of space
    /// the children take up.
    pub fn content_height(&self) -> f32 {
        self.0.content_size.y
    }
}

// TODO: Add more here..
fn layout_equality(layout_a: &ReflectedLayout, layout_b: &ReflectedLayout) -> bool {
    layout_a.size == layout_b.size
        && layout_a.location == layout_b.location
        && layout_a.content_size == layout_b.content_size
}

impl std::cmp::PartialEq<WidgetLayout> for WidgetPreviousLayout {
    fn eq(&self, other: &WidgetLayout) -> bool {
        layout_equality(self, other)
    }
}

impl PartialEq for WidgetLayout {
    fn eq(&self, other: &Self) -> bool {
        layout_equality(self, other)
    }
}

/// The previous layout from the last frame.
/// Useful in some cases to see if a widget's layout has
/// changed.
#[derive(Component, Debug, Clone, Copy, Deref, DerefMut, Reflect)]
pub struct WidgetPreviousLayout(ReflectedLayout);

impl PartialEq for WidgetPreviousLayout {
    fn eq(&self, other: &Self) -> bool {
        layout_equality(self, other)
    }
}

// TODO: Document how layouting works..
pub(crate) fn run(
    mut commands: Commands,
    default_font: Res<DefaultFont>,
    mut font_manager: ResMut<FontManager>,
    mut ui_layout: ResMut<UiLayout>,
    mut query: Query<
        (
            Entity,
            One<&dyn Widget>,
            &WoodpeckerStyle,
            Option<&Parent>,
            Option<&Children>,
        ),
        (Without<StateMarker>, Without<PreviousWidget>),
    >,
    state_marker_query: Query<&StateMarker>,
    prev_marker_query: Query<&PreviousWidget>,
    layout_query: Query<&WidgetLayout>,
    children_query: Query<(Entity, &Children, One<&dyn Widget>), (Changed<Children>, Without<PreviousWidget>)>,
    mut vello_query: Query<&mut VelloScene>,
    widget_render: Query<&WidgetRender>,
    context: Res<WoodpeckerContext>,
    font_assets: Res<Assets<VelloFont>>,
    image_assets: Res<Assets<Image>>,
) {
    let Ok(mut vello_scene) = vello_query.get_single_mut() else {
        error!("Woodpecker UI: No vello scene spawned!");
        return;
    };
    vello_scene.reset();

    let root_node = context.get_root_widget();
    // This needs to be in the correct order
    // TODO: This probably doesn't need to be in the correct order..
    traverse_upsert_node(
        root_node,
        &query,
        &widget_render,
        &default_font,
        &mut font_manager,
        &mut ui_layout,
        root_node,
    );

    for (entity, children, _) in children_query.iter() {
        let normal_children = children
            .iter()
            // We only want to add non-fixed entities as children
            .filter(|child| {
                let Ok((_, _, styles, _, _)) = query.get(**child) else {
                    return true;
                };
                !matches!(styles.position, WidgetPosition::Fixed)
            })
            .filter(|child| {
                !state_marker_query.contains(**child) && !prev_marker_query.contains(**child)
            })
            .copied()
            .collect::<Vec<_>>();
        ui_layout.add_children(entity, &normal_children);

        // Add fixed children to the root node.
        for child in children {
            let Ok((_, _, styles, _, _)) = query.get(*child) else {
                continue;
            };
            if styles.position == WidgetPosition::Fixed {
                ui_layout.add_child(root_node, *child);
            }
        }
    }

    let Ok((width, height)) = query
        .get(root_node)
        .map(|(_, _, style, _, _)| (style.width.value_or(1.0), style.height.value_or(1.0)))
    else {
        return;
    };

    ui_layout.compute(root_node, Vec2::new(width, height));

    let mut cached_layout = HashMap::default();

    let mut order = 0;

    // After layout computations update layouts and render scene.
    // Needs to be done in the correct order..
    // We also need to know if we are going back up the tree so we can pop the clipping and opacity layers.
    traverse_render_tree(
        root_node,
        &mut query,
        &default_font,
        &mut font_manager,
        &widget_render,
        &mut cached_layout,
        &mut vello_scene,
        &font_assets,
        &image_assets,
        &ui_layout,
        root_node,
        &mut order,
    );

    for (entity, layout) in cached_layout.iter() {
        if let Ok(prev_layout) = layout_query.get(*entity) {
            commands
                .entity(*entity)
                .insert(WidgetPreviousLayout(prev_layout.0));
        }
        commands.entity(*entity).insert(WidgetLayout(layout.into()));
    }
}

fn traverse_render_tree(
    root_node: Entity,
    query: &mut Query<
        (
            Entity,
            One<&dyn Widget>,
            &WoodpeckerStyle,
            Option<&Parent>,
            Option<&Children>,
        ),
        (Without<StateMarker>, Without<PreviousWidget>),
    >,
    default_font: &DefaultFont,
    font_manager: &mut FontManager,
    widget_render: &Query<&WidgetRender>,
    cached_layout: &mut HashMap<Entity, Layout>,
    vello_scene: &mut VelloScene,
    font_assets: &Assets<VelloFont>,
    image_assets: &Assets<Image>,
    ui_layout: &UiLayout,
    current_node: Entity,
    order: &mut u32,
) {
    let Ok((entity, _, styles, parent, children)) = query.get_mut(current_node) else {
        return;
    };
    let Some(layout) = ui_layout.get_layout(entity).cloned() else {
        return;
    };

    let mut layout = layout;
    if let Some(parent_layout) = parent.map(|parent| {
        cached_layout
            .get(&parent.get())
            .copied()
            .unwrap_or_else(|| *ui_layout.get_layout(parent.get()).unwrap())
    }) {
        if styles.position != WidgetPosition::Fixed {
            layout.location.x += parent_layout.location.x;
            layout.location.y += parent_layout.location.y;
        }
    }

    layout.order = *order;

    let mut did_layer = false;
    if let Ok(widget_render) = widget_render.get(entity) {
        let parent_layout = parent.map(|parent| {
            cached_layout
                .get(&parent.get())
                .copied()
                .unwrap_or_else(|| *ui_layout.get_layout(parent.get()).unwrap())
        });
        if parent_layout.is_some() || root_node == entity {
            did_layer = widget_render.render(
                vello_scene,
                &layout,
                &parent_layout.unwrap_or_default(),
                default_font,
                font_assets,
                font_manager,
                image_assets,
                styles,
            );
        }
    }
    cached_layout.insert(entity, layout);

    let Some(children) = children.map(|c| c.iter().copied().collect::<Vec<_>>()) else {
        if did_layer {
            vello_scene.pop_layer();
        }
        return;
    };

    for child in children.iter() {
        *order += 1;
        traverse_render_tree(
            root_node,
            query,
            default_font,
            font_manager,
            widget_render,
            cached_layout,
            vello_scene,
            font_assets,
            image_assets,
            ui_layout,
            *child,
            order,
        );
        *order -= 1;
    }

    if did_layer {
        vello_scene.pop_layer();
    }
}

fn traverse_upsert_node(
    root_node: Entity,
    query: &Query<
        (
            Entity,
            One<&dyn Widget>,
            &WoodpeckerStyle,
            Option<&Parent>,
            Option<&Children>,
        ),
        (Without<StateMarker>, Without<PreviousWidget>),
    >,
    query_widget_render: &Query<&WidgetRender>,
    default_font: &DefaultFont,
    font_manager: &mut FontManager,
    layout: &mut UiLayout,
    current_node: Entity,
) {
    let Ok((entity, _, styles, parent, children)) = query.get(current_node) else {
        return;
    };

    let layout_measure = if let Ok(widget_render) = query_widget_render.get(entity) {
        if let Some(parent_layout) = if let Some(parent_entity) = parent {
            layout.get_layout(parent_entity.get())
        } else {
            layout.get_layout(root_node)
        } {
            // TODO: Move to a function so we can get rid of this stupid nesting...
            if let WidgetRender::Text { content, word_wrap } = widget_render {
                // Measure text
                let font_handle = styles
                    .font
                    .as_ref()
                    .map(|a| Handle::Weak(*a))
                    .unwrap_or(default_font.0.clone());
                if let Some(buffer) = font_manager.layout(
                    Vec2::new(
                        parent_layout.size.width,
                        parent_layout.size.height + 100000.0,
                    ),
                    styles,
                    &font_handle,
                    content,
                    *word_wrap,
                ) {
                    let mut size = Vec2::new(0.0, 0.0);
                    buffer.layout_runs().for_each(|r| {
                        size.x = size.x.max(r.line_w);
                        size.y += r.line_height;
                    });
                    Some(LayoutMeasure::Fixed(super::measure::FixedMeasure { size }))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    layout.upsert_node(entity, styles, layout_measure);
    let Some(children) = children else {
        return;
    };
    for child in children.iter() {
        traverse_upsert_node(
            root_node,
            query,
            query_widget_render,
            default_font,
            font_manager,
            layout,
            *child,
        );
    }
}
