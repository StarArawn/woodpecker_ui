use crate::{
    convert_render_target::RenderTargetImages,
    hook_helper::StateMarker,
    image::ImageManager,
    layout::{system::ReflectedLayout, UiLayout},
    prelude::*,
    svg::{SvgAsset, SvgManager},
    DefaultFont,
};
use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_trait_query::One;
use bevy_vello::prelude::{UiVelloScene, VelloFont};

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
    vello_query: Query<'w, 's, &'static mut UiVelloScene>,
    widget_render: Query<'w, 's, &'static WidgetRender>,
    stacking_context_query: Query<'w, 's, &'static StackingContext>,
    context: Res<'w, WoodpeckerContext>,
    font_assets: Res<'w, Assets<VelloFont>>,
    image_assets: ResMut<'w, Assets<Image>>,
    svg_assets: Res<'w, Assets<SvgAsset>>,
    metrics: ResMut<'w, WidgetMetrics>,
    camera_query: Query<'w, 's, &'static Camera, With<WoodpeckerView>>,
    ui_layout: Res<'w, UiLayout>,
    animating_query: Query<'w, 's, (), With<Animating>>,
}

/// Whether the vello `Scene` rebuild in [`run`] can be skipped this frame and last frame's
/// content reused as-is.
///
/// `layout_dirty_this_frame` mirrors `layout::system::run`'s own verdict on whether it took its
/// skip-if-clean fast path (see that function's `SETTLE_FRAMES` for why that's a few frames
/// more conservative than a literal "anything changed this instant" check) -- via
/// `Changed<WoodpeckerStyle>`, this already transitively covers a playing `Transition` or
/// `AnimationTimeline`, since both mutate `WoodpeckerStyle` every active frame.
/// `widgets_rendered_this_frame` catches any widget re-render that didn't happen to touch
/// `WoodpeckerStyle`. `any_animating_entities` covers anything marked [`Animating`] -- a
/// not-yet-settled `Spring` or an indeterminate `Spinner`, for instance -- which deliberately
/// reads live state from inside a render closure or its own update system rather than a value
/// either of the other two signals would catch.
fn should_skip_render(
    layout_dirty_this_frame: bool,
    widgets_rendered_this_frame: usize,
    any_animating_entities: bool,
) -> bool {
    !layout_dirty_this_frame && widgets_rendered_this_frame == 0 && !any_animating_entities
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
        stacking_context_query,
        context,
        font_assets,
        mut image_assets,
        svg_assets,
        mut metrics,
        camera_query,
        ui_layout,
        animating_query,
    } = renderer_system_param;

    let Ok(mut vello_scene) = vello_query.single_mut() else {
        error!("Woodpecker UI: No vello scene spawned!");
        return;
    };

    let Ok(camera) = camera_query.single() else {
        error!("Woodpecker UI: No camera found or multiple UI cameras found.");
        return;
    };

    let widgets_rendered = metrics.get_widgets_rendered_since_last_frame();
    let any_animating = !animating_query.is_empty();
    let skip = should_skip_render(ui_layout.dirty_this_frame, widgets_rendered, any_animating);
    if skip {
        return;
    }

    // bevy_vello's own compositing already applies the window's scale factor, so scene
    // content here must stay in logical pixels -- scaling again would double-apply it.
    let camera_scale = Vec2::ONE;

    let camera_size = camera.logical_target_size().unwrap_or(Vec2::ZERO);

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
        &mut order,
        &mut render_commands,
        &mut query,
        &widget_render,
        &stacking_context_query,
        &layout_query,
        root_node,
        true,
    );

    // Once tree is traversed we sort the commands. `order` alone is already a single,
    // globally-comparable, correctly-nested paint-order rank (see
    // `layout::system::order_children_for_paint`'s doc comment) -- it's assigned by this same
    // bucketed traversal, so no separate `z` comparison is needed anymore.
    render_commands.sort_unstable_by_key(|a| a.order);

    // Now we can render with vello
    for command in render_commands {
        command.widget_render.render(
            &mut vello_scene,
            &command.layout,
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
    widget_render: &Query<&WidgetRender>,
    stacking_context_query: &Query<&StackingContext>,
    layout_query: &Query<&WidgetLayout>,
    current_node: Entity,
    should_render: bool,
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

    let mut order =
        (*order_counter as i32 + styles.z_index.and_then(|z| z.get_relative()).unwrap_or(0)) as u32;
    *order_counter += 1;

    let mut did_layer = false;
    // A settled `Modal` (opacity 1.0) skips its own `Layer` push/pop as a no-op, since its
    // clip rect is always the full viewport -- otherwise vello's nested layer depth gets
    // exceeded by every ever-opened ancestor modal, not just the visible chain.
    let is_modal_layer = matches!(widget_render.get(entity), Ok(WidgetRender::Layer))
        && stacking_context_query.contains(entity);
    let skip_modal_layer = is_modal_layer && styles.opacity >= 1.0;

    if let Ok(widget_render) = widget_render.get(entity) {
        let parent_layout = parent.map(|parent| *layout_query.get(parent.parent()).unwrap());
        if (parent_layout.is_some() || root_node == entity) && should_render && !skip_modal_layer {
            if matches!(widget_render, WidgetRender::Layer) {
                did_layer = true;
            }

            if styles.opacity > 0.0 && styles.opacity < 1.0 && !did_layer {
                did_layer = true;
                render_commands.push(RenderCommand {
                    order,
                    widget_render: WidgetRender::Layer,
                    layout: WidgetLayout(ReflectedLayout {
                        location: Vec2::splat(0.0),
                        size: Vec2::splat(10000.0),
                        ..Default::default()
                    }),
                    styles: *styles,
                });

                order = *order_counter;
                *order_counter += 1;
            }

            render_commands.push(RenderCommand {
                order,
                layout: *layout,
                widget_render: widget_render.clone(),
                styles: *styles,
            });
        }
    }

    let Some(children) = children.map(|c| c.iter().collect::<Vec<_>>()) else {
        if did_layer {
            let order = *order_counter;
            *order_counter += 1;
            render_commands.push(RenderCommand {
                order,
                widget_render: WidgetRender::PopLayer,
                ..Default::default()
            });
        }
        return;
    };
    // Same bucketed sibling order as `layout::system::traverse_layout_update` -- see
    // `order_children_for_paint`'s doc comment for why this (not just a `StackingContext`
    // check) is what keeps a descendant's `Global(n)` from escaping its own sibling group.
    let children = crate::layout::system::order_children_for_paint(&children, |child| {
        query
            .get(child)
            .ok()
            .and_then(|(_, _, styles, ..)| styles.z_index)
            .and_then(|z| z.get_global())
    });

    for child in children.iter() {
        traverse_render_tree(
            root_node,
            order_counter,
            render_commands,
            query,
            widget_render,
            stacking_context_query,
            layout_query,
            *child,
            should_render,
        );
    }

    if did_layer {
        let order = *order_counter;
        *order_counter += 1;
        render_commands.push(RenderCommand {
            order,
            widget_render: WidgetRender::PopLayer,
            ..Default::default()
        });
    }
}

#[derive(Default)]
struct RenderCommand {
    order: u32,
    layout: WidgetLayout,
    widget_render: WidgetRender,
    styles: WoodpeckerStyle,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_idle_frame_with_nothing_dirty_or_animating_is_skipped() {
        assert!(should_skip_render(false, 0, false));
    }

    #[test]
    fn a_dirty_layout_forces_a_rebuild() {
        assert!(!should_skip_render(true, 0, false));
    }

    #[test]
    fn a_widget_render_this_frame_forces_a_rebuild() {
        assert!(!should_skip_render(false, 1, false));
    }

    #[test]
    fn an_animating_entity_forces_a_rebuild() {
        assert!(!should_skip_render(false, 0, true));
    }
}
