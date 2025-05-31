#![allow(dead_code)]

use bevy::{ecs::system::SystemParam, platform::collections::HashMap, prelude::*};
use bevy_trait_query::One;
use taffy::Layout;

use crate::{
    context::{Widget, WoodpeckerContext},
    font::FontManager,
    hook_helper::StateMarker,
    prelude::{PreviousWidget, WidgetPosition, WidgetRender},
    styles::Edge,
    svg::SvgAsset,
    DefaultFont,
};

use super::{measure::LayoutMeasure, UiLayout, WoodpeckerStyle};

#[derive(Debug, Copy, Clone, Reflect, Default)]
pub struct ReflectedLayout {
    /// The z value of the node.
    /// This can be adjusted by the user to render nodes ontop of nodes
    /// in the tree regardless of order.
    pub z: u32,
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
            z: 0,
            order: 0,
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
#[derive(Component, Debug, Clone, Copy, Deref, DerefMut, Reflect, Default)]
pub struct WidgetLayout(pub ReflectedLayout);

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
pub struct WidgetPreviousLayout(pub ReflectedLayout);

impl PartialEq for WidgetPreviousLayout {
    fn eq(&self, other: &Self) -> bool {
        layout_equality(self, other)
    }
}

#[derive(SystemParam)]
pub(crate) struct LayoutSystemParam<'w, 's> {
    commands: Commands<'w, 's>,
    default_font: Res<'w, DefaultFont>,
    font_manager: ResMut<'w, FontManager>,
    ui_layout: ResMut<'w, UiLayout>,
    query: Query<
        'w,
        's,
        (
            Entity,
            One<&'static dyn Widget>,
            &'static WoodpeckerStyle,
            Option<&'static ChildOf>,
            Option<&'static Children>,
        ),
        (Without<StateMarker>, Without<PreviousWidget>),
    >,
    state_marker_query: Query<'w, 's, &'static StateMarker>,
    prev_marker_query: Query<'w, 's, &'static PreviousWidget>,
    children_query: Query<
        'w,
        's,
        (Entity, &'static Children, One<&'static dyn Widget>),
        (Changed<Children>, Without<PreviousWidget>),
    >,
    layout_query: Query<'w, 's, &'static WidgetLayout>,
    widget_render: Query<'w, 's, &'static WidgetRender>,
    context: Res<'w, WoodpeckerContext>,
    image_assets: Res<'w, Assets<Image>>,
    svg_assets: Res<'w, Assets<SvgAsset>>,
    removed_widgets: RemovedComponents<'w, 's, WidgetLayout>,
}

// TODO: Document how layouting works..
pub(crate) fn run(layout_system_param: LayoutSystemParam) {
    let LayoutSystemParam {
        mut commands,
        default_font,
        mut font_manager,
        mut ui_layout,
        state_marker_query,
        query,
        prev_marker_query,
        children_query,
        layout_query,
        widget_render,
        context,
        image_assets,
        svg_assets,
        mut removed_widgets,
    } = layout_system_param;

    let root_node = context.get_root_widget();
    ui_layout.root_entity = root_node;

    for entity in removed_widgets.read() {
        ui_layout.remove_child(entity);
    }

    // This needs to be in the correct order
    traverse_upsert_node(
        root_node,
        &query,
        &widget_render,
        &default_font,
        &mut font_manager,
        &image_assets,
        &svg_assets,
        &mut ui_layout,
        root_node,
        Vec2::new(1.0, 1.0),
    );

    for (entity, children, _) in children_query.iter() {
        let normal_children = children
            .iter()
            // We only want to add non-fixed entities as children
            .filter(|child| {
                let Ok((_, _, styles, _, _)) = query.get(*child) else {
                    return false;
                };
                !matches!(styles.position, WidgetPosition::Fixed)
            })
            .filter(|child| {
                !state_marker_query.contains(*child) && !prev_marker_query.contains(*child)
            })
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

    // TODO(PERF): Figure out how we can combine traversal and compute together..
    let mut order = 0;
    let mut cache = HashMap::default();
    traverse_layout_update(
        &mut commands,
        root_node,
        &ui_layout,
        &query,
        &layout_query,
        &mut cache,
        &mut order,
        0,
    );
}

fn traverse_layout_update(
    commands: &mut Commands,
    entity: Entity,
    ui_layout: &UiLayout,
    query: &Query<
        (
            Entity,
            One<&dyn Widget>,
            &WoodpeckerStyle,
            Option<&ChildOf>,
            Option<&Children>,
        ),
        (Without<StateMarker>, Without<PreviousWidget>),
    >,
    layout_query: &Query<&WidgetLayout>,
    cache: &mut HashMap<Entity, Layout>,
    order: &mut u32,
    parent_id: u32,
) {
    let Ok((entity, _, styles, parent, children)) = query.get(entity) else {
        return;
    };
    if let Some(layout) = ui_layout.get_layout(entity) {
        let mut layout = *layout;
        if let Ok(prev_layout) = layout_query.get(entity) {
            commands
                .entity(entity)
                .insert(WidgetPreviousLayout(prev_layout.0));
        }

        if let Some(parent_layout) = parent.map(|parent| {
            cache
                .get(&parent.parent())
                .unwrap_or(ui_layout.get_layout(parent.parent()).unwrap())
        }) {
            if styles.position != WidgetPosition::Fixed {
                layout.location.x += parent_layout.location.x;
                layout.location.y += parent_layout.location.y;
            }
        }

        cache.insert(entity, layout);
        let mut layout = WidgetLayout((&layout).into());
        layout.order = ((*order as i32)
            + styles
                .z_index
                .map(|z| z.get_relative())
                .flatten()
                .unwrap_or(0)) as u32;
        layout.z = styles
            .z_index
            .map(|z| z.get_global())
            .flatten()
            .unwrap_or(parent_id);
        *order += 1;
        commands.entity(entity).insert(layout);

        let Some(children) = children.map(|c| c.iter().collect::<Vec<_>>()) else {
            return;
        };

        for child in children.iter() {
            traverse_layout_update(
                commands,
                *child,
                ui_layout,
                query,
                layout_query,
                cache,
                order,
                layout.z,
            );
        }
    }
}

fn traverse_upsert_node(
    root_node: Entity,
    query: &Query<
        (
            Entity,
            One<&dyn Widget>,
            &WoodpeckerStyle,
            Option<&ChildOf>,
            Option<&Children>,
        ),
        (Without<StateMarker>, Without<PreviousWidget>),
    >,
    query_widget_render: &Query<&WidgetRender>,
    default_font: &DefaultFont,
    font_manager: &mut FontManager,
    image_assets: &Assets<Image>,
    svg_assets: &Assets<SvgAsset>,
    layout: &mut UiLayout,
    current_node: Entity,
    camera_scale: Vec2,
) {
    let Ok((entity, _, styles, parent, children)) = query.get(current_node) else {
        return;
    };

    let layout_measure = if let Ok(widget_render) = query_widget_render.get(entity) {
        if let Some(parent_layout) = if let Some(parent_entity) = parent {
            layout.get_layout(parent_entity.parent())
        } else {
            layout.get_layout(root_node)
        } {
            let widget_layout = WidgetLayout(ReflectedLayout::from(parent_layout));
            match_render_size(
                font_manager,
                image_assets,
                svg_assets,
                default_font,
                widget_render,
                styles,
                &widget_layout,
                camera_scale,
            )
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
            image_assets,
            svg_assets,
            layout,
            child,
            camera_scale,
        );
    }
}

fn match_render_size(
    font_manager: &mut FontManager,
    image_assets: &Assets<Image>,
    svg_assets: &Assets<SvgAsset>,
    default_font: &DefaultFont,
    widget_render: &WidgetRender,
    styles: &WoodpeckerStyle,
    parent_layout: &WidgetLayout,
    camera_scale: Vec2,
) -> Option<LayoutMeasure> {
    match widget_render {
        WidgetRender::Image { handle } => {
            let image = image_assets.get(handle)?;

            let size = image.size().as_vec2();

            Some(LayoutMeasure::Image(super::measure::ImageMeasure { size }))
        }
        WidgetRender::Svg { handle, .. } => {
            let svg_asset = svg_assets.get(handle)?;

            let size = Vec2::new(svg_asset.width, svg_asset.height);
            Some(LayoutMeasure::Image(super::measure::ImageMeasure { size }))
        }
        WidgetRender::RichText { content } => measure_text(
            &content.text,
            styles,
            font_manager,
            default_font,
            parent_layout,
            camera_scale,
        ),
        WidgetRender::Text { content } => measure_text(
            content,
            styles,
            font_manager,
            default_font,
            parent_layout,
            camera_scale,
        ),
        _ => None,
    }
}

pub(crate) fn measure_text(
    text: &str,
    styles: &WoodpeckerStyle,
    font_manager: &mut FontManager,
    default_font: &DefaultFont,
    parent_layout: &WidgetLayout,
    camera_scale: Vec2,
) -> Option<LayoutMeasure> {
    // Measure text
    // TODO: Cache this.
    let mut layout_editor = parley::PlainEditor::new(styles.font_size);
    layout_editor.set_text(text);
    let text_styles = layout_editor.edit_styles();
    text_styles.insert(parley::StyleProperty::LineHeight(
        styles
            .line_height
            .map(|lh| styles.font_size / lh)
            .unwrap_or(1.2),
    ));
    text_styles.insert(parley::StyleProperty::FontStack(parley::FontStack::Single(
        parley::FontFamily::Named(
            font_manager
                .get_family(styles.font.as_ref().unwrap_or(&default_font.0.id()))
                .into(),
        ),
    )));

    text_styles.insert(parley::StyleProperty::OverflowWrap(
        match styles.text_wrap {
            crate::styles::TextWrap::None => parley::OverflowWrap::Normal,
            crate::styles::TextWrap::Glyph => parley::OverflowWrap::Anywhere,
            crate::styles::TextWrap::Word => parley::OverflowWrap::BreakWord,
            crate::styles::TextWrap::WordOrGlyph => parley::OverflowWrap::Anywhere,
        },
    ));
    layout_editor.set_width(Some(parent_layout.size.x * camera_scale.x));
    let alignment = match styles
        .text_alignment
        .unwrap_or(crate::font::TextAlign::Left)
    {
        crate::font::TextAlign::Left => parley::Alignment::Left,
        crate::font::TextAlign::Right => parley::Alignment::Right,
        crate::font::TextAlign::Center => parley::Alignment::Middle,
        crate::font::TextAlign::Justified => parley::Alignment::Justified,
        crate::font::TextAlign::End => parley::Alignment::End,
    };
    layout_editor.set_alignment(alignment);
    let text_layout = layout_editor.layout(&mut font_manager.font_cx, &mut font_manager.layout_cx);

    if !text_layout.is_empty() {
        let mut size = Vec2::new(0.0, 0.0);
        text_layout.lines().for_each(|l| {
            let line_metrics = l.metrics();
            size.x = size.x.max(line_metrics.advance + 1.0);
            size.y += line_metrics.line_height;
        });
        Some(LayoutMeasure::Fixed(super::measure::FixedMeasure { size }))
    } else {
        None
    }
}
