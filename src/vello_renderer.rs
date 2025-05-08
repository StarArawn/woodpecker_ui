use crate::{
    hook_helper::StateMarker,
    image::ImageManager,
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
    image_assets: Res<'w, Assets<Image>>,
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
        mut query,
        layout_query,
        mut vello_query,
        widget_render,
        context,
        font_assets,
        image_assets,
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

    vello_scene.reset();

    metrics.clear_quad_last_frame();

    let root_node = context.get_root_widget();
    // After layout computations update layouts and render scene.
    // Needs to be done in the correct order..
    // We also need to know if we are going back up the tree so we can pop the clipping and opacity layers.
    traverse_render_tree(
        root_node,
        &mut query,
        &default_font,
        &mut font_manager,
        &mut svg_manager,
        &mut image_manager,
        &mut metrics,
        &widget_render,
        &mut vello_scene,
        &font_assets,
        &image_assets,
        &svg_assets,
        &layout_query,
        root_node,
        true,
        camera_scale,
    );

    metrics.commit_quad_frame();
}

fn traverse_render_tree(
    root_node: Entity,
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
    metrics: &mut WidgetMetrics,
    widget_render: &Query<&WidgetRender>,
    vello_scene: &mut VelloScene,
    font_assets: &Assets<VelloFont>,
    image_assets: &Assets<Image>,
    svg_assets: &Assets<SvgAsset>,
    layout_query: &Query<&WidgetLayout>,
    current_node: Entity,
    should_render: bool,
    camera_scale: Vec2,
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

    let mut did_layer = false;
    if let Ok(widget_render) = widget_render.get(entity) {
        let parent_layout = parent.map(|parent| *layout_query.get(parent.parent()).unwrap());
        if (parent_layout.is_some() || root_node == entity) && should_render {
            did_layer = widget_render.render(
                vello_scene,
                layout,
                &parent_layout.unwrap_or_default(),
                default_font,
                font_assets,
                image_assets,
                svg_assets,
                font_manager,
                svg_manager,
                image_manager,
                metrics,
                styles,
                camera_scale,
            );
        }
    }

    let Some(children) = children.map(|c| c.iter().collect::<Vec<_>>()) else {
        if did_layer {
            vello_scene.pop_layer();
        }
        return;
    };

    for child in children.iter() {
        traverse_render_tree(
            root_node,
            query,
            default_font,
            font_manager,
            svg_manager,
            image_manager,
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
        );
    }

    if did_layer {
        vello_scene.pop_layer();
    }
}
