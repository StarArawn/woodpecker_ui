use crate::{
    convert_render_target::RenderTargetImages,
    hook_helper::StateMarker,
    image::ImageManager,
    layout::system::ReflectedLayout,
    prelude::*,
    svg::{SvgAsset, SvgManager},
    DefaultFont,
};
use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_trait_query::One;
use bevy_vello::{prelude::VelloFont, VelloScene};

#[derive(SystemParam)]
pub(crate) struct RenderSystemParam<'w, 's> {
    default_font: Res<'w, DefaultFont>,
    font_manager: ResMut<'w, FontManager>,
    svg_manager: ResMut<'w, SvgManager>,
    image_manager: ResMut<'w, ImageManager>,
    render_targets: ResMut<'w, RenderTargetImages>,
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
    layout_query: Query<'w, 's, &'static WidgetLayout>,
    vello_query: Query<'w, 's, &'static mut VelloScene>,
    widget_render: Query<'w, 's, &'static WidgetRender>,
    context: Res<'w, WoodpeckerContext>,
    font_assets: Res<'w, Assets<VelloFont>>,
    image_assets: ResMut<'w, Assets<Image>>,
    svg_assets: Res<'w, Assets<SvgAsset>>,
    metrics: ResMut<'w, WidgetMetrics>,
    camera_query: Query<'w, 's, &'static Camera, With<WoodpeckerView>>,
}

// TODO: Document how renderer works
pub(crate) fn run(renderer_system_param: RenderSystemParam) {
    let RenderSystemParam {
        default_font,
        mut font_manager,
        mut svg_manager,
        mut image_manager,
        mut render_targets,
        mut query,
        layout_query,
        mut vello_query,
        widget_render,
        context,
        font_assets,
        mut image_assets,
        svg_assets,
        mut metrics,
        camera_query,
    } = renderer_system_param;

    let Ok(mut vello_scene) = vello_query.single_mut() else {
        error!("Woodpecker UI: No vello scene spawned!");
        return;
    };

    let Ok(camera) = camera_query.single() else {
        error!("Woodpecker UI: No camera found or multiple UI cameras found.");
        return;
    };

    let camera_scale = Vec2::new(
        camera.target_scaling_factor().unwrap_or(1.0),
        camera.target_scaling_factor().unwrap_or(1.0),
    );

    let camera_size = camera
        .physical_target_size()
        .unwrap_or(UVec2::ZERO)
        .as_vec2();

    vello_scene.reset();

    metrics.clear_quad_last_frame();

    let root_node = context.get_root_widget();
    let mut render_commands = vec![];
    let mut order = 0;
    // After layout computations update layouts and render scene.
    // Needs to be done in the correct order..
    // We also need to know if we are going back up the tree so we can pop the clipping and opacity layers.
    traverse_render_tree(
        root_node,
        0,
        &mut order,
        &mut render_commands,
        &mut query,
        &default_font,
        &mut font_manager,
        &mut svg_manager,
        &mut image_manager,
        &mut render_targets,
        &mut metrics,
        &widget_render,
        &mut vello_scene,
        &font_assets,
        &mut image_assets,
        &svg_assets,
        &layout_query,
        root_node,
        true,
        camera_scale,
        camera_size,
    );

    // Once tree is traversed we sort the commands
    render_commands.sort_unstable_by(|a, b| a.z.cmp(&b.z).then_with(|| a.order.cmp(&b.order)));

    // Now we can render with vello
    for command in render_commands {
        // dbg!((command.widget_render.to_string(), command.z, command.order));
        command.widget_render.render(
            &mut vello_scene,
            &command.layout,
            &command.parent_layout,
            &default_font,
            &font_assets,
            &mut image_assets,
            &svg_assets,
            &mut font_manager,
            &mut svg_manager,
            &mut image_manager,
            &mut render_targets,
            &mut metrics,
            &command.styles,
            camera_scale,
            camera_size,
        );
    }

    metrics.commit_quad_frame();
}

fn traverse_render_tree(
    root_node: Entity,
    parent_id: u32,
    order_counter: &mut u32,
    render_commands: &mut Vec<RenderCommand>,
    query: &mut Query<
        (
            Entity,
            One<&dyn Widget>,
            &WoodpeckerStyle,
            Option<&ChildOf>,
            Option<&Children>,
        ),
        (Without<StateMarker>, Without<PreviousWidget>),
    >,
    default_font: &DefaultFont,
    font_manager: &mut FontManager,
    svg_manager: &mut SvgManager,
    image_manager: &mut ImageManager,
    render_targets: &mut RenderTargetImages,
    metrics: &mut WidgetMetrics,
    widget_render: &Query<&WidgetRender>,
    vello_scene: &mut VelloScene,
    font_assets: &Assets<VelloFont>,
    image_assets: &mut Assets<Image>,
    svg_assets: &Assets<SvgAsset>,
    layout_query: &Query<&WidgetLayout>,
    current_node: Entity,
    should_render: bool,
    camera_scale: Vec2,
    camera_size: Vec2,
) {
    let Ok((entity, _, styles, parent, children)) = query.get_mut(current_node) else {
        return;
    };

    let Ok(layout) = layout_query.get(entity) else {
        return;
    };

    if matches!(styles.display, WidgetDisplay::None)
        || matches!(styles.visibility, WidgetVisibility::Hidden)
    {
        return;
    }

    let z_index = styles.z_index;
    let z = z_index.unwrap_or(parent_id);
    let mut order = *order_counter;
    *order_counter += 1;

    let mut did_layer = false;
    if let Ok(widget_render) = widget_render.get(entity) {
        let parent_layout = parent.map(|parent| *layout_query.get(parent.parent()).unwrap());
        if (parent_layout.is_some() || root_node == entity) && should_render {
            if matches!(widget_render, WidgetRender::Layer) {
                did_layer = true;
            }

            if styles.opacity > 0.0 && styles.opacity < 1.0 && !did_layer {
                did_layer = true;
                render_commands.push(RenderCommand {
                    z,
                    order,
                    widget_render: WidgetRender::Layer,
                    layout: WidgetLayout(ReflectedLayout {
                        location: Vec2::splat(0.0),
                        size: Vec2::splat(10000.0),
                        ..Default::default()
                    }),
                    styles: *styles,
                    ..Default::default()
                });

                order = *order_counter;
                *order_counter += 1;
            }

            render_commands.push(RenderCommand {
                z,
                order,
                layout: *layout,
                parent_layout: parent_layout.unwrap_or_default(),
                widget_render: widget_render.clone(),
                styles: *styles,
            });
        }
    }

    let Some(children) = children.map(|c| c.iter().collect::<Vec<_>>()) else {
        if did_layer {
            let order = *order_counter;
            // vello_scene.pop_layer();
            render_commands.push(RenderCommand {
                z,
                order,
                widget_render: WidgetRender::PopLayer,
                ..Default::default()
            });
        }
        return;
    };

    for child in children.iter() {
        traverse_render_tree(
            root_node,
            z,
            order_counter,
            render_commands,
            query,
            default_font,
            font_manager,
            svg_manager,
            image_manager,
            render_targets,
            metrics,
            widget_render,
            vello_scene,
            font_assets,
            image_assets,
            svg_assets,
            layout_query,
            *child,
            should_render,
            camera_scale,
            camera_size,
        );
    }

    if did_layer {
        // vello_scene.pop_layer();
        let order = *order_counter;
        render_commands.push(RenderCommand {
            z,
            order,
            widget_render: WidgetRender::PopLayer,
            ..Default::default()
        });
    }
}

struct RenderCommand {
    z: u32,
    order: u32,
    layout: WidgetLayout,
    parent_layout: WidgetLayout,
    widget_render: WidgetRender,
    styles: WoodpeckerStyle,
}

impl Default for RenderCommand {
    fn default() -> Self {
        Self {
            z: 0,
            order: 0,
            layout: Default::default(),
            parent_layout: Default::default(),
            widget_render: Default::default(),
            styles: Default::default(),
        }
    }
}
